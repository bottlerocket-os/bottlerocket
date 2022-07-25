use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Just need serde's Error in scope to get its trait methods
use super::error;
use serde::de::Error as _;
use snafu::{ensure, ResultExt};
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::net::IpAddr;
use std::ops::Deref;
use std::str::FromStr;

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

impl FromStr for KubernetesName {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_names() {
        for ok in &["howdy", "42", "18-eighteen."] {
            ok.parse::<KubernetesName>().unwrap();
        }
    }

    #[test]
    fn bad_names() {
        for err in &["", "HOWDY", "@", "hi/there", &"a".repeat(254)] {
            err.parse::<KubernetesName>().unwrap_err();
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

impl FromStr for KubernetesLabelKey {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_keys() {
        for ok in &[
            "no-prefix",
            "have.a/prefix",
            "more-chars_here.now",
            &"a".repeat(63),
            &format!("{}/{}", "a".repeat(253), "name"),
        ] {
            ok.parse::<KubernetesLabelKey>().unwrap();
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
            err.parse::<KubernetesLabelKey>().unwrap_err();
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

impl FromStr for KubernetesLabelValue {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_values() {
        for ok in &["", "more-chars_here.now", &"a".repeat(63)] {
            ok.parse::<KubernetesLabelValue>().unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &[".bad", "bad.", &"a".repeat(64)] {
            err.parse::<KubernetesLabelValue>().unwrap_err();
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

impl FromStr for KubernetesTaintValue {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
            ok.parse::<KubernetesTaintValue>().unwrap();
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
            err.parse::<KubernetesTaintValue>().unwrap_err();
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

impl FromStr for KubernetesClusterName {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        ensure!(
            !input.is_empty(),
            error::InvalidClusterNameSnafu {
                name: input,
                msg: "must not be empty"
            }
        );
        ensure!(
            input.parse::<KubernetesLabelValue>().is_ok(),
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

    #[test]
    fn good_cluster_names() {
        for ok in &["more-chars_here.now", &"a".repeat(63)] {
            ok.parse::<KubernetesClusterName>().unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &["", ".bad", "bad.", &"a".repeat(64)] {
            err.parse::<KubernetesClusterName>().unwrap_err();
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

impl FromStr for KubernetesAuthenticationMode {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_modes() {
        for ok in &["aws", "tls"] {
            ok.parse::<KubernetesAuthenticationMode>().unwrap();
        }
    }

    #[test]
    fn bad_modes() {
        for err in &["", "anonymous"] {
            err.parse::<KubernetesAuthenticationMode>().unwrap_err();
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

impl FromStr for KubernetesBootstrapToken {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_tokens() {
        for ok in &["abcdef.0123456789abcdef", "07401b.f395accd246ae52d"] {
            ok.parse::<KubernetesBootstrapToken>().unwrap();
        }
    }

    #[test]
    fn bad_names() {
        for err in &["", "ABCDEF.0123456789ABCDEF", "secret", &"a".repeat(23)] {
            err.parse::<KubernetesBootstrapToken>().unwrap_err();
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

impl FromStr for KubernetesEvictionHardKey {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
            ok.parse::<KubernetesEvictionHardKey>().unwrap();
        }
    }

    #[test]
    fn bad_eviction_hard_key() {
        for err in &["", "storage.available", ".bad", "bad.", &"a".repeat(64)] {
            err.parse::<KubernetesEvictionHardKey>().unwrap_err();
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

impl FromStr for KubernetesThresholdValue {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.ends_with("%") {
            let input_f32 = input[..input.len() - 1]
                .parse::<f32>()
                .context(error::InvalidPercentageSnafu { input })?;
            ensure!(
                (0.0..100.0).contains(&input_f32),
                error::InvalidThresholdPercentageSnafu { input }
            )
        } else {
            ensure!(
                KUBERNETES_QUANTITY.is_match(input),
                error::PatternSnafu {
                    thing: "Kubernetes quantity",
                    pattern: KUBERNETES_QUANTITY.clone(),
                    input
                }
            );
        };

        Ok(KubernetesThresholdValue {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(KubernetesThresholdValue, "KubernetesThresholdValue");

#[cfg(test)]
mod test_kubernetes_threshold_value {
    use super::KubernetesThresholdValue;

    #[test]
    fn good_kubernetes_threshold_value() {
        for ok in &[
            "10%", "129e6", "10Mi", "1024M", "1Gi", "120Ki", "1Ti", "1000n", "100m",
        ] {
            ok.parse::<KubernetesThresholdValue>().unwrap();
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
            err.parse::<KubernetesThresholdValue>().unwrap_err();
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

impl FromStr for KubernetesReservedResourceKey {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_reserved_resources_key() {
        for ok in &["cpu", "memory", "ephemeral-storage"] {
            ok.parse::<KubernetesReservedResourceKey>().unwrap();
        }
    }

    #[test]
    fn bad_reserved_resources_key() {
        for err in &["", "cpa", ".bad", "bad.", &"a".repeat(64)] {
            err.parse::<KubernetesReservedResourceKey>().unwrap_err();
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

impl FromStr for KubernetesQuantityValue {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_kubernetes_quantity_value() {
        for ok in &[
            "129e6", "10Mi", "1024M", "1Gi", "120Ki", "1Ti", "1000n", "100m",
        ] {
            ok.parse::<KubernetesQuantityValue>().unwrap();
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
            err.parse::<KubernetesQuantityValue>().unwrap_err();
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

impl FromStr for KubernetesCloudProvider {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn allowed_providers() {
        for ok in &["aws", "external", "\"\"", ""] {
            ok.parse::<KubernetesCloudProvider>().unwrap();
        }
    }

    #[test]
    fn disallowed_providers() {
        for err in &["internal"] {
            err.parse::<KubernetesCloudProvider>().unwrap_err();
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

impl FromStr for CpuManagerPolicy {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_cpu_manager_policy() {
        for ok in &["static", "none"] {
            ok.parse::<CpuManagerPolicy>().unwrap();
        }
    }

    #[test]
    fn bad_cpu_manager_policy() {
        for err in &["", "bad", "100", &"a".repeat(64)] {
            err.parse::<CpuManagerPolicy>().unwrap_err();
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

impl FromStr for KubernetesDurationValue {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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
            ok.parse::<KubernetesDurationValue>().unwrap();
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
            err.parse::<KubernetesDurationValue>().unwrap_err();
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

impl FromStr for TopologyManagerScope {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_topology_manager_scope() {
        for ok in &["container", "pod"] {
            ok.parse::<TopologyManagerScope>().unwrap();
        }
    }

    #[test]
    fn bad_topology_manager_scope() {
        for err in &["", "bad", "100", &"a".repeat(64)] {
            err.parse::<TopologyManagerScope>().unwrap_err();
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

impl FromStr for TopologyManagerPolicy {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
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

    #[test]
    fn good_topology_manager_policy() {
        for ok in &["none", "restricted", "best-effort", "single-numa-node"] {
            ok.parse::<TopologyManagerPolicy>().unwrap();
        }
    }

    #[test]
    fn bad_topology_manager_policy() {
        for err in &["", "bad", "100", &"a".repeat(64)] {
            err.parse::<TopologyManagerPolicy>().unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// imageGCHighThresholdPercent is the percent of disk usage after which image
/// garbage collection is always run. The percent is calculated by dividing this
/// field value by 100, so this field must be between 0 and 100, inclusive. When
/// specified, the value must be greater than imageGCLowThresholdPercent.
/// Default: 85
/// https://kubernetes.io/docs/reference/config-api/kubelet-config.v1beta1/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ImageGCHighThresholdPercent {
    inner: String,
}

impl FromStr for ImageGCHighThresholdPercent {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed_input: i32 = input
            .parse::<i32>()
            .context(error::ParseIntSnafu { input })?;
        ensure!(
            !input.is_empty(),
            error::InvalidImageGCHighThresholdPercentSnafu {
                input,
                msg: "must not be empty",
            }
        );
        ensure!(
            (IMAGE_GC_THRESHOLD_MIN..=IMAGE_GC_THRESHOLD_MAX).contains(&parsed_input),
            error::InvalidImageGCHighThresholdPercentSnafu {
                input,
                msg: "must be between 0 and 100 (inclusive)"
            }
        );

        Ok(ImageGCHighThresholdPercent {
            inner: input.to_owned(),
        })
    }
}
string_impls_for!(ImageGCHighThresholdPercent, "ImageGCHighThresholdPercent");

#[cfg(test)]
mod test_image_gc_high_threshold_percent {
    use super::ImageGCHighThresholdPercent;

    // test 1: good values should succeed
    #[test]
    fn image_gc_high_threshold_percent_between_0_and_100_inclusive() {
        for ok in &["0", "1", "99", "100"] {
            ok.parse::<ImageGCHighThresholdPercent>().unwrap();
        }
    }

    // test 2: values too low should return Errors
    #[test]
    fn image_gc_high_threshold_percent_less_than_0_fails() {
        ("-1").parse::<ImageGCHighThresholdPercent>().unwrap_err();
    }

    // test 3: values too high should return Errors
    #[test]
    fn image_gc_high_threshold_percent_greater_than_100_fails() {
        ("101").parse::<ImageGCHighThresholdPercent>().unwrap_err();
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// imageGCLowThresholdPercent is the percent of disk usage before which image
/// garbage collection is never run. Lowest disk usage to garbage collect to.
/// The percent is calculated by dividing this field value by 100, so the field
/// value must be between 0 and 100, inclusive. When specified, the value must
/// be less than imageGCHighThresholdPercent.
/// Default: 80
/// https://kubernetes.io/docs/reference/config-api/kubelet-config.v1beta1/

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ImageGCLowThresholdPercent {
    inner: String,
}

impl FromStr for ImageGCLowThresholdPercent {
    type Err = error::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let parsed_input: i32 = input
            .parse::<i32>()
            .context(error::ParseIntSnafu { input })?;
        ensure!(
            !input.is_empty(),
            error::InvalidImageGCLowThresholdPercentSnafu {
                input,
                msg: "must not be empty",
            }
        );
        ensure!(
            (IMAGE_GC_THRESHOLD_MIN..=IMAGE_GC_THRESHOLD_MAX).contains(&parsed_input),
            error::InvalidImageGCLowThresholdPercentSnafu {
                input,
                msg: "must be between 0 and 100 (inclusive)"
            }
        );

        Ok(ImageGCLowThresholdPercent {
            inner: input.to_owned(),
        })
    }
}
string_impls_for!(ImageGCLowThresholdPercent, "ImageGCLowThresholdPercent");

#[cfg(test)]
mod test_image_gc_low_threshold_percent {
    use super::ImageGCLowThresholdPercent;

    // test 1: good values should succeed
    #[test]
    fn image_gc_low_threshold_percent_between_0_and_100_inclusive() {
        for ok in &["0", "1", "99", "100"] {
            ok.parse::<ImageGCLowThresholdPercent>().unwrap();
        }
    }

    // test 2: values too low should return Errors
    #[test]
    fn image_gc_low_threshold_percent_less_than_0_fails() {
        ("-1").parse::<ImageGCLowThresholdPercent>().unwrap_err();
    }

    // test 3: values too high should return Errors
    #[test]
    fn image_gc_low_threshold_percent_greater_than_100_fails() {
        ("101").parse::<ImageGCLowThresholdPercent>().unwrap_err();
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

    pub fn into_iter(self) -> impl Iterator<Item = IpAddr> {
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
