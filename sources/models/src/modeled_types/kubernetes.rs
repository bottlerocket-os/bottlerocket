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
use std::ops::Deref;

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
            error::Pattern {
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
            error::BigPattern {
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
            error::BigPattern {
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
            error::BigPattern {
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
            error::InvalidClusterName {
                name: input,
                msg: "must not be empty"
            }
        );
        ensure!(
            KubernetesLabelValue::try_from(input).is_ok(),
            error::InvalidClusterName {
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
    fn bad_alues() {
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
            error::InvalidAuthenticationMode { input }
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
            error::Pattern {
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
        serde_plain::from_str::<EvictionSignal>(&input).context(error::InvalidPlainValue {
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
        if input.ends_with("%") {
            let input_f32 = input[..input.len() - 1]
                .parse::<f32>()
                .context(error::InvalidPercentage { input })?;
            ensure!(
                (0.0..100.0).contains(&input_f32),
                error::InvalidThresholdPercentage { input }
            )
        } else {
            ensure!(
                KUBERNETES_QUANTITY.is_match(input),
                error::Pattern {
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
        serde_plain::from_str::<ReservedResources>(&input).context(error::InvalidPlainValue {
            field: "Reserved sources key",
        })?;
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
            error::Pattern {
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
        ensure!(
            matches!(input, "aws" | "external"),
            error::InvalidAuthenticationMode { input }
        );
        Ok(KubernetesCloudProvider {
            inner: input.to_string(),
        })
    }
}

string_impls_for!(KubernetesCloudProvider, "KubernetesCloudProvider");

#[cfg(test)]
mod test_kubernetes_cloud_provider {
    use super::KubernetesCloudProvider;
    use std::convert::TryFrom;

    #[test]
    fn good_modes() {
        for ok in &["aws", "external"] {
            KubernetesCloudProvider::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_modes() {
        for err in &["", "internal"] {
            KubernetesCloudProvider::try_from(*err).unwrap_err();
        }
    }
}
