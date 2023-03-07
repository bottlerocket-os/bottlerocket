use argh::FromArgs;
use std::str::FromStr;

pub const DEFAULT_CHECK_PATH: &str = "/usr/libexec/cis-checks/bottlerocket";

#[derive(Clone, Debug)]
pub enum Format {
    Text,
    Json,
}

impl FromStr for Format {
    type Err = ();

    fn from_str(value: &str) -> Result<Format, Self::Err> {
        match value.to_ascii_lowercase().as_str() {
            "text" => Ok(Format::Text),
            "json" => Ok(Format::Json),
            _ => Err(()),
        }
    }
}

fn str_to_format(value: &str) -> Result<Format, String> {
    match Format::from_str(value) {
        Ok(f) => Ok(f),
        _ => Err("invalid format, options are 'text' or 'json'".to_string()),
    }
}

#[derive(FromArgs, Debug)]
/// Command line arguments for the bloodhound program.
pub struct Arguments {
    /// path to the directory containing checker binaries
    #[argh(option, default = "DEFAULT_CHECK_PATH.to_string()", short = 'c')]
    pub check_dir: String,
    /// format of the output
    #[argh(
        option,
        default = "Format::Text",
        from_str_fn(str_to_format),
        short = 'f'
    )]
    pub format: Format,
    /// the CIS benchmark compliance level to check
    #[argh(option, default = "1", short = 'l')]
    pub level: u8,
    /// write output to a file at given path [default: stdout]
    #[argh(option, short = 'o')]
    pub output: Option<String>,
}
