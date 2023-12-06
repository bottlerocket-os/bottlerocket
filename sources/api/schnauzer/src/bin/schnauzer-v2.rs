//! schnauzer-v2
//!
//! A settings generator for rendering handlebars templates using data from the Bottlerocket API.
use std::process;

// Returning a Result from main makes it print a Debug representation of the error, but with Snafu
// we have nice Display representations of the error, so we wrap "main" (run) and print any error.
// https://github.com/shepmaster/snafu/issues/110
#[tokio::main]
async fn main() {
    if let Err(e) = schnauzer::v2::cli::run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}
