use std::{collections::HashSet, fs::File, path::Path};

use bloodhound::{
    check_file_not_mode, ensure_file_owner_and_group_root,
    results::{CheckStatus, Checker, CheckerMetadata, CheckerResult, Mode},
};
use libc::{S_IRWXG, S_IRWXO, S_IWGRP, S_IWOTH, S_IXGRP, S_IXOTH, S_IXUSR};
use serde::Deserialize;

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

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04011000Checker {}

impl Checker for K8S04011000Checker {
    fn execute(&self) -> CheckerResult {
        ensure_file_owner_and_group_root(KUBELET_CONF_FILE)
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "If the kubelet config.yaml configuration file is being used validate file ownership is set to root:root"
                .to_string(),
            id: "4.1.10".to_string(),
            level: 1,
            name: "k8s04011000".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04020100Checker {}

impl Checker for K8S04020100Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct Anonymous {
            enabled: bool,
        }

        #[derive(Deserialize)]
        struct Authentication {
            anonymous: Anonymous,
        }

        #[derive(Deserialize)]
        struct KubeletConfig {
            authentication: Authentication,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if config.authentication.anonymous.enabled {
                    result.error = "anonymous authentication is configured".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                result.error = "unable to parse kubelet config".to_string()
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --anonymous-auth argument is set to false".to_string(),
            id: "4.2.1".to_string(),
            level: 1,
            name: "k8s04020100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04020200Checker {}

impl Checker for K8S04020200Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct Authorization {
            mode: String,
        }

        #[derive(Deserialize)]
        struct KubeletConfig {
            authorization: Authorization,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if config.authorization.mode == "AlwaysAllow" {
                    result.error = "AlwaysAllow authorization is configured".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                result.error = "unable to parse kubelet config".to_string()
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --authorization-mode argument is not set to AlwaysAllow"
                .to_string(),
            id: "4.2.2".to_string(),
            level: 1,
            name: "k8s04020200".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04020300Checker {}

impl Checker for K8S04020300Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct X509 {
            #[serde(rename = "clientCAFile")]
            client_ca_file: String,
        }

        #[derive(Deserialize)]
        struct Authentication {
            x509: X509,
        }

        #[derive(Deserialize)]
        struct KubeletConfig {
            authentication: Authentication,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if !config.authentication.x509.client_ca_file.is_empty()
                    && Path::new(&config.authentication.x509.client_ca_file).exists()
                {
                    result.status = CheckStatus::PASS;
                } else {
                    result.error = "CA file not set to expected path".to_string();
                    result.status = CheckStatus::FAIL;
                }
            } else {
                result.error = "unable to parse kubelet config".to_string()
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --client-ca-file argument is set as appropriate".to_string(),
            id: "4.2.3".to_string(),
            level: 1,
            name: "k8s04020300".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04020400Checker {}

impl Checker for K8S04020400Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "readOnlyPort")]
            read_only_port: i32,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if config.read_only_port != 0 {
                    result.error = "Kubelet readOnlyPort not set to 0".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                result.error = "unable to parse kubelet config".to_string()
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Verify that the --read-only-port argument is set to 0".to_string(),
            id: "4.2.4".to_string(),
            level: 1,
            name: "k8s04020400".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04020500Checker {}

impl Checker for K8S04020500Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "streamingConnectionIdleTimeout")]
            streaming_connection_idle_timeout: i32,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if config.streaming_connection_idle_timeout == 0 {
                    result.error = "Kubelet streamingConnectionIdleTimeout is set to 0".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                // Normally this value should not be present in the config file, so deserialization is expected to fail.
                result.status = CheckStatus::PASS;
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --streaming-connection-idle-timeout argument is not set to 0"
                .to_string(),
            id: "4.2.5".to_string(),
            level: 1,
            name: "k8s04020500".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04020600Checker {}

impl Checker for K8S04020600Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "makeIPTablesUtilChains")]
            make_iptables_util_chains: bool,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if !config.make_iptables_util_chains {
                    result.error = "Kubelet makeIPTablesUtilChains is disabled".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                // Normally this value should not be present in the config file, so deserialization is expected to fail.
                result.status = CheckStatus::PASS;
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --make-iptables-util-chains argument is set to true"
                .to_string(),
            id: "4.2.6".to_string(),
            level: 1,
            name: "k8s04020600".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04020900Checker {}

impl Checker for K8S04020900Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "tlsCertFile")]
            tls_cert_file: String,
            #[serde(rename = "tlsPrivateKeyFile")]
            tls_private_key_file: String,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if (!config.tls_cert_file.is_empty() && Path::new(&config.tls_cert_file).exists())
                    && (!config.tls_private_key_file.is_empty()
                        && Path::new(&config.tls_private_key_file).exists())
                {
                    result.status = CheckStatus::PASS;
                } else {
                    result.error = "TLS files not set to expected path".to_string();
                    result.status = CheckStatus::FAIL;
                }
            } else {
                // If certs not provided then `serverTLSBootstrap` will be used. Deserialization expected to fail in this case.
                result.status = CheckStatus::PASS;
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --tls-cert-file and --tls-private-key-file arguments are set as appropriate".to_string(),
            id: "4.2.9".to_string(),
            level: 1,
            name: "k8s04020900".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04021000Checker {}

// Not actually applicable for Bottlerocket, but leaving logic here in case we
// make any changes in the future.
impl Checker for K8S04021000Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "rotateCertificates")]
            rotate_certificates: bool,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if !config.rotate_certificates {
                    result.error = "Kubelet rotateCertificates is disabled".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                // Default value is `false`, so it is a failure if this is not in the config file.
                result.error = "Kubelet rotateCertificates is disabled".to_string();
                result.status = CheckStatus::FAIL;
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the --rotate-certificates argument is not set to false".to_string(),
            id: "4.2.10".to_string(),
            level: 1,
            name: "k8s04021000".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04021100Checker {}

impl Checker for K8S04021100Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct FeatureGates {
            #[serde(rename = "RotateKubeletServerCertificate")]
            rotate_kubelet_server_certificate: bool,
        }

        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "featureGates")]
            feature_gates: FeatureGates,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if !config.feature_gates.rotate_kubelet_server_certificate {
                    result.error = "Kubelet RotateKubeletServerCertificate is disabled".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                // Feature gate has been defaulted to enabled since k8s 1.12, so if it is not found that is fine
                result.status = CheckStatus::PASS;
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Verify that the RotateKubeletServerCertificate argument is set to true"
                .to_string(),
            id: "4.2.11".to_string(),
            level: 1,
            name: "k8s04021100".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04021200Checker {}

impl Checker for K8S04021200Checker {
    fn execute(&self) -> CheckerResult {
        let allowed_suites: HashSet<&str> = vec![
            "TLS_ECDHE_ECDSA_WITH_AES_128_GCM_SHA256",
            "TLS_ECDHE_RSA_WITH_AES_128_GCM_SHA256",
            "TLS_ECDHE_ECDSA_WITH_CHACHA20_POLY1305",
            "TLS_ECDHE_RSA_WITH_AES_256_GCM_SHA384",
            "TLS_ECDHE_RSA_WITH_CHACHA20_POLY1305",
            "TLS_ECDHE_ECDSA_WITH_AES_256_GCM_SHA384",
            "TLS_RSA_WITH_AES_256_GCM_SHA384",
            "TLS_RSA_WITH_AES_128_GCM_SHA256",
        ]
        .into_iter()
        .collect();

        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "tlsCipherSuites")]
            tls_cipher_suites: Vec<String>,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                let configured_suites: HashSet<&str> = config
                    .tls_cipher_suites
                    .iter()
                    .map(|s| s.as_str())
                    .collect();
                if !configured_suites.is_subset(&allowed_suites) {
                    result.error = "Found disallowed cipher suites".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                result.error = "unable to parse kubelet config".to_string()
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that the Kubelet only makes use of Strong Cryptographic Ciphers"
                .to_string(),
            id: "4.2.12".to_string(),
            level: 1,
            name: "k8s04021200".to_string(),
            mode: Mode::Automatic,
        }
    }
}

// =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<= =>o.o<=

pub struct K8S04021300Checker {}

impl Checker for K8S04021300Checker {
    fn execute(&self) -> CheckerResult {
        #[derive(Deserialize)]
        struct KubeletConfig {
            #[serde(rename = "podPidsLimit")]
            pod_pids_limit: i64,
        }

        let mut result = CheckerResult::default();

        if let Ok(kubelet_file) = File::open(KUBELET_CONF_FILE) {
            if let Ok(config) = serde_yaml::from_reader::<_, KubeletConfig>(kubelet_file) {
                if config.pod_pids_limit <= 0 {
                    result.error = "podPidsLimit is unrestricted".to_string();
                    result.status = CheckStatus::FAIL;
                } else {
                    result.status = CheckStatus::PASS;
                }
            } else {
                // If the setting is not present then there is no pod pid limit (whatever the host allows)
                result.error = "podPidsLimit is not configured".to_string();
                result.status = CheckStatus::FAIL;
            }
        } else {
            result.error = format!("unable to read '{}'", KUBELET_CONF_FILE);
        }

        result
    }

    fn metadata(&self) -> CheckerMetadata {
        CheckerMetadata {
            title: "Ensure that a limit is set on pod PIDs".to_string(),
            id: "4.2.13".to_string(),
            level: 1,
            name: "k8s04021300".to_string(),
            mode: Mode::Automatic,
        }
    }
}
