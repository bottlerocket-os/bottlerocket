use crate::args::{Arguments, Command, SendBootSuccess, SendHealthPing};
use crate::error::Result;
use crate::host_check::HostCheck;
use crate::main_inner;
use crate::service_check::{ServiceCheck, ServiceHealth};
use httptest::responders::status_code;
use httptest::{matchers::*, Expectation, Server};
use log::LevelFilter;
use std::fs::write;
use std::path::PathBuf;
use tempfile::TempDir;

const OS_RELEASE: &str = r#"PRETTY_NAME=Bottlerocket
VARIANT_ID=myvariant
VERSION_ID=1.2.3
BUILD_ID=abcdef0
"#;

struct MockCheck {}

impl ServiceCheck for MockCheck {
    fn check(&self, service_name: &str) -> Result<ServiceHealth> {
        if service_name.ends_with("failed") {
            Ok(ServiceHealth {
                is_healthy: false,
                exit_code: Some(1),
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
        Ok("".to_string())
    }

    fn configured_time_ms(&self) -> Result<String> {
        Ok("".to_string())
    }

    fn network_ready_time_ms(&self) -> Result<String> {
        Ok("".to_string())
    }

    fn filesystem_ready_time_ms(&self) -> Result<String> {
        Ok("".to_string())
    }
}

// dynamically create a config file where we can set server address, list of services, and send_metrics
fn create_config_file_contents(metrics_url: &str, services: &[&str], send_metrics: bool) -> String {
    let svcs = services
        .iter()
        .map(|&s| format!("\"{}\"", s))
        .collect::<Vec<String>>()
        .join(", ");
    format!(
        r#"
    metrics_url = "{}"
    send_metrics = {}
    service_checks = [{}]
    region = "us-west-2"
    seed = 1234
    version_lock = "v0.1.2"
    ignore_waves = false
    "#,
        metrics_url, send_metrics, svcs
    )
}

// create the config and os-release files in a tempdir and return the tempdir
fn create_test_files(metrics_url: &str, services: &[&str], send_metrics: bool) -> TempDir {
    let t = TempDir::new().unwrap();
    write(
        config_path(&t),
        create_config_file_contents(metrics_url, services, send_metrics),
    )
    .unwrap();
    write(os_release_path(&t), OS_RELEASE).unwrap();
    t
}

// create the path to the config in the tempdir
fn config_path(tempdir: &TempDir) -> PathBuf {
    tempdir
        .path()
        .join("metricdog.toml")
        .to_str()
        .unwrap()
        .into()
}

// create the path to os-release in the tempdir
fn os_release_path(tempdir: &TempDir) -> PathBuf {
    tempdir.path().join("os-release").to_str().unwrap().into()
}

#[test]
fn send_boot_success() {
    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("GET", "/metrics"))
            .respond_with(status_code(200)),
    );
    let metrics_url = server.url_str("/metrics");
    let tempdir = create_test_files(&metrics_url, &["a", "b"], true);
    let args = Arguments {
        config: Some(config_path(&tempdir)),
        log_level: LevelFilter::Off,
        os_release: Some(os_release_path(&tempdir)),
        command: Command::SendBootSuccess(SendBootSuccess {}),
    };
    main_inner(args, Box::new(MockCheck {}), Box::new(MockCheck {})).unwrap();
}

#[test]
/// assert that a request is NOT sent to the server when the user sets `send_metrics` to false
fn opt_out() {
    let server = Server::run();
    // expect the get request zero times
    server.expect(
        Expectation::matching(request::method_path("GET", "/metrics"))
            .times(0)
            .respond_with(status_code(200)),
    );
    let metrics_url = server.url_str("/metrics");
    let tempdir = create_test_files(&metrics_url, &[], false);
    let args = Arguments {
        config: Some(config_path(&tempdir)),
        log_level: LevelFilter::Off,
        os_release: Some(os_release_path(&tempdir)),
        command: Command::SendBootSuccess(SendBootSuccess {}),
    };
    main_inner(args, Box::new(MockCheck {}), Box::new(MockCheck {})).unwrap();
}

#[test]
/// assert that send-boot-success exits without error even when there is no HTTP server
fn send_boot_success_no_server() {
    let metrics_url = "http://localhost:0/metrics";
    let tempdir = create_test_files(metrics_url, &[], true);
    let args = Arguments {
        config: Some(config_path(&tempdir)),
        log_level: LevelFilter::Off,
        os_release: Some(os_release_path(&tempdir)),
        command: Command::SendBootSuccess(SendBootSuccess {}),
    };
    main_inner(args, Box::new(MockCheck {}), Box::new(MockCheck {})).unwrap();
}

#[test]
/// assert that send-boot-success exits without error even if the server sends a 404
fn send_boot_success_404() {
    let server = Server::run();
    server.expect(
        Expectation::matching(request::method_path("GET", "/metrics"))
            .respond_with(status_code(404)),
    );
    let metrics_url = server.url_str("/metrics");
    let tempdir = create_test_files(&metrics_url, &[], true);
    let args = Arguments {
        config: Some(config_path(&tempdir)),
        log_level: LevelFilter::Off,
        os_release: Some(os_release_path(&tempdir)),
        command: Command::SendBootSuccess(SendBootSuccess {}),
    };
    main_inner(args, Box::new(MockCheck {}), Box::new(MockCheck {})).unwrap();
}

#[test]
/// assert that send-health-ping works as expected using a mock `ServiceCheck`
fn send_health_ping() {
    let server = Server::run();
    let matcher = all_of![
        request::method_path("GET", "/metrics"),
        request::query(url_decoded(contains(("is_healthy", "false")))),
        request::query(url_decoded(contains(("failed_services", "afailed:1")))),
    ];
    server.expect(Expectation::matching(matcher).respond_with(status_code(200)));
    let metrics_url = server.url_str("/metrics");
    let tempdir = create_test_files(&metrics_url, &["afailed", "b"], true);
    let args = Arguments {
        config: Some(config_path(&tempdir)),
        log_level: LevelFilter::Off,
        os_release: Some(os_release_path(&tempdir)),
        command: Command::SendHealthPing(SendHealthPing {}),
    };
    main_inner(args, Box::new(MockCheck {}), Box::new(MockCheck {})).unwrap();
}
