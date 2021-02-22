use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
// Just need serde's Error in scope to get its trait methods
use serde::de::Error as _;
use snafu::ensure;
use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::ops::Deref;
use super::error;

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
       [[:alnum:]]  # at least one alphanumeric
       (
           ([[:alnum:]._-]{0,61})?  # more characters allowed in middle
           [[:alnum:]]  # have to end with alphanumeric
       )?
       :  # separate the label value from the effect
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
        ] {
            KubernetesTaintValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_values() {
        for err in &[".bad", "bad.", &"a".repeat(254), "value:", ":effect"] {
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
            matches!(input, "aws" | "tls" ),
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
    pub(crate) static ref KUBERNETES_BOOTSTRAP_TOKEN: Regex = Regex::new(
        r"^[a-z0-9]{6}\.[a-z0-9]{16}$").unwrap();
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

/// EvictionHardKey represents a string that contains a valid Kubernetes eviction hard
/// signal. There are few valid eviction hard signals [memory.available], [nodefs.available],
/// [imagefs.available], and [nodefs.inodesFree].
/// https://kubernetes.io/docs/tasks/administer-cluster/out-of-resource/


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EvictionHardKey {
    inner: String,
}

impl TryFrom<&str> for EvictionHardKey {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {
        let evitionsignal = vec![
            "memory.available","nodefs.available",
            "nodefs.inodesFree","imagefs.available",
            "imagefs.inodesFree","pid.available"
            ];

        ensure!(
            evitionsignal.contains(&input),
            error::InvalideEvictionHard {
                input,
                msg: format!("must be one of designated signals"),
            }
        );

        Ok(EvictionHardKey {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(EvictionHardKey, "EvictionHardKey");

#[cfg(test)]
mod test_kubernetes_eviction_hard_key {
    use super::EvictionHardKey;
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
            EvictionHardKey::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_eviction_hard_key() {
        for err in &["", "storage.available", ".bad", "bad.", &"a".repeat(64)] {
            EvictionHardKey::try_from(*err).unwrap_err();
        }
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// EvictionHardValue represents a string that contains a valid Kubernetes eviction threshold quantity
/// An eviction threshold can be expressed as Gi/Mi or a percentage using the % token
/// https://kubernetes.io/docs/tasks/administer-cluster/out-of-resource/


#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct EvictionHardValue {
    inner: String,
}

impl TryFrom<&str> for EvictionHardValue {
    type Error = error::Error;

    fn try_from(input: &str) -> Result<Self, Self::Error> {

        ensure!(
            input.ends_with("Gi") || input.ends_with("Mi") || input.ends_with("%"),
            error::InvalideEvictionHard {
                input,
                msg: format!("must be ends with Gi, Mi, or %"),
            }
        );

        Ok(EvictionHardValue {
            inner: input.to_string(),
        })
    }
}
string_impls_for!(EvictionHardValue, "EvictionHardValue");

#[cfg(test)]
mod test_kubernetes_eviction_hard_value {
    use super::EvictionHardValue;
    use std::convert::TryFrom;

    #[test]
    fn good_eviction_hard_value() {
        for ok in &["10Gi", "500Mi", "30%"] {
            EvictionHardValue::try_from(*ok).unwrap();
        }
    }

    #[test]
    fn bad_eviction_hard_value() {
        for err in &["", "bad", "100", &"a".repeat(64)] {
            EvictionHardValue::try_from(*err).unwrap_err();
        }
    }
}