use crate::interface_id::InterfaceName;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub(crate) struct WickedVlanTag {
    #[serde(rename = "$unflatten=device")]
    device: InterfaceName,
    #[serde(rename = "$unflatten=tag")]
    id: u16,
}

impl WickedVlanTag {
    pub(crate) fn new(device: InterfaceName, id: u16) -> Self {
        WickedVlanTag { device, id }
    }
}
