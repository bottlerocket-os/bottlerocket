//! Stable-supported shim for [`Option::deref`] ([tracking issue][inner_deref]).
//!
//! [inner_deref]: https://github.com/rust-lang/rust/issues/50264

use std::ops::Deref;

pub(crate) trait OptionDeref<T: Deref> {
    fn deref_shim(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionDeref<T> for Option<T> {
    fn deref_shim(&self) -> Option<&T::Target> {
        self.as_ref().map(Deref::deref)
    }
}
