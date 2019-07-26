//! Convenience binary for reading a JSON document on stdin and outputting the canonical JSON form
//! on stdout.

use olpc_cjson::CanonicalFormatter;
use serde::Serialize;
use serde_json::Serializer;
use std::io;

type Result<T> = std::result::Result<T, Box<std::error::Error>>;

fn main() -> Result<()> {
    let mut ser = Serializer::with_formatter(io::stdout(), CanonicalFormatter::new());
    let value: serde_json::Value = serde_json::from_reader(io::stdin())?;
    value.serialize(&mut ser)?;
    Ok(())
}
