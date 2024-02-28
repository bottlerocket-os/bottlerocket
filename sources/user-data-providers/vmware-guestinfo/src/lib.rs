/// VMWare guestinfo
#[macro_use]
extern crate log;

use async_trait::async_trait;
use serde::Deserialize;
use snafu::{ensure, ResultExt};
use std::io::{Cursor, Read};
use user_data_provider::provider::UserDataProvider;
use user_data_provider::{compression::OptionalCompressionReader, settings::SettingsJson};

// The fields in which user data and its encoding are stored in guestinfo
const GUESTINFO_USERDATA: &str = "guestinfo.userdata";
const GUESTINFO_USERDATA_ENCODING: &str = "guestinfo.userdata.encoding";

pub struct VmwareGuestinfo;

impl VmwareGuestinfo {
    /// Fetch the user data's encoding from guestinfo.
    // `guestinfo.userdata.encoding` informs us how to handle the data in the
    // `guestinfo.userdata` field
    fn fetch_encoding() -> Result<UserDataEncoding> {
        let maybe_encoding = Self::backdoor_get_bytes(GUESTINFO_USERDATA_ENCODING)?;
        let user_data_encoding: UserDataEncoding = match maybe_encoding {
            Some(val) => {
                let encoding_str = String::from_utf8(val).context(error::InvalidUtf8Snafu {
                    what: GUESTINFO_USERDATA_ENCODING,
                })?;
                info!("Found user data encoding: {}", encoding_str);

                serde_plain::from_str(&encoding_str).context(error::UnknownEncodingSnafu {
                    encoding: encoding_str,
                })?
            }

            // The cloudinit VMware guestinfo data provider assumes any user data without an
            // associated encoding means raw data is being passed.  We will follow suit here.
            None => {
                warn!(
                    "'{}' unset, assuming raw user data",
                    GUESTINFO_USERDATA_ENCODING
                );
                UserDataEncoding::Raw
            }
        };

        Ok(user_data_encoding)
    }

    /// Request a key's value from guestinfo
    fn backdoor_get_bytes(key: &str) -> Result<Option<Vec<u8>>> {
        // Probe and access the VMware backdoor.  `kernel lockdown(7)` may block "privileged"
        // mode because of its use of `iopl()`; the 5.15 kernels have it disabled regardless
        // of lockdown mode. If this fails, fall back to "unprivileged" access without first
        // requesting access to the relevant IO ports. KVM and VMware both have them special-
        // cased in their emulation to not raise an exception to the guest OS and things
        // should work out.
        let mut backdoor = vmw_backdoor::probe_backdoor_privileged()
            .or_else(|e| {
                debug!(
                    "Unable to access guestinfo via privileged mode, using unprivileged: {}",
                    e
                );
                vmw_backdoor::probe_backdoor()
            })
            .context(error::BackdoorSnafu {
                op: "probe and acquire access",
            })?;

        let mut erpc = backdoor
            .open_enhanced_chan()
            .context(error::BackdoorSnafu {
                op: "open eRPC channel",
            })?;

        erpc.get_guestinfo(key.as_bytes())
            .context(error::GuestInfoSnafu { what: key })
    }
}

#[async_trait]
impl UserDataProvider for VmwareGuestinfo {
    async fn user_data(
        &self,
    ) -> std::result::Result<Option<SettingsJson>, Box<dyn std::error::Error>> {
        info!("Attempting to retrieve user data via guestinfo interface");

        // It would be extremely odd to get here and not be on VMware, but check anyway
        ensure!(vmw_backdoor::is_vmware_cpu(), error::NotVmwareSnafu);

        let user_data_encoding = Self::fetch_encoding()?;
        let user_data_bytes = match Self::backdoor_get_bytes(GUESTINFO_USERDATA)? {
            Some(val) => val,
            None => return Ok(None),
        };

        let user_data_string = match user_data_encoding {
            // gzip+base64 is gzip'ed user data that is base64 encoded
            UserDataEncoding::Base64 | UserDataEncoding::GzipBase64 => {
                info!("Decoding user data");
                let mut reader = Cursor::new(user_data_bytes);
                let decoder = base64::read::DecoderReader::new(
                    &mut reader,
                    &base64::engine::general_purpose::STANDARD,
                );

                // Decompresses the data if it is gzip'ed
                let mut output = String::new();
                let mut compression_reader = OptionalCompressionReader::new(decoder);
                compression_reader.read_to_string(&mut output).context(
                    error::DecompressionSnafu {
                        what: "guestinfo user data",
                    },
                )?;
                output
            }

            UserDataEncoding::Raw => {
                String::from_utf8(user_data_bytes).context(error::InvalidUtf8Snafu {
                    what: GUESTINFO_USERDATA,
                })?
            }
        };

        let json = SettingsJson::from_toml_str(user_data_string, "guestinfo")
            .context(error::SettingsToJsonSnafu { from: "guestinfo" })?;
        Ok(Some(json))
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=

// Acceptable user data encodings
// When case-insensitive de/serialization is finalized, that's what we would want to use
// here instead of aliases: https://github.com/serde-rs/serde/pull/1902
#[derive(Debug, Deserialize)]
enum UserDataEncoding {
    #[serde(alias = "b64")]
    #[serde(alias = "B64")]
    #[serde(alias = "base64")]
    Base64,
    #[serde(alias = "gz+b64")]
    #[serde(alias = "Gz+B64")]
    #[serde(alias = "gzip+base64")]
    #[serde(alias = "Gzip+Base64")]
    GzipBase64,
    Raw,
}

// =^..^=   =^..^=   =^..^=   =^..^=

mod error {
    use snafu::Snafu;
    use std::io;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("VMware backdoor: failed to '{}': '{}'", op, source))]
        Backdoor {
            op: String,
            source: vmw_backdoor::VmwError,
        },

        #[snafu(display("Failed to decompress {}: {}", what, source))]
        Decompression { what: String, source: io::Error },

        #[snafu(display("Failed to fetch key '{}' from guestinfo: {}", what, source))]
        GuestInfo {
            what: String,
            source: vmw_backdoor::VmwError,
        },

        #[snafu(display("'{}' contains invalid utf-8: {}", what, source))]
        InvalidUtf8 {
            what: String,
            source: std::string::FromUtf8Error,
        },

        #[snafu(display(
            "Unable to read user data from guestinfo, this is not a VMware virtual CPU"
        ))]
        NotVmware,

        #[snafu(display("Unable to serialize settings from {}: {}", from, source))]
        SettingsToJson {
            from: String,
            source: user_data_provider::settings::Error,
        },

        #[snafu(display("Unknown user data encoding: '{}': {}", encoding, source))]
        UnknownEncoding {
            encoding: String,
            source: serde_plain::Error,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;
