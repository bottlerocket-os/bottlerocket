// This is linked into place at variant/mod.rs because the build system mounts a temporary
// directory at variant/ - see README.md.
mod current;
pub use current::*;
