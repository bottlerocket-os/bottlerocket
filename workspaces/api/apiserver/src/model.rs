//! The model module is the schema for the data store.
//!
//! The datastore::serialization and datastore::deserialization modules make it easy to map between
//! Rust types and the datastore, and thus, all inputs and outputs are type-checked.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;

use crate::modeled_types::ValidBase64;

///// Primary user-visible settings

// Note: fields are marked with skip_serializing_if=Option::is_none so that settings GETs don't
// show field=null for everything that isn't set in the relevant group of settings.

// Kubernetes related settings. The dynamic settings are retrieved from
// IMDS via Sundog's child "Pluto".
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct KubernetesSettings {
    // Settings we require the user to specify, likely via user data.

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_certificate: Option<ValidBase64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_server: Option<String>,

    // Dynamic settings.

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cluster_dns_ip: Option<Ipv4Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_ip: Option<Ipv4Addr>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub pod_infra_container_image: Option<String>,
}

// Updog settings. Taken from userdata.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct UpdatesSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_base_url: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_base_url: Option<String>,
}

// Note: we have to use 'rename' here because the top-level Settings structure is the only one
// that uses its name in serialization; internal structures use the field name that poitns to it
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "settings", rename_all = "kebab-case")]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub hostname: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub kubernetes: Option<KubernetesSettings>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates: Option<UpdatesSettings>,
}

///// Internal services

// Note: Top-level objects that get returned from the API should have a serde "rename" attribute
// matching the struct name, but in kebab-case, e.g. ConfigurationFiles -> "configuration-files".
// This lets it match the datastore name.
// Objects that live inside those top-level objects, e.g. Service lives in Services, should have
// rename="" so they don't add an extra prefix to the datastore path that doesn't actually exist.
// This is important because we have APIs that can return those sub-structures directly.

pub type Services = HashMap<String, Service>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "", rename_all = "kebab-case")]
pub struct Service {
    pub configuration_files: Vec<String>,
    pub restart_commands: Vec<String>,
}

pub type ConfigurationFiles = HashMap<String, ConfigurationFile>;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "", rename_all = "kebab-case")]
pub struct ConfigurationFile {
    pub path: String,
    pub template_path: String,
}

///// Metadata

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename = "metadata", rename_all = "kebab-case")]
pub struct Metadata {
    pub key: String,
    pub md: String,
    pub val: toml::Value,
}
