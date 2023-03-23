use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Just need serde's Error in scope to get its trait methods
use super::error;
use serde::de::Error as _;
use serde_json::Value;
use snafu::{ensure, ResultExt};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::net::IpAddr;
use std::ops::Deref;

use crate::SingleLineString;

// Declare constant values usable by any type
const IMAGE_GC_THRESHOLD_MAX: i32 = 100;
const IMAGE_GC_THRESHOLD_MIN: i32 = 0;

/// KubernetesName represents a string that contains a valid Kubernetes resource name.  It stores
/// the original string and makes it accessible through standard traits.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/names/#names
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesName {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_NAME: Regex = Regex::new(r"^[0-9a-z.-]{1,253}$").unwrap();
}

impl TryFrom<&str> for KubernetesName {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_NAME.is_match(input),
            error::PatternSnafu {
                thing: "Kubernetes name",
                pattern: KUBERNETES_NAME.clone(),
                input
            }
        );
        Ok(KubernetesName {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesName, "KubernetesName");

#[cfg(test)]
mod test_kubernetes_name {
    use super::KubernetesName;
    use std::convert::TryFrom;

    #[test]
    fn good_names() {
        for ok in &["howdy", "42", "18-eighteen."] {
            KubernetesName::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_names() {
        for err in &["", "HOWDY", "@", "hi/there", &"a".repeat(254)] {
            KubernetesName::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesLabelKey represents a string that contains a valid Kubernetes label key.  It stores
/// the original string and makes it accessible through standard traits.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#syntax-and-character-set
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesLabelKey {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_LABEL_KEY: Regex = Regex::new(
        r"(?x)^
       (  # optional prefix
           [[:alnum:].-]{1,253}/  # DNS label characters followed by slash
       )?
       [[:alnum:]]  # at least one alphanumeric
       (
           ([[:alnum:]._-]{0,61})?  # more characters allowed in middle
           [[:alnum:]]  # have to end with alphanumeric
       )?
   $"
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesLabelKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_LABEL_KEY.is_match(input),
            error::BigPatternSnafu {
                thing: "Kubernetes label key",
                input
            }
        );
        Ok(KubernetesLabelKey {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesLabelKey, "KubernetesLabelKey");

#[cfg(test)]
mod test_kubernetes_label_key {
    use super::KubernetesLabelKey;
    use std::convert::TryFrom;

    #[test]
    fn good_keys() {
        for ok in &[
            "no-prefix",
            "have.a/prefix",
            "more-chars_here.now",
            &"a".repeat(63),
            &format!("{}/{}", "a".repeat(253), "name"),
        ] {
            KubernetesLabelKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_keys() {
        for err in &[
            ".bad",
            "bad.",
            &"a".repeat(64),
            &format!("{}/{}", "a".repeat(254), "name"),
        ] {
            KubernetesLabelKey::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesLabelValue represents a string that contains a valid Kubernetes label value.  It
/// stores the original string and makes it accessible through standard traits.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#syntax-and-character-set
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesLabelValue {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_LABEL_VALUE: Regex = Regex::new(
        r"(?x)
        ^$ |  # may be empty, or:
        ^
           [[:alnum:]]  # at least one alphanumeric
           (
               ([[:alnum:]._-]{0,61})?  # more characters allowed in middle
               [[:alnum:]]  # have to end with alphanumeric
           )?
        $
   "
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesLabelValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_LABEL_VALUE.is_match(input),
            error::BigPatternSnafu {
                thing: "Kubernetes label value",
                input
            }
        );
        Ok(KubernetesLabelValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesLabelValue, "KubernetesLabelValue");

#[cfg(test)]
mod test_kubernetes_label_value {
    use super::KubernetesLabelValue;
    use std::convert::TryFrom;

    #[test]
    fn good_values() {
        for ok in &["", "more-chars_here.now", &"a".repeat(63)] {
            KubernetesLabelValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &[".bad", "bad.", &"a".repeat(64)] {
            KubernetesLabelValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesTaintValue represents a string that contains a valid Kubernetes taint value, which is
/// like a label value, plus a colon, plus an "effect".  It stores the original string and makes it
/// accessible through standard traits.
///
/// Note: Kubelet won't launch if you specify an effect it doesn't know about, but we don't want to
/// gatekeep all possible values, so be careful.
// Note: couldn't find an exact spec for this.  Cobbling things together, and guessing a bit as to
// the syntax of the effect.
// https://kubernetes.io/docs/concepts/overview/working-with-objects/labels/#syntax-and-character-set
// https://kubernetes.io/docs/concepts/configuration/taint-and-toleration/
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesTaintValue {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_TAINT_VALUE: Regex = Regex::new(
        r"(?x)^
       (
          [[:alnum:]]  # values have to start with alphanumeric if they're specified
          (
             ([[:alnum:]._-]{0,61})?  # more characters allowed in middle
             [[:alnum:]]  # values have to end with alphanumeric
          )?  # only the first alphanumeric is required, further chars optional
       )? # the taint value is optional
       :  # separate the taint value from the effect
       [[:alnum:]]{1,253}  # effect
   $"
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesTaintValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_TAINT_VALUE.is_match(input),
            error::BigPatternSnafu {
                thing: "Kubernetes taint value",
                input
            }
        );
        Ok(KubernetesTaintValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesTaintValue, "KubernetesTaintValue");

#[cfg(test)]
mod test_kubernetes_taint_value {
    use super::KubernetesTaintValue;
    use std::convert::TryFrom;

    #[test]
    fn good_values() {
        // All the examples from the docs linked above
        for ok in &[
            "value:NoSchedule",
            "value:PreferNoSchedule",
            "value:NoExecute",
            ":NoSchedule",
            "a:NoSchedule",
            "a-b:NoSchedule",
        ] {
            KubernetesTaintValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &[
            ".bad",
            "bad.",
            &"a".repeat(254),
            "value:",
            ":",
            "-a:NoSchedule",
            "a-:NoSchedule",
        ] {
            KubernetesTaintValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesClusterName represents a string that contains a valid Kubernetes cluster name.  It
/// stores the original string and makes it accessible through standard traits.
// Note: I was unable to find the rules for cluster naming.  We know they have to fit into label
// values, because of the common cluster-name label, but they also can't be empty.  This combines
// those two characteristics into a new type, until we find an explicit syntax.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesClusterName {
    inner: String,
}

impl TryFrom<&str> for KubernetesClusterName {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            !input.is_empty(),
            error::InvalidClusterNameSnafu {
                name: input,
                msg: "must not be empty"
            }
        );
        ensure!(
            KubernetesLabelValue::try_from(input).is_ok(),
            error::InvalidClusterNameSnafu {
                name: input,
                msg: "cluster names must be valid Kubernetes label values"
            }
        );

        Ok(KubernetesClusterName {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesClusterName, "KubernetesClusterName");

#[cfg(test)]
mod test_kubernetes_cluster_name {
    use super::KubernetesClusterName;
    use std::convert::TryFrom;

    #[test]
    fn good_cluster_names() {
        for ok in &["more-chars_here.now", &"a".repeat(63)] {
            KubernetesClusterName::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &["", ".bad", "bad.", &"a".repeat(64)] {
            KubernetesClusterName::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesAuthenticationMode represents a string that is a valid authentication mode for the
/// kubelet.  It stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesAuthenticationMode {
    inner: String,
}

impl TryFrom<&str> for KubernetesAuthenticationMode {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        ensure!(
            matches!(input, "aws" | "tls"),
            error::InvalidAuthenticationModeSnafu { input }
        );
        Ok(KubernetesAuthenticationMode {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesAuthenticationMode, "KubernetesAuthenticationMode");

#[cfg(test)]
mod test_kubernetes_authentication_mode {
    use super::KubernetesAuthenticationMode;
    use std::convert::TryFrom;

    #[test]
    fn good_modes() {
        for ok in &["aws", "tls"] {
            KubernetesAuthenticationMode::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_modes() {
        for err in &["", "anonymous"] {
            KubernetesAuthenticationMode::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesBootstrapToken represents a string that is a valid bootstrap token for Kubernetes.
/// It stores the original string and makes it accessible through standard traits.
// https://kubernetes.io/docs/reference/access-authn-authz/bootstrap-tokens/
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesBootstrapToken {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_BOOTSTRAP_TOKEN: Regex =
        Regex::new(r"^[a-z0-9]{6}\.[a-z0-9]{16}$").unwrap();
}

impl TryFrom<&str> for KubernetesBootstrapToken {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_BOOTSTRAP_TOKEN.is_match(input),
            error::PatternSnafu {
                thing: "Kubernetes bootstrap token",
                pattern: KUBERNETES_BOOTSTRAP_TOKEN.clone(),
                input
            }
        );
        Ok(KubernetesBootstrapToken {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesBootstrapToken, "KubernetesBootstrapToken");

#[cfg(test)]
mod test_kubernetes_bootstrap_token {
    use super::KubernetesBootstrapToken;
    use std::convert::TryFrom;

    #[test]
    fn good_tokens() {
        for ok in &["abcdef.0123456789abcdef", "07401b.f395accd246ae52d"] {
            KubernetesBootstrapToken::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_names() {
        for err in &["", "ABCDEF.0123456789ABCDEF", "secret", &"a".repeat(23)] {
            KubernetesBootstrapToken::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesEvictionHardKey represents a string that contains a valid Kubernetes eviction hard key.
/// https://kubernetes.io/docs/tasks/administer-cluster/out-of-resource/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesEvictionHardKey {
    inner: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
enum EvictionSignal {
    #[serde(rename = "memory.available")]
    MemoryAvailable,
    #[serde(rename = "nodefs.available")]
    NodefsAvailable,
    #[serde(rename = "nodefs.inodesFree")]
    NodefsInodesFree,
    #[serde(rename = "imagefs.available")]
    ImagefsAvailable,
    #[serde(rename = "imagefs.inodesFree")]
    ImagefsInodesFree,
    #[serde(rename = "pid.available")]
    PidAvailable,
}

impl TryFrom<&str> for KubernetesEvictionHardKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str::<EvictionSignal>(input).context(error::InvalidPlainValueSnafu {
            field: "Eviction Hard key",
        })?;
        Ok(KubernetesEvictionHardKey {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(KubernetesEvictionHardKey, "KubernetesEvictionHardKey");

#[cfg(test)]
mod test_kubernetes_eviction_hard_key {
    use super::KubernetesEvictionHardKey;
    use std::convert::TryFrom;

    #[test]
    fn good_eviction_hard_key() {
        for ok in &[
            "memory.available",
            "nodefs.available",
            "nodefs.inodesFree",
            "imagefs.available",
            "imagefs.inodesFree",
            "pid.available",
        ] {
            KubernetesEvictionHardKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_eviction_hard_key() {
        for err in &["", "storage.available", ".bad", "bad.", &"a".repeat(64)] {
            KubernetesEvictionHardKey::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesThresholdValue represents a string that contains a valid kubernetes threshold value.

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesThresholdValue {
    inner: String,
}

// Regular expression of Kubernetes quantity. i.e. 128974848, 129e6, 129M, 123Mi
lazy_static! {
    pub(crate) static ref KUBERNETES_QUANTITY: Regex = Regex::new(
        r"(?x)
        # format1 for scientific notations (e.g. 123e4) or:
        ^([+-]?[0-9.]+)((e)?[0-9]*)$ |
        # format2 for values with unit suffixes [EPTGMK] and [EiPiTiGiMiKi] (e.g. 100K or 100Ki),
        # or no units (e.g. 100) or:
        ^([+-]?[0-9.]+)((E|P|T|G|M|K)i?)?$ |
        # format3 for values with unit suffixes [numk] (e.g. 100n 1000k)
        ^([+-]?[0-9.]+)(n|u|m|k)?$
   "
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesThresholdValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        if let Some(stripped) = input.strip_suffix('%') {
            let input_f32 = stripped
                .parse::<f32>()
                .context(error::InvalidPercentageSnafu { input })?;
            ensure!(
                (0.0..100.0).contains(&input_f32),
                error::InvalidThresholdPercentageSnafu { input }
            );
        } else {
            ensure!(
                KUBERNETES_QUANTITY.is_match(input),
                error::PatternSnafu {
                    thing: "Kubernetes quantity",
                    pattern: KUBERNETES_QUANTITY.clone(),
                    input
                }
            );
        }
        Ok(KubernetesThresholdValue {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(KubernetesThresholdValue, "KubernetesThresholdValue");

#[cfg(test)]
mod test_kubernetes_threshold_value {
    use super::KubernetesThresholdValue;
    use std::convert::TryFrom;

    #[test]
    fn good_kubernetes_threshold_value() {
        for ok in &[
            "10%", "129e6", "10Mi", "1024M", "1Gi", "120Ki", "1Ti", "1000n", "100m",
        ] {
            KubernetesThresholdValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_kubernetes_threshold_value() {
        for err in &[
            "",
            "anything%",
            "12ki",
            "100e23m",
            "1100KTi",
            "100Kiii",
            "1000i",
            &"a".repeat(64),
        ] {
            KubernetesThresholdValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesReservedResourceKey represents a string that contains a valid Kubernetes kubeReserved
/// and systemReserved resources i.e. cpu, memory.
/// https://kubernetes.io/docs/tasks/administer-cluster/reserve-compute-resources/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesReservedResourceKey {
    inner: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ReservedResources {
    Cpu,
    Memory,
    #[serde(rename = "ephemeral-storage")]
    EphemeralStorage,
}

impl TryFrom<&str> for KubernetesReservedResourceKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str::<ReservedResources>(input).context(
            error::InvalidPlainValueSnafu {
                field: "Reserved sources key",
            },
        )?;
        Ok(KubernetesReservedResourceKey {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(
    KubernetesReservedResourceKey,
    "KubernetesReservedResourceKey"
);

#[cfg(test)]
mod test_reserved_resources_key {
    use super::KubernetesReservedResourceKey;
    use std::convert::TryFrom;

    #[test]
    fn good_reserved_resources_key() {
        for ok in &["cpu", "memory", "ephemeral-storage"] {
            KubernetesReservedResourceKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_reserved_resources_key() {
        for err in &["", "cpa", ".bad", "bad.", &"a".repeat(64)] {
            KubernetesReservedResourceKey::try_from(*err).unwrap_err();
        }
    }
}

/// // =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesQuantityValue represents a string that contains a valid kubernetes quantity value.
/// https://kubernetes.io/docs/concepts/configuration/manage-resources-containers/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesQuantityValue {
    inner: String,
}

impl TryFrom<&str> for KubernetesQuantityValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            KUBERNETES_QUANTITY.is_match(input),
            error::PatternSnafu {
                thing: "Kubernetes quantity",
                pattern: KUBERNETES_QUANTITY.clone(),
                input
            }
        );

        Ok(KubernetesQuantityValue {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(KubernetesQuantityValue, "KubernetesQuantityValue");

#[cfg(test)]
mod test_kubernetes_quantity_value {
    use super::KubernetesQuantityValue;
    use std::convert::TryFrom;

    #[test]
    fn good_kubernetes_quantity_value() {
        for ok in &[
            "129e6", "10Mi", "1024M", "1Gi", "120Ki", "1Ti", "1000n", "100m",
        ] {
            KubernetesQuantityValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_kubernetes_quantity_value() {
        for err in &[
            "",
            "12%",
            "anything%",
            "12ki",
            "100e23m",
            "1100KTi",
            "100Kiii",
            "1000i",
            &"a".repeat(64),
        ] {
            KubernetesQuantityValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesCloudProvider represents a string that is a valid cloud provider for the
/// kubelet.  It stores the original string and makes it accessible through standard traits.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesCloudProvider {
    inner: String,
}

impl TryFrom<&str> for KubernetesCloudProvider {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, error::Error> {
        // Kubelet expects the empty string to be double-quoted when be passed to `--cloud-provider`
        let cloud_provider = if input.is_empty() { "\"\"" } else { input };
        ensure!(
            matches!(cloud_provider, "aws" | "external" | "\"\""),
            error::InvalidCloudProviderSnafu {
                input: cloud_provider
            }
        );
        Ok(KubernetesCloudProvider {
            inner: cloud_provider.to_string(),
        })
    }
}

string_impls_for!(KubernetesCloudProvider, "KubernetesCloudProvider");

#[cfg(test)]
mod test_kubernetes_cloud_provider {
    use super::KubernetesCloudProvider;
    use std::convert::TryFrom;

    #[test]
    fn allowed_providers() {
        for ok in &["aws", "external", "\"\"", ""] {
            KubernetesCloudProvider::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn disallowed_providers() {
        for err in &["internal"] {
            KubernetesCloudProvider::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// CpuManagerPolicy represents a string that contains a valid cpu management policy. Default: none
/// https://kubernetes.io/docs/tasks/administer-cluster/cpu-management-policies/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct CpuManagerPolicy {
    inner: String,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ValidCpuManagerPolicy {
    Static,
    None,
}

impl TryFrom<&str> for CpuManagerPolicy {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str::<ValidCpuManagerPolicy>(input)
            .context(error::InvalidCpuManagerPolicySnafu { input })?;
        Ok(CpuManagerPolicy {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(CpuManagerPolicy, "CpuManagerPolicy");

#[cfg(test)]
mod test_cpu_manager_policy {
    use super::CpuManagerPolicy;
    use std::convert::TryFrom;

    #[test]
    fn good_cpu_manager_policy() {
        for ok in &["static", "none"] {
            CpuManagerPolicy::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_cpu_manager_policy() {
        for err in &["", "bad", "100", &"a".repeat(64)] {
            CpuManagerPolicy::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesDurationValue represents a string that contains a valid Kubernetes duration value.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct KubernetesDurationValue {
    inner: String,
}

lazy_static! {
    pub(crate) static ref KUBERNETES_DURATION_VALUE: Regex = Regex::new(
        r"^(([0-9]+\.)?[0-9]+h)?(([0-9]+\.)?[0-9]+m)?(([0-9]+\.)?[0-9]+s)?(([0-9]+\.)?[0-9]+ms)?$"
    )
    .unwrap();
}

impl TryFrom<&str> for KubernetesDurationValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        ensure!(
            !input.is_empty(),
            error::InvalidKubernetesDurationValueSnafu { input }
        );
        ensure!(
            KUBERNETES_DURATION_VALUE.is_match(input),
            error::InvalidKubernetesDurationValueSnafu { input }
        );
        Ok(KubernetesDurationValue {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesDurationValue, "KubernetesDurationValue");

#[cfg(test)]
mod test_kubernetes_duration_value {
    use super::KubernetesDurationValue;
    use std::convert::TryFrom;

    #[test]
    fn good_tokens() {
        for ok in &[
            "9ms",
            "99s",
            "20m",
            "1h",
            "1h2m3s10ms",
            "4m5s10ms",
            "2h3s10ms",
            "1.5h3.5m",
        ] {
            KubernetesDurationValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_names() {
        for err in &[
            "",
            "100",
            "...3ms",
            "1..5s",
            "ten second",
            "1m2h",
            "9ns",
            &"a".repeat(23),
        ] {
            KubernetesDurationValue::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// TopologyManagerScope represents a string that contains a valid topology management scope. Default: container
/// https://kubernetes.io/docs/tasks/administer-cluster/topology-manager/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TopologyManagerScope {
    inner: String,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ValidTopologyManagerScope {
    Container,
    Pod,
}

impl TryFrom<&str> for TopologyManagerScope {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str::<ValidTopologyManagerScope>(input)
            .context(error::InvalidTopologyManagerScopeSnafu { input })?;
        Ok(TopologyManagerScope {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(TopologyManagerScope, "TopologyManagerScope");

#[cfg(test)]
mod test_topology_manager_scope {
    use super::TopologyManagerScope;
    use std::convert::TryFrom;

    #[test]
    fn good_topology_manager_scope() {
        for ok in &["container", "pod"] {
            TopologyManagerScope::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_topology_manager_scope() {
        for err in &["", "bad", "100", &"a".repeat(64)] {
            TopologyManagerScope::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// TopologyManagerPolicy represents a string that contains a valid topology management policy. Default: none
/// https://kubernetes.io/docs/tasks/administer-cluster/topology-manager/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TopologyManagerPolicy {
    inner: String,
}
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ValidTopologyManagerPolicy {
    None,
    Restricted,
    #[serde(rename = "best-effort")]
    BestEffort,
    #[serde(rename = "single-numa-node")]
    SingleNumaNode,
}

impl TryFrom<&str> for TopologyManagerPolicy {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        serde_plain::from_str::<ValidTopologyManagerPolicy>(input)
            .context(error::InvalidTopologyManagerPolicySnafu { input })?;
        Ok(TopologyManagerPolicy {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(TopologyManagerPolicy, "TopologyManagerPolicy");

#[cfg(test)]
mod test_topology_manager_policy {
    use super::TopologyManagerPolicy;
    use std::convert::TryFrom;

    #[test]
    fn good_topology_manager_policy() {
        for ok in &["none", "restricted", "best-effort", "single-numa-node"] {
            TopologyManagerPolicy::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_topology_manager_policy() {
        for err in &["", "bad", "100", &"a".repeat(64)] {
            TopologyManagerPolicy::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// This enum is used by `IntegerPercent` to "remember" how the number was deserialized.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
enum IntegerPercentMode {
    Number,
    String,
}

/// This type allows for the representation of `imageGCHighThresholdPercent` and
/// `imageGCHighThresholdPercent` as numbers in Bottlerocket userdata and API interactions.
/// See https://github.com/bottlerocket-os/bottlerocket/issues/2883
///
/// The type "remembers" whether it was deserialized from a string or a number and reserializes the
/// same way. This allows for backward compatibility where users may expect these to be strings, but
/// allows for new userdata/API-interactions to represent these as numbers.
///
/// ## About Kubernetes GC Threshold Percent
///
/// `imageGCHighThresholdPercent` and `imageGCHighThresholdPercent` are percentages of disk usage
/// after which image garbage collection is always run. The percent is calculated by dividing by
/// 100, so this field must be between 0 and 100, inclusive. When specified, the value of
/// `imageGCHighThresholdPercent` must be greater than `imageGCHighThresholdPercent`, however this
/// is not enforced by the Bottlerocket API.
/// Default: 85
/// https://kubernetes.io/docs/reference/config-api/kubelet-config.v1beta1/
///
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct IntegerPercent {
    value: i32,
    mode: IntegerPercentMode,
}

impl IntegerPercent {
    fn new(value: i32, mode: IntegerPercentMode) -> Result<Self, error::Error> {
        ensure!(
            (IMAGE_GC_THRESHOLD_MIN..=IMAGE_GC_THRESHOLD_MAX).contains(&value),
            error::InvalidImageGCLowThresholdPercentSnafu {
                input: value.to_string(),
                msg: "must be between 0 and 100 (inclusive)"
            }
        );
        Ok(Self { value, mode })
    }
}

impl Display for IntegerPercent {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.value, f)
    }
}

impl Serialize for IntegerPercent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self.mode {
            IntegerPercentMode::Number => self.value.serialize(serializer),
            IntegerPercentMode::String => {
                let s = self.value.to_string();
                s.serialize(serializer)
            }
        }
    }
}

impl<'de> Deserialize<'de> for IntegerPercent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // We need to deserialize it first into a type that can handle both numbers and strings.
        let json_value = Value::deserialize(deserializer)?;

        // We expect the json_value to be either a string or a number, but either way we need to
        // convert it to a string and parse it because we cannot cast a json number to i32.
        let (s, mode) = match &json_value {
            Value::Number(n) => (n.to_string(), IntegerPercentMode::Number),
            Value::String(s) => (s.clone(), IntegerPercentMode::String),
            _ => {
                return Err(D::Error::custom(format!(
                    "Unable to deserialize value, it is not a number or a string: {:?}",
                    json_value,
                )))
            }
        };

        let value = s
            .parse::<i32>()
            .map_err(|e| D::Error::custom(format!("Unable to parse {} as an integer: {}", s, e)))?;

        // This new function will clamp the range to 0..100 with a nice error message.
        Self::new(value, mode).map_err(|e| D::Error::custom(e.to_string()))
    }
}

#[cfg(test)]
mod test_integer_percent {
    use super::{IntegerPercent, IntegerPercentMode};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use serde_plain::derive_fromstr_from_deserialize;
    use std::fmt::Debug;
    use std::str::FromStr;

    #[derive(Debug, Serialize, Deserialize)]
    struct Object {
        number: IntegerPercent,
    }

    #[test]
    fn valid_string_42() {
        let json_value = json!({"number":"42"});
        let json = serde_json::to_string_pretty(&json_value).unwrap();
        let object: Object = serde_json::from_value(json_value).unwrap();
        assert_eq!(object.number.value, 42);
        assert!(matches!(object.number.mode, IntegerPercentMode::String));
        let serialized = serde_json::to_string_pretty(&object).unwrap();
        assert_eq!(json, serialized);
    }

    #[test]
    fn valid_number_42() {
        let json_value = json!({"number":42});
        let json = serde_json::to_string_pretty(&json_value).unwrap();
        let object: Object = serde_json::from_value(json_value).unwrap();
        assert_eq!(object.number.value, 42);
        assert!(matches!(object.number.mode, IntegerPercentMode::Number));
        let serialized = serde_json::to_string_pretty(&object).unwrap();
        assert_eq!(json, serialized);
    }

    #[test]
    fn invalid_string_not_a_number() {
        let json_value = json!({"number":"foo"});
        assert!(serde_json::from_value::<Object>(json_value).is_err());
    }

    #[test]
    fn invalid_string_out_of_range() {
        let json_value = json!({"number":"99999999"});
        assert!(serde_json::from_value::<Object>(json_value).is_err());
    }

    #[test]
    fn invalid_number_out_of_range() {
        let json_value = json!({"number":99999999});
        assert!(serde_json::from_value::<Object>(json_value).is_err());
    }

    // Adding these impls to preserve legacy tests as they were written.
    derive_fromstr_from_deserialize!(IntegerPercent);
    impl TryFrom<&str> for IntegerPercent {
        type Error = serde_plain::Error;
        fn try_from(value: &str) -> Result<Self, Self::Error> {
            Self::from_str(value)
        }
    }

    // legacy test 1: good values should succeed
    #[test]
    fn image_gc_threshold_percent_between_0_and_100_inclusive() {
        for ok in &["0", "1", "99", "100"] {
            IntegerPercent::try_from(*ok).unwrap();
        }
    }

    // legacy test 2: values too low should return Errors
    #[test]
    fn image_gc_threshold_percent_less_than_0_fails() {
        IntegerPercent::try_from("-1").unwrap_err();
    }

    // legacy test 3: values too high should return Errors
    #[test]
    fn image_gc_threshold_percent_greater_than_100_fails() {
        IntegerPercent::try_from("101").unwrap_err();
    }

    // pseudo-legacy test 4: empty values should return Errors
    #[test]
    fn image_gc_threshold_percent_empty() {
        IntegerPercent::try_from("").unwrap_err();
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// KubernetesClusterDnsIp represents the --cluster-dns settings for kubelet.
///
/// This model allows the value to be either a list of IPs, or a single IP string
/// for backwards compatibility.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KubernetesClusterDnsIp {
    Scalar(IpAddr),
    Vector(Vec<IpAddr>),
}

impl KubernetesClusterDnsIp {
    pub fn iter<'a>(&'a self) -> Box<dyn Iterator<Item = &'a IpAddr> + 'a> {
        match self {
            Self::Scalar(inner) => Box::new(std::iter::once(inner)),
            Self::Vector(inner) => Box::new(inner.iter()),
        }
    }
}

impl IntoIterator for KubernetesClusterDnsIp {
    type Item = IpAddr;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Scalar(inner) => vec![inner],
            Self::Vector(inner) => inner,
        }
        .into_iter()
    }
}

#[cfg(test)]
mod test_cluster_dns_ip {
    use super::KubernetesClusterDnsIp;
    use std::net::IpAddr;
    use std::str::FromStr;

    #[test]
    fn test_parse_cluster_dns_ip_from_str() {
        assert_eq!(
            serde_json::from_str::<KubernetesClusterDnsIp>(r#""127.0.0.1""#).unwrap(),
            KubernetesClusterDnsIp::Scalar(IpAddr::from_str("127.0.0.1").unwrap())
        );
        assert_eq!(
            serde_json::from_str::<KubernetesClusterDnsIp>(r#""::1""#).unwrap(),
            KubernetesClusterDnsIp::Scalar(IpAddr::from_str("::1").unwrap())
        );
    }

    #[test]
    fn test_parse_cluster_dns_ip_from_list() {
        assert_eq!(
            serde_json::from_str::<KubernetesClusterDnsIp>(r#"[]"#).unwrap(),
            KubernetesClusterDnsIp::Vector(vec![])
        );
        assert_eq!(
            serde_json::from_str::<KubernetesClusterDnsIp>(r#"["127.0.0.1", "::1"]"#).unwrap(),
            KubernetesClusterDnsIp::Vector(vec![
                IpAddr::from_str("127.0.0.1").unwrap(),
                IpAddr::from_str("::1").unwrap()
            ])
        );
    }

    #[test]
    fn test_iter_cluster_dns_ips() {
        assert_eq!(
            KubernetesClusterDnsIp::Vector(vec![])
                .iter()
                .collect::<Vec<&IpAddr>>(),
            Vec::<&IpAddr>::new(),
        );

        assert_eq!(
            KubernetesClusterDnsIp::Vector(vec![
                IpAddr::from_str("127.0.0.1").unwrap(),
                IpAddr::from_str("::1").unwrap()
            ])
            .iter()
            .collect::<Vec<&IpAddr>>(),
            vec![
                &IpAddr::from_str("127.0.0.1").unwrap(),
                &IpAddr::from_str("::1").unwrap()
            ]
        );

        assert_eq!(
            KubernetesClusterDnsIp::Scalar(IpAddr::from_str("127.0.0.1").unwrap())
                .iter()
                .collect::<Vec<&IpAddr>>(),
            vec![&IpAddr::from_str("127.0.0.1").unwrap()],
        );
    }

    #[test]
    fn test_first_cluster_dns_ips() {
        assert_eq!(KubernetesClusterDnsIp::Vector(vec![]).iter().next(), None);

        assert_eq!(
            KubernetesClusterDnsIp::Vector(vec![
                IpAddr::from_str("127.0.0.1").unwrap(),
                IpAddr::from_str("::1").unwrap()
            ])
            .iter()
            .next(),
            Some(&IpAddr::from_str("127.0.0.1").unwrap())
        );

        assert_eq!(
            KubernetesClusterDnsIp::Scalar(IpAddr::from_str("127.0.0.1").unwrap())
                .iter()
                .next(),
            Some(&IpAddr::from_str("127.0.0.1").unwrap())
        );
    }
}

type EnvVarMap = HashMap<SingleLineString, SingleLineString>;

/// CredentialProvider contains the settings for a credential provider for use
/// in CredentialProviderConfig.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CredentialProvider {
    enabled: bool,
    image_patterns: Vec<SingleLineString>,
    cache_duration: Option<KubernetesDurationValue>,
    environment: Option<EnvVarMap>,
}
