/*!
whippet is a D-Bus listener that reponds to events on D-Bus.
Currently, it is used to notify netdog of events for the primary interface on the network1 D-Bus service.
*/
use futures::stream::StreamExt;
use snafu::{ensure, ResultExt};
use std::process;
use std::process::Command;
use std::string::String;
use zbus::zvariant::OwnedObjectPath;
use zbus::Connection;

mod networkmanager;

use networkmanager::{LinkState, NetworkManagerProxy};

static NETDOG: &str = "/usr/bin/netdog";

pub(crate) async fn run() -> Result<(), crate::error::Error> {
    // Connect to the system bus
    let conn = Connection::system().await?;

    // Get the manager created from the connection to the system bus
    let manager = NetworkManagerProxy::new(&conn).await?;

    // List the lists on the bus
    let links = manager.list_links().await?;

    // Call netdog primary-interface to get the name of the primary interface
    println!("Calling netdog to get primary interface");
    let primary_interface_name_result = Command::new(NETDOG)
        .arg("primary-interface")
        .output()
        .context(error::NetdogExecutionSnafu)?;
    ensure!(
        primary_interface_name_result.status.success(),
        error::FailedNetdogSnafu {
            stderr: String::from_utf8_lossy(&primary_interface_name_result.stderr)
        }
    );

    let primary_interface_output_str = String::from_utf8(primary_interface_name_result.stdout)
        .context(error::PrimaryInterfaceStringSnafu {})?;
    let primary_interface: String = primary_interface_output_str
        .trim()
        .to_lowercase()
        .trim_matches('"')
        .to_string();
    println!("Primary interface is {}", &primary_interface);

    // Put the path in an option since we might not find it
    let mut path_to_primary: Option<&OwnedObjectPath> = None;

    // Iterate over the links found by list_links()
    for (id, name, path) in links.iter() {
        println!("Link id: {id} Name: {name} Path: {path:?}");
        if name == &primary_interface {
            // Now grab the Properties path from the listing to start watching for changes
            path_to_primary = Some(path);
            let link_status: LinkState =
                serde_json::from_str(manager.describe_link(*id).await?.as_str())
                    .context(error::DbusDescribeLinkSnafu {})?;
            println!(
                "Found {} is {}",
                &primary_interface, link_status.administrative_state
            );
            if link_status.administrative_state == "configured" {
                // call netdog now since its already configured, then the polling for changes can block
                println!(
                    "Calling netdog write-primary-interface-status for {}",
                    link_status.name
                );
                let primary_interface_status_result = Command::new(NETDOG)
                    .arg("write-primary-interface-status")
                    .output()
                    .context(error::NetdogExecutionSnafu)?;
                ensure!(
                    primary_interface_status_result.status.success(),
                    error::FailedNetdogSnafu {
                        stderr: String::from_utf8_lossy(&primary_interface_status_result.stderr)
                    }
                );
            }
        } else {
            println!("DEBUG: found {} but is not {}", name, &primary_interface);
        }
    }

    // If we found the primary device, start listening to events
    if let Some(p) = path_to_primary {
        let links = zbus::fdo::PropertiesProxy::builder(&conn)
            .destination("org.freedesktop.network1")?
            .path(p)?
            .build()
            .await?;
        let mut link_props_changed = links.receive_properties_changed().await?;
        // Build a loop to just wait for events, this would be the core of a real long running system, in theory we could
        // spin up multiple async loops at once to listen on multiple things or respond to OS signals
        while let Some(signal) = link_props_changed.next().await {
            let args = signal.args()?;

            for (name, value) in args.changed_properties().iter() {
                println!(
                    "{}.{} changed to `{:?}`",
                    args.interface_name(),
                    name,
                    value
                );
                // Call netdog write-primary-interface-status so netdog can handle any changes that have occured
                let primary_interface_status_result = Command::new(NETDOG)
                    .arg("write-primary-interface-status")
                    .output()
                    .context(error::NetdogExecutionSnafu)?;
                ensure!(
                    primary_interface_status_result.status.success(),
                    error::FailedNetdogSnafu {
                        stderr: String::from_utf8_lossy(&primary_interface_status_result.stderr)
                    }
                );
            }
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        eprintln!("{}", e);
        process::exit(1);
    }
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::string::FromUtf8Error;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("`netdog` failed: {}", stderr))]
        FailedNetdog { stderr: String },

        #[snafu(display("Failed to run 'netdog': {}", source))]
        NetdogExecution { source: io::Error },

        #[snafu(display("Failed to run parse primary interface: {}", source))]
        PrimaryInterfaceString { source: FromUtf8Error },

        #[snafu(display("D-Bus failure: {}", source))]
        ZbusFailure { source: zbus::Error },

        #[snafu(display("Failed to deserialize describe link output: {}", source))]
        DbusDescribeLink { source: serde_json::Error },
    }

    impl From<zbus::Error> for crate::error::Error {
        fn from(err: zbus::Error) -> Self {
            crate::error::Error::ZbusFailure { source: err }
        }
    }
}
