# schnauzer

Current version: 0.1.0

schnauzer serves two primary purposes:
* To provide a library for rendering configuration file templates for Bottlerocket
* To provide a settings generator binary for settings values which are simple computations on other settings.

The schnauzer library can be used to render file- or string-based templates that contain
settings references, e.g. "foo-{{ settings.bar }}", or additional rendering functionality ("helpers").
The settings and helpers used by templates are defined in any settings extensions installed on the system.

(The name "schnauzer" comes from the fact that Schnauzers are search and rescue dogs (similar to this search and
replace task) and because they have mustaches.)

### Templates
Templates use the [handlebars templating language](https://handlebarsjs.com/) to express configuration files in any
textual format. All template files must be prefixed with a TOML *frontmatter* section, which tells the template
renderer which settings to use, as well as how to import any helpers needed.

An template file could look something like this:

```toml
[required-extensions]
frobnicate = "v1"  # The version of the helper can be specified as a string...
std = { version = "v1", helpers = ["base64_decode"] } # ... or use the object form to import helpers.

# Use at least three `+` characters to separate the frontmatter from the template body.
+++
{
    "enabled": settings.frobnicate.enabled,
    "frobnicate-key": "{{ base64_decode settings.frobnicate-key }}"
}
```

### The schnauzer Library
The primary user interface is provided via `schnauzer::render_template` and `schnauzer::render_template_file`.
These functions require the user to pass the template, as well as a `TemplateImporter`, which tells schnauzer how
to resolve references to settings extensions and the requested helpers.

Most users will want to use `schnauzer::BottlerocketTemplateImporter`, which uses settings extensions to resolve
settings and helper data; however, custom `TemplateImporter` implementrations can be used as well.

For static datasets to be used for tests, enable the `testfakes` feature in `Cargo.toml`.

### The schnauzer Settings Generator


### schnauzer v1
schnauzer was originally written to render simpler templates which always had the full scope of Bottlerocket
settings and helper functions available to them, making it incompatible with the concept of Out-of-Tree Builds.

The original schnauzer library deprecated, but continues to be made available under `schnauzer::v1` until it can
be safely removed. The original schnauzer settings generator is still provided as `schnauzer`, until it can be
removed and replaced with the `schnauzer-v2` generator.

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
