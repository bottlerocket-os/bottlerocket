mod checks;

use bloodhound::results::*;
use checks::*;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd_name = Path::new(&args[0])
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    let checker: Box<dyn Checker> = match cmd_name {
        "br01010101" => Box::new(BR01010101Checker {}),
        "br01020100" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "Ensure software update repositories are configured".to_string(),
            id: "1.2.1".to_string(),
            level: 1,
        }),
        "br01030100" => Box::new(BR01030100Checker {}),
        "br01040100" => Box::new(BR01040100Checker {}),
        "br01040200" => Box::new(BR01040200Checker {}),
        "br01040300" => Box::new(BR01040300Checker {}),
        "br01040400" => Box::new(BR01040400Checker {}),
        "br01050100" => Box::new(BR01050100Checker {}),
        "br01050200" => Box::new(BR01050200Checker {}),
        "br01060000" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "Ensure updates, patches, and additional security software are installed"
                .to_string(),
            id: "1.6".to_string(),
            level: 1,
        }),
        "br02010101" => Box::new(BR02010101Checker {}),
        "br03010100" => Box::new(BR03010100Checker {}),
        "br03020100" => Box::new(BR03020100Checker {}),
        "br03020200" => Box::new(BR03020200Checker {}),
        "br03020300" => Box::new(BR03020300Checker {}),
        "br03020400" => Box::new(BR03020400Checker {}),
        "br03020500" => Box::new(BR03020500Checker {}),
        "br03040103" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "Ensure IPv4 outbound and established connections are configured".to_string(),
            id: "3.4.1.3".to_string(),
            level: 1,
        }),
        "br03040203" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "Ensure IPv6 outbound and established connections are configured".to_string(),
            id: "3.4.2.3".to_string(),
            level: 1,
        }),
        &_ => {
            eprintln!("Command {} is not supported.", cmd_name);
            return;
        }
    };

    // Check if the metadata subcommand is being called
    let get_metadata = env::args().nth(1).unwrap_or_default() == "metadata";

    if get_metadata {
        let metadata = checker.metadata();
        println!("{}", metadata);
    } else {
        let result = checker.execute();
        println!("{}", result);
    }
}
