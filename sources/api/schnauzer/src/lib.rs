#[macro_use]
extern crate log;

pub mod helpers;
pub mod v1;
pub mod v2;

// Hoist the v2 API to the top-level of the library.
#[cfg(feature = "testfakes")]
pub use v2::import::fake::{FakeHelperResolver, FakeImporter, FakeSettingsResolver};

pub use v2::{
    error::RenderError, import, render_template, render_template_file, template,
    BottlerocketTemplateImporter,
};
