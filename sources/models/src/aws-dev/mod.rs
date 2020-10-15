use model_derive::model;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::modeled_types::Identifier;
use crate::{AwsSettings, ContainerImage, KernelSettings, NtpSettings, UpdatesSettings};

// Note: we have to use 'rename' here because the top-level Settings structure is the only one
// that uses its name in serialization; internal structures use the field name that points to it
#[model(rename = "settings", impl_default = true)]
struct Settings {
    motd: String,
    updates: UpdatesSettings,
    host_containers: HashMap<Identifier, ContainerImage>,
    ntp: NtpSettings,
    kernel: KernelSettings,
    aws: AwsSettings,
}
