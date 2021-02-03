//! The cdrom module implements the `PlatformDataProvider` trait for gathering userdata from a
//! mounted CDRom.

use super::{PlatformDataProvider, SettingsJson};
use serde::Deserialize;
use snafu::{ensure, OptionExt, ResultExt};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

pub(crate) struct CdromDataProvider;

impl CdromDataProvider {
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
            Some("xml") | Some("XML") => Self::ovf_user_data(user_data_file)?,
            // Since we only look for a specific list of file names, we should never find a file
            // with an extension we don't understand.
            Some(_) => unreachable!(),
            None => fs::read_to_string(&user_data_file).context(error::InputFileRead {
                path: user_data_file,
            })?,
        };

        if user_data_str.is_empty() {
            return Ok(None);
        }
        trace!("Received user data: {}", user_data_str);

        // Remove outer "settings" layer before sending to API
        let mut val: toml::Value =
            toml::from_str(&user_data_str).context(error::TOMLUserDataParse)?;
        let table = val.as_table_mut().context(error::UserDataNotTomlTable)?;
        let inner = table
            .remove("settings")
            .context(error::UserDataMissingSettings)?;

        let json = SettingsJson::from_val(&inner, "user data").context(error::SettingsToJSON)?;
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
        let reader = BufReader::new(file);

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

        // Create a valid utf8 str
        let decoded = std::str::from_utf8(&decoded_bytes).context(error::InvalidUTF8 {
            base64_string: base64_str.to_string(),
        })?;

        Ok(decoded.to_string())
    }
}

impl PlatformDataProvider for CdromDataProvider {
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

        #[snafu(display("Unable to read input file '{}': {}", path.display(), source))]
        InputFileRead { path: PathBuf, source: io::Error },

        #[snafu(display(
            "Invalid (non-utf8) output from base64 string '{}': {}",
            base64_string,
            source
        ))]
        InvalidUTF8 {
            base64_string: String,
            source: std::str::Utf8Error,
        },

        #[snafu(display("Error serializing TOML to JSON: {}", source))]
        SettingsToJSON { source: serde_json::error::Error },

        #[snafu(display("Error parsing TOML user data: {}", source))]
        TOMLUserDataParse { source: toml::de::Error },

        #[snafu(display("Found multiple user data files in '{}', expected 1", location))]
        UserDataFileCount { location: String },

        #[snafu(display("TOML data did not contain 'settings' section"))]
        UserDataMissingSettings,

        #[snafu(display("Data is not a TOML table"))]
        UserDataNotTomlTable,

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

        let actual_user_data = CdromDataProvider::ovf_user_data(xml).unwrap();

        assert_eq!(actual_user_data, expected_user_data)
    }

    #[test]
    fn test_read_xml_user_data() {
        let xml = test_data().join("ovf-env.xml");
        let expected_user_data = "settings.motd = \"hello\"";

        let actual_user_data = CdromDataProvider::ovf_user_data(xml).unwrap();

        assert_eq!(actual_user_data, expected_user_data)
    }
}
