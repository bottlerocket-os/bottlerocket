use bloodhound::{
    check_file_not_mode, ensure_file_owner_and_group_root,
    results::{Checker, CheckerMetadata, CheckerResult, Mode},
};
use libc::{S_IRWXG, S_IRWXO, S_IWGRP, S_IWOTH, S_IXGRP, S_IXOTH, S_IXUSR};

// Bottlerocket doesn't use the standard path for most of these files ¯\_(ツ)_/¯
const KUBELET_SERVICE_FILE: &str = "/etc/systemd/system/kubelet.service.d/exec-start.conf";
const KUBELET_KUBECONFIG_FILE: &str = "/etc/kubernetes/kubelet/kubeconfig";
const KUBELET_CLIENT_CA_FILE: &str = "/etc/kubernetes/pki/ca.crt";
const KUBELET_CONF_FILE: &str = "/etc/kubernetes/kubelet/config";

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

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04010500Checker {}

impl Checker for K8S04010500Checker {
    fn execute(&self) -> CheckerResult {
        let no_x_xw_xw = S_IXUSR | S_IXGRP | S_IWGRP | S_IXOTH | S_IWOTH;
        check_file_not_mode(KUBELET_KUBECONFIG_FILE, no_x_xw_xw)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --kubeconfig kubelet.conf file permissions are set to 644 or more restrictive".to_string(),
            id: "4.1.5".to_string(),
            level: 1,
            name: "k8s04010500".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04010600Checker {}

impl Checker for K8S04010600Checker {
    fn execute(&self) -> CheckerResult {
        ensure_file_owner_and_group_root(KUBELET_KUBECONFIG_FILE)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --kubeconfig kubelet.conf file ownership is set to root:root"
                .to_string(),
            id: "4.1.6".to_string(),
            level: 1,
            name: "k8s04010600".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04010700Checker {}

impl Checker for K8S04010700Checker {
    fn execute(&self) -> CheckerResult {
        let no_x_xwr_xwr = S_IXUSR | S_IRWXG | S_IRWXO;
        check_file_not_mode(KUBELET_CLIENT_CA_FILE, no_x_xwr_xwr)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the certificate authorities file permissions are set to 600 or more restrictive".to_string(),
            id: "4.1.7".to_string(),
            level: 1,
            name: "k8s04010700".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04010800Checker {}

impl Checker for K8S04010800Checker {
    fn execute(&self) -> CheckerResult {
        ensure_file_owner_and_group_root(KUBELET_CLIENT_CA_FILE)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title:
                "Ensure that the client certificate authorities file ownership is set to root:root"
                    .to_string(),
            id: "4.1.8".to_string(),
            level: 1,
            name: "k8s04010800".to_string(),
            mode: Mode::Automatic,
        }
    }
}


// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04010900Checker {}

impl Checker for K8S04010900Checker {
    fn execute(&self) -> CheckerResult {
        let no_x_xwr_xwr = S_IXUSR | S_IRWXG | S_IRWXO;
        check_file_not_mode(KUBELET_CONF_FILE, no_x_xwr_xwr)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "If the kubelet config.yaml configuration file is being used validate permissions set to 600 or more restrictive".to_string(),
            id: "4.1.9".to_string(),
            level: 1,
            name: "k8s04010900".to_string(),
            mode: Mode::Automatic,
        }
    }
}
