use crate::schema::decoded::{Decoded, Hex};
use crate::schema::key::Key;
use std::collections::HashMap;

/// The iterator produced by `Root::keys`.
//
// This is its own struct because I couldn't find a way to convince rustc that the lifetimes were
// fine otherwise.
pub(super) struct KeysIter<'a> {
    /// The key IDs permitted to sign a role.
    pub(super) keyids_iter: std::slice::Iter<'a, Decoded<Hex>>,
    /// The `keys` field of `Root`, so that we can look up the `Key` by its key ID.
    pub(super) keys: &'a HashMap<Decoded<Hex>, Key>,
}

impl<'a> Iterator for KeysIter<'a> {
    type Item = &'a Key;

    fn next(&mut self) -> Option<Self::Item> {
        self.keyids_iter
            .next()
            .and_then(|keyid| self.keys.get(keyid))
    }
}
