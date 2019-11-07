use super::error::{self, Result};
use super::{Role, Root, Signed};
use olpc_cjson::CanonicalFormatter;
use serde::Serialize;
use snafu::{ensure, OptionExt, ResultExt};

impl Root {
    pub fn verify_role<T: Role + Serialize>(&self, role: &Signed<T>) -> Result<()> {
        let role_keys = self
            .roles
            .get(&T::TYPE)
            .context(error::MissingRole { role: T::TYPE })?;
        let mut valid = 0;

        let mut data = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut data, CanonicalFormatter::new());
        role.signed
            .serialize(&mut ser)
            .context(error::JsonSerialization {
                what: format!("{} role", T::TYPE),
            })?;

        for signature in &role.signatures {
            if role_keys.keyids.contains(&signature.keyid) {
                if let Some(key) = self.keys.get(&signature.keyid) {
                    if key.verify(&data, &signature.sig) {
                        valid += 1;
                    }
                }
            }
        }

        ensure!(
            valid >= u64::from(role_keys.threshold),
            error::SignatureThreshold {
                role: T::TYPE,
                threshold: role_keys.threshold,
                valid,
            }
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Root, Signed};

    #[test]
    fn simple_rsa() {
        let root: Signed<Root> =
            serde_json::from_str(include_str!("../../tests/data/simple-rsa/root.json")).unwrap();
        root.signed.verify_role(&root).unwrap();
    }

    #[test]
    fn no_root_json_signatures_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/no-root-json-signatures/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("missing signature should not verify");
    }

    #[test]
    fn invalid_root_json_signatures_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/invalid-root-json-signature/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("invalid (unauthentic) root signature should not verify");
    }

    #[test]
    // FIXME: this is not actually testing for expired metadata!
    // These tests should be transformed into full repositories and go through Repository::load
    #[ignore]
    fn expired_root_json_signature_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/expired-root-json-signature/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("expired root signature should not verify");
    }

    #[test]
    fn mismatched_root_json_keyids_is_err() {
        let root: Signed<Root> = serde_json::from_str(include_str!(
            "../../tests/data/mismatched-root-json-keyids/root.json"
        ))
        .expect("should be parsable root.json");
        root.signed
            .verify_role(&root)
            .expect_err("mismatched root role keyids (provided and signed) should not verify");
    }
}
