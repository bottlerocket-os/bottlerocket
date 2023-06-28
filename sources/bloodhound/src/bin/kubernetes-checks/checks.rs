use bloodhound::{
    check_file_not_mode, ensure_file_owner_and_group_root,
    results::{Checker, CheckerMetadata, CheckerResult, Mode},
};
use libc::{S_IWGRP, S_IWOTH, S_IXGRP, S_IXOTH, S_IXUSR};

// Bottlerocket doesn't use the standard path for most of these files ¯\_(ツ)_/¯
const KUBELET_SERVICE_FILE: &str = "/etc/systemd/system/kubelet.service.d/exec-start.conf";

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04010100Checker {}

impl Checker for K8S04010100Checker {
    fn execute(&self) -> CheckerResult {
        let no_x_xw_xw = S_IXUSR | S_IXGRP | S_IWGRP | S_IXOTH | S_IWOTH;
        check_file_not_mode(KUBELET_SERVICE_FILE, no_x_xw_xw)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the kubelet service file permissions are set to 644 or more restrictive".to_string(),
            id: "4.1.1".to_string(),
            level: 1,
            name: "k8s04010100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04010200Checker {}

impl Checker for K8S04010200Checker {
    fn execute(&self) -> CheckerResult {
        ensure_file_owner_and_group_root(KUBELET_SERVICE_FILE)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the kubelet service file ownership is set to root:root".to_string(),
            id: "4.1.2".to_string(),
            level: 1,
            name: "k8s04010200".to_string(),
            mode: Mode::Automatic,
        }
    }
}
