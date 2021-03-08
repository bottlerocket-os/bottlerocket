//! The vmware module implements the `PlatformDataProvider` trait for gathering userdata on VMWare
//! via mounted CDRom or the guestinfo interface

use super::{PlatformDataProvider, SettingsJson};
use crate::compression::{expand_file_maybe, expand_slice_maybe, OptionalCompressionReader};
use serde::Deserialize;
use snafu::{ensure, ResultExt};
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use std::iter::FromIterator;
use std::path::Path;

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

    /// Given the list of acceptable filenames, ensure only 1 exists and parse
    /// it for user data
    fn user_data() -> Result<Option<SettingsJson>> {
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
            error::UserDataFileCount {
                location: Self::CD_ROM_MOUNT
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
                expand_file_maybe(&user_data_file).context(error::InputFileRead {
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

        let json = SettingsJson::from_toml_str(&user_data_str, "user data").context(
            error::SettingsToJSON {
                from: user_data_file.display().to_string(),
            },
        )?;

        Ok(Some(json))
    }

    /// Read and base64 decode user data contained in an OVF file
    // In VMWare, user data is supplied to the host via an XML file.  Within
    // the XML file, there is a `PropertySection` that contains `Property` elements
    // with attributes.  User data is base64 encoded inside a `Property` element with
    // the attribute "user-data".
    // <Property key="user-data" value="1234abcd"/>
    fn ovf_user_data<P: AsRef<Path>>(path: P) -> Result<String> {
        let path = path.as_ref();
        let file = File::open(path).context(error::InputFileRead { path })?;
        let reader = OptionalCompressionReader::new(BufReader::new(file));

        // Deserialize the OVF file, dropping everything we don't care about
        let ovf: Environment =
            serde_xml_rs::from_reader(reader).context(error::XmlDeserialize { path })?;

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
        let decoded_bytes = base64::decode(&base64_str).context(error::Base64Decode {
            base64_string: base64_str.to_string(),
        })?;

        // Decompress the data if it's compressed
        let decoded = expand_slice_maybe(&decoded_bytes).context(error::Decompression {
            what: "OVF user data",
        })?;

        Ok(decoded)
    }
}

impl PlatformDataProvider for VmwareDataProvider {
    fn platform_data(&self) -> std::result::Result<Vec<SettingsJson>, Box<dyn std::error::Error>> {
        let mut output = Vec::new();

        match Self::user_data() {
            Err(e) => return Err(e).map_err(Into::into),
            Ok(None) => warn!("No user data found."),
            Ok(Some(s)) => output.push(s),
        }
        Ok(output)
    }
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
    #[snafu(visibility = "pub(super)")]
    pub(crate) enum Error {
        #[snafu(display("Unable to base64 decode string '{}': '{}'", base64_string, source))]
        Base64Decode {
            base64_string: String,
            source: base64::DecodeError,
        },

        #[snafu(display("Failed to decompress {}: {}", what, source))]
        Decompression { what: String, source: io::Error },

        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display("Unable to serialize settings from {}: {}", from, source))]
        SettingsToJSON {
            from: String,
            source: crate::settings::Error,
        },

        #[snafu(display("Found multiple user data files in '{}', expected 1", location))]
        UserDataFileCount { location: String },

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
