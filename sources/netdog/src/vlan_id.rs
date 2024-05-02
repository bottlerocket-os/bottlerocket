//! The vlan_id module contains the definition of a valid VLAN ID, and the code to support
//! deserialization of the structure.  A valid VLAN ID must fall between the range of 0-4094.
use serde::de::Error;
use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub(crate) struct VlanId {
    inner: u16,
}

impl<'de> Deserialize<'de> for VlanId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id: u16 = Deserialize::deserialize(deserializer)?;

        if id > 4094 {
            return Err(D::Error::custom(format!(
                "invalid vlan ID '{}': must be between 0-4094",
                id
            )));
        }

        Ok(VlanId { inner: id })
    }
}

impl Deref for VlanId {
    type Target = u16;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Display for VlanId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}
