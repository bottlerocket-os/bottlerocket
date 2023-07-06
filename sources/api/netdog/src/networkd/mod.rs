mod config;
mod devices;

use self::config::{NetDevConfig, NetworkConfig};
use crate::interface_id::InterfaceName;
use std::collections::HashMap;

// A map of network device -> associated VLANs.  This type exists to assist in generating a
// device's network configuration, which must contain it's associated VLANs.
pub(self) type Vlans = HashMap<InterfaceName, Vec<InterfaceName>>;

/// Devices implement this trait if they require a .netdev file
trait NetDevFileCreator {
    fn create_netdev(&self) -> NetDevConfig;
}

/// Devices implement this trait if they require one or more .network files (bonds, for example,
/// create multiple .network files for the bond and it's workers)
trait NetworkFileCreator {
    fn create_networks(&self, vlans: &Vlans) -> Vec<NetworkConfig>;
}

mod error {
    use snafu::Snafu;
    use std::io;
    use std::path::PathBuf;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(crate)))]
    pub(crate) enum Error {
        #[snafu(display("Unable to create '{}', missing name or MAC", what))]
        ConfigMissingName { what: String },

        #[snafu(display("Unable to write {} to {}: {}", what, path.display(), source))]
        NetworkDConfigWrite {
            what: String,
            path: PathBuf,
            source: io::Error,
        },
    }
}
pub(crate) type Result<T> = std::result::Result<T, error::Error>;
