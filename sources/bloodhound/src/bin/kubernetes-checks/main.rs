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
        "k8s04010100" => Box::new(K8S04010100Checker {}),
        "k8s04010200" => Box::new(K8S04010200Checker {}),
        "k8s04010300" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "If proxy kubeconfig file exists ensure permissions are set to 644 or more restrictive".to_string(),
            id: "4.1.3".to_string(),
            level: 1,
        }),
        "k8s04010400" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "If proxy kubeconfig file exists ensure ownership is set to root:root".to_string(),
            id: "4.1.4".to_string(),
            level: 1,
        }),
        "k8s04010500" => Box::new(K8S04010500Checker {}),
        "k8s04010600" => Box::new(K8S04010600Checker {}),
        "k8s04010700" => Box::new(K8S04010700Checker {}),
        "k8s04010800" => Box::new(K8S04010800Checker {}),
        "k8s04010900" => Box::new(K8S04010900Checker {}),
        "k8s04011000" => Box::new(K8S04011000Checker {}),
        "k8s04020100" => Box::new(K8S04020100Checker {}),
        "k8s04020200" => Box::new(K8S04020200Checker {}),
        "k8s04020300" => Box::new(K8S04020300Checker {}),
        "k8s04020400" => Box::new(K8S04020400Checker {}),
        "k8s04020500" => Box::new(K8S04020500Checker {}),
        "k8s04020600" => Box::new(K8S04020600Checker {}),
        "k8s04020700" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "Ensure that the --hostname-override argument is not set (not valid for Bottlerocket)".to_string(),
            id: "4.2.7".to_string(),
            level: 1,
        }),
        "k8s04020800" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "Ensure that the eventRecordQPS argument is set to a level which ensures appropriate event capture".to_string(),
            id: "4.2.8".to_string(),
            level: 2,
        }),
        "k8s04020900" => Box::new(K8S04020900Checker {}),
        // IAM (external) auth is used, so certificate rotation does not apply. See EKS CIS Benchmark.
        "k8s04021000" => Box::new(ManualChecker {
            name: cmd_name.to_string(),
            title: "Ensure that the --rotate-certificates argument is not set to false (not valid for Bottlerocket)".to_string(),
            id: "4.2.10".to_string(),
            level: 1,
        }),
        "k8s04021100" => Box::new(K8S04021100Checker {}),
        "k8s04021200" => Box::new(K8S04021200Checker {}),
        "k8s04021300" => Box::new(K8S04021300Checker {}),
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
