use crate::config::Config;
use crate::error::Result;
use crate::host_check::HostCheck;
use crate::metricdog::Metricdog;
use crate::service_check::{ServiceCheck, ServiceHealth};
use bottlerocket_release::BottlerocketRelease;
use httptest::{matchers::*, responders::*, Expectation, Server};
use tempfile::TempDir;

const OS_RELEASE: &str = r#"NAME=Bottlerocket
ID=bottlerocket
PRETTY_NAME="Bottlerocket OS 0.4.0"
VARIANT_ID=aws-k8s-1.16
VERSION_ID=0.4.0
BUILD_ID=7303622
"#;

fn os_release() -> BottlerocketRelease {
    let td = TempDir::new().unwrap();
    let path = td.path().join("os-release");
    std::fs::write(&path, OS_RELEASE).unwrap();
    BottlerocketRelease::from_file(&path).unwrap()
}

struct MockCheck {}

impl ServiceCheck for MockCheck {
    fn check(&self, service_name: &str) -> Result<ServiceHealth> {
        if service_name.ends_with("fail1") {
            Ok(ServiceHealth {
                is_healthy: false,
                exit_code: Some(1),
            })
        } else if service_name.ends_with("fail2") {
            Ok(ServiceHealth {
                is_healthy: false,
                exit_code: Some(2),
            })
        } else {
            Ok(ServiceHealth {
                is_healthy: true,
                exit_code: None,
            })
        }
    }
}

impl HostCheck for MockCheck {
    fn is_first_boot(&self) -> Result<bool> {
        Ok(true)
    }

    fn preconfigured_time_ms(&self) -> Result<String> {
        Ok("123".to_string())
    }

    fn configured_time_ms(&self) -> Result<String> {
        Ok("456".to_string())
    }

    fn network_ready_time_ms(&self) -> Result<String> {
        Ok("789".to_string())
    }

    fn filesystem_ready_time_ms(&self) -> Result<String> {
        Ok("321".to_string())
    }
}

#[test]
fn send_healthy_ping() {
    let server = Server::run();
    let matcher = all_of![
        request::method_path("GET", "/metrics"),
        request::query(url_decoded(contains(("sender", "metricdog")))),
        request::query(url_decoded(contains(("event", "health_ping")))),
        request::query(url_decoded(contains(("version", "0.4.0")))),
        request::query(url_decoded(contains(("variant", "aws-k8s-1.16")))),
        request::query(url_decoded(contains(("arch", std::env::consts::ARCH)))),
        request::query(url_decoded(contains(("region", "us-east-1")))),
        request::query(url_decoded(contains(("seed", "2041")))),
        request::query(url_decoded(contains(("failed_services", "")))),
        request::query(url_decoded(contains(("is_healthy", "true")))),
    ];
    server.expect(Expectation::matching(matcher).respond_with(status_code(200)));
    let metrics_url = server.url_str("/metrics");
    let metricdog = Metricdog::from_parts(
        Config {
            metrics_url,
            send_metrics: true,
            service_checks: vec![
                String::from("service_a"),
                String::from("service_b"),
                String::from("service_c"),
            ],
            region: String::from("us-east-1"),
            seed: 2041,
            version_lock: String::from("latest"),
            ignore_waves: false,
        },
        os_release(),
        Box::new(MockCheck {}),
        Box::new(MockCheck {}),
    )
    .unwrap();
    metricdog.send_health_ping().unwrap();
}

#[test]
fn send_unhealthy_ping() {
    let server = Server::run();
    let matcher = all_of![
        request::method_path("GET", "/metrics"),
        request::query(url_decoded(contains(("sender", "metricdog")))),
        request::query(url_decoded(contains(("event", "health_ping")))),
        request::query(url_decoded(contains(("version", "0.4.0")))),
        request::query(url_decoded(contains(("variant", "aws-k8s-1.16")))),
        request::query(url_decoded(contains(("arch", std::env::consts::ARCH)))),
        request::query(url_decoded(contains(("region", "us-east-1")))),
        request::query(url_decoded(contains(("seed", "2041")))),
        request::query(url_decoded(contains((
            "failed_services",
            "service_afail2:2,service_cfail1:1"
        )))),
        request::query(url_decoded(contains(("is_healthy", "false")))),
    ];
    server.expect(Expectation::matching(matcher).respond_with(status_code(200)));
    let metrics_url = server.url_str("/metrics");
    let metricdog = Metricdog::from_parts(
        Config {
            metrics_url,
            send_metrics: true,
            // note that these are out-of-order sort order to ensure that failed services are sorted
            // in the url.
            service_checks: vec![
                String::from("service_cfail1"),
                String::from("service_afail2"),
                String::from("service_b"),
            ],
            region: String::from("us-east-1"),
            seed: 2041,
            version_lock: String::from("latest"),
            ignore_waves: false,
        },
        os_release(),
        Box::new(MockCheck {}),
        Box::new(MockCheck {}),
    )
    .unwrap();
    metricdog.send_health_ping().unwrap();
}

#[test]
fn send_boot_success() {
    let server = Server::run();
    let matcher = all_of![
        request::method_path("GET", "/metrics"),
        request::query(url_decoded(contains(("sender", "metricdog")))),
        request::query(url_decoded(contains(("event", "boot_success")))),
        request::query(url_decoded(contains(("version", "0.4.0")))),
        request::query(url_decoded(contains(("variant", "aws-k8s-1.16")))),
        request::query(url_decoded(contains(("arch", std::env::consts::ARCH)))),
        request::query(url_decoded(contains(("region", "us-east-1")))),
        request::query(url_decoded(contains(("seed", "2041")))),
        request::query(url_decoded(contains(("is_first_boot", "true")))),
        request::query(url_decoded(contains(("preconfigured_time_ms", "123")))),
        request::query(url_decoded(contains(("configured_time_ms", "456")))),
        request::query(url_decoded(contains(("network_ready_time_ms", "789")))),
        request::query(url_decoded(contains(("filesystem_ready_time_ms", "321")))),
    ];
    server.expect(Expectation::matching(matcher).respond_with(status_code(200)));
    let metrics_url = server.url_str("/metrics");
    let metricdog = Metricdog::from_parts(
        Config {
            metrics_url,
            send_metrics: true,
            service_checks: vec![
                String::from("service_afail2"),
                String::from("service_b"),
                String::from("service_cfail1"),
            ],
            region: String::from("us-east-1"),
            seed: 2041,
            version_lock: String::from("latest"),
            ignore_waves: false,
        },
        os_release(),
        Box::new(MockCheck {}),
        Box::new(MockCheck {}),
    )
    .unwrap();
    metricdog.send_boot_success().unwrap();
}
