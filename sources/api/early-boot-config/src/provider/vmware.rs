//! The vmware module implements the `PlatformDataProvider` trait for gathering userdata on VMware
//! via mounted CDRom or the guestinfo interface

use super::{PlatformDataProvider, SettingsJson};
use crate::compression::{expand_file_maybe, expand_slice_maybe, OptionalCompressionReader};
use async_trait::async_trait;
use serde::Deserialize;
use snafu::{ensure, ResultExt};
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Read;
use std::iter::FromIterator;
use std::path::Path;
use std::str;

use crate::provider::local_file;

pub(crate) struct VmwareDataProvider;

impl VmwareDataProvider {
    // This program expects that the CD-ROM is already mounted.  Mounting happens elsewhere in a
    // systemd unit file
    const CD_ROM_MOUNT: &'static str = "/media/cdrom";
    // A mounted CD-ROM may contain an OVF file or a user-supplied file named `user-data`
    const USER_DATA_FILENAMES: [&'static str; 5] = [
        "user-data",
        "ovf-env.xml",
        "OVF-ENV.XML",
        "ovf_env.xml",
        "OVF_ENV.XML",
    ];

    // The fields in which user data and its encoding are stored in guestinfo
    const GUESTINFO_USERDATA: &'static str = "guestinfo.userdata";
    const GUESTINFO_USERDATA_ENCODING: &'static str = "guestinfo.userdata.encoding";

    /// Read and decode user data from files via mounted CD-ROM
    fn cdrom_user_data() -> Result<Option<SettingsJson>> {
        // Given the list of acceptable filenames, ensure only 1 exists and parse
        // it for user data
        info!("Attempting to retrieve user data from mounted CD-ROM");
        let mut user_data_files = Self::USER_DATA_FILENAMES
            .iter()
            .map(|filename| Path::new(Self::CD_ROM_MOUNT).join(filename))
            .filter(|file| file.exists());

        let user_data_file = match user_data_files.next() {
            Some(file) => file,
            None => return Ok(None),
        };

        ensure!(
            user_data_files.next().is_none(),
            error::UserDataFileCountSnafu {
                place: Self::CD_ROM_MOUNT
            }
        );

        // XML files require extra processing, while a user-supplied file should already be in TOML
        // format
        info!("'{}' exists, using it", user_data_file.display());
        let user_data_str = match user_data_file.extension().and_then(OsStr::to_str) {
            Some("xml") | Some("XML") => Self::ovf_user_data(&user_data_file)?,
            // Since we only look for a specific list of file names, we should never find a file
            // with an extension we don't understand.
            Some(_) => unreachable!(),
            None => {
                // Read the file, decompressing it if compressed.
                expand_file_maybe(&user_data_file).context(error::InputFileReadSnafu {
                    path: &user_data_file,
                })?
            }
        };

        if user_data_str.is_empty() {
            return Ok(None);
        }

        // User data could be 700MB compressed!  Eek!  :)
        if user_data_str.len() <= 2048 {
            trace!("Received user data: {}", user_data_str);
        } else {
            trace!(
                "Received long user data, starts with: {}",
                // (this isn't perfect because chars aren't grapheme clusters, but will error
                // toward printing the whole input, which is fine)
                String::from_iter(user_data_str.chars().take(2048))
            );
        }

        let json = SettingsJson::from_toml_str(&user_data_str, "user data from CD-ROM").context(
            error::SettingsToJsonSnafu {
                from: user_data_file.display().to_string(),
            },
        )?;

        Ok(Some(json))
    }

    /// Read and base64 decode user data contained in an OVF file
    // In VMware, user data is supplied to the host via an XML file.  Within
    // the XML file, there is a `PropertySection` that contains `Property` elements
    // with attributes.  User data is base64 encoded inside a `Property` element with
    // the attribute "user-data".
    // <Property key="user-data" value="1234abcd"/>
    fn ovf_user_data<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();
        let file = File::open(path).context(error::InputFileReadSnafu { path })?;
        let reader = OptionalCompressionReader::new(BufReader::new(file));

        // Deserialize the OVF file, dropping everything we don't care about
        let ovf: Environment =
            serde_xml_rs::from_reader(reader).context(error::XmlDeserializeSnafu { path })?;

        // We have seen the keys in the `Property` section be "namespaced" like "oe:key" or
        // "of:key".  Since we aren't trying to validate the schema beyond the presence of the
        // elements we care about, we can ignore the namespacing.  An example of this type of
        // namespacing can be found in the unit test sample data. `serde_xml_rs` effectively
        // ignores these namespaces and returns "key" / "value":
        // https://github.com/Rreverser/serde-xml-rs/issues/64#issuecomment=540448434
        let mut base64_str = String::new();
        let user_data_key = "user-data";
        for property in ovf.property_section.properties {
            if property.key == user_data_key {
                base64_str = property.value;
                break;
            }
        }

        // Base64 decode the &str
        let decoded_bytes = base64::decode(base64_str).context(error::Base64DecodeSnafu {
            what: "OVF user data",
        })?;

        // Decompress the data if it's compressed
        let decoded = expand_slice_maybe(&decoded_bytes).context(error::DecompressionSnafu {
            what: "OVF user data",
        })?;

        Ok(decoded)
    }

    /// Read and decode user data based on values retrieved from the guestinfo interface
    fn guestinfo_user_data() -> Result<Option<SettingsJson>> {
        info!("Attempting to retrieve user data via guestinfo interface");

        // It would be extremely odd to get here and not be on VMware, but check anyway
        ensure!(vmw_backdoor::is_vmware_cpu(), error::NotVmwareSnafu);

        // `guestinfo.userdata.encoding` informs us how to handle the data in the
        // `guestinfo.userdata` field
        let maybe_encoding = Self::backdoor_get_bytes(Self::GUESTINFO_USERDATA_ENCODING)?;
        let user_data_encoding: UserDataEncoding = match maybe_encoding {
            Some(val) => {
                let encoding_str = String::from_utf8(val).context(error::InvalidUtf8Snafu {
                    what: Self::GUESTINFO_USERDATA_ENCODING,
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
                    Self::GUESTINFO_USERDATA_ENCODING
                );
                UserDataEncoding::Raw
            }
        };

        let user_data_bytes = match Self::backdoor_get_bytes(Self::GUESTINFO_USERDATA)? {
            Some(val) => val,
            None => return Ok(None),
        };

        let user_data_string = match user_data_encoding {
            // gzip+base64 is gzip'ed user data that is base64 encoded
            UserDataEncoding::Base64 | UserDataEncoding::GzipBase64 => {
                info!("Decoding user data");
                let mut reader = Cursor::new(user_data_bytes);
                let decoder = base64::read::DecoderReader::new(&mut reader, base64::STANDARD);

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
                    what: Self::GUESTINFO_USERDATA,
                })?
            }
        };

        let json = SettingsJson::from_toml_str(user_data_string, "user data from guestinfo")
            .context(error::SettingsToJsonSnafu { from: "guestinfo" })?;
        Ok(Some(json))
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
impl PlatformDataProvider for VmwareDataProvider {
    async fn platform_data(
        &self,
    ) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();

        // First read from any site-local defaults. It's unlikely that this file will exist, but
        // this is consistent with other platforms.
        match local_file::user_data_defaults()? {
            Some(s) => output.push(s),
            None => info!(
                "No user data found via site defaults file: {}",
                local_file::USER_DATA_DEFAULTS_FILE
            ),
        }

        // Attempt to read from a local file next. This comes from the private settings filesystem
        // rather than the data storage filesystem, and is also unlikely to exist.
        match local_file::user_data()? {
            Some(s) => output.push(s),
            None => info!(
                "No user data found via local file: {}",
                local_file::USER_DATA_FILE
            ),
        }

        // Then look at the CD-ROM for user data. This isn't the preferred method of supplying user
        // data, but might still be used.
        match Self::cdrom_user_data()? {
            Some(s) => output.push(s),
            None => info!("No user data found via CD-ROM"),
        }

        // Now, check guestinfo which is the preferred method. If it's populated, it will override
        // any earlier settings found.
        match Self::guestinfo_user_data()? {
            Some(s) => output.push(s),
            None => warn!("No user data found via guestinfo"),
        }

        // Finally, apply any site-local overrides. It's unlikely to exist but again, this is
        // consistent with other platforms.
        match local_file::user_data_overrides()? {
            Some(s) => output.push(s),
            None => info!(
                "No user data found via site overrides file: {}",
                local_file::USER_DATA_OVERRIDES_FILE
            ),
        }

        Ok(output)
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

// Minimal expected structure for an OVF file with user data
#[derive(Debug, Deserialize)]
struct Environment {
    #[serde(rename = "PropertySection", default)]
    pub property_section: PropertySection,
}

#[derive(Default, Debug, Deserialize)]
struct PropertySection {
    #[serde(rename = "Property", default)]
    pub properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
struct Property {
    pub key: String,
    pub value: String,
}

// =^..^=   =^..^=   =^..^=   =^..^=

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub(crate) enum Error {
        #[snafu(display("VMware backdoor: failed to '{}': '{}'", op, source))]
        Backdoor {
            op: String,
            source: vmw_backdoor::VmwError,
        },

        #[snafu(display("Unable to decode base64 in {}: '{}'", what, source))]
        Base64Decode {
            what: String,
            source: base64::DecodeError,
        },

        #[snafu(display("Failed to decompress {}: {}", what, source))]
        Decompression { what: String, source: io::Error },

        #[snafu(display("Failed to fetch key '{}' from guestinfo: {}", what, source))]
        GuestInfo {
            what: String,
            source: vmw_backdoor::VmwError,
        },

        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

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
            source: crate::settings::Error,
        },

        #[snafu(display("Unknown user data encoding: '{}': {}", encoding, source))]
        UnknownEncoding {
            encoding: String,
            source: serde_plain::Error,
        },

        #[snafu(display("Found multiple user data files in '{}', expected 1", place))]
        UserDataFileCount { place: String },

        #[snafu(display("Unable to deserialize XML from: '{}': {}", path.display(), source))]
        XmlDeserialize {
            path: PathBuf,
            source: serde_xml_rs::Error,
        },
    }
}

type Result<T> = std::result::Result<T, error::Error>;

#[cfg(test)]
mod test {
    use super::*;
    use std::path::PathBuf;

    fn test_data() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("test_data")
    }

    #[test]
    fn test_read_xml_user_data_namespaced_keys() {
        let xml = test_data().join("namespaced_keys.xml");
        let expected_user_data = "settings.motd = \"hello\"";

        let actual_user_data = VmwareDataProvider::ovf_user_data(xml).unwrap();

        assert_eq!(actual_user_data, expected_user_data)
    }

    #[test]
    fn test_read_xml_user_data() {
        let xml = test_data().join("ovf-env.xml");
        let expected_user_data = "settings.motd = \"hello\"";

        let actual_user_data = VmwareDataProvider::ovf_user_data(xml).unwrap();

        assert_eq!(actual_user_data, expected_user_data)
    }
}
