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
settings and helper data; however, custom `TemplateImporter` implementations can be used as well.

For static datasets to be used for tests, enable the `testfakes` feature in `Cargo.toml`.

### The schnauzer Settings Generator
Bottlerocket settings can be generated via `schnauzer-v2` (pending rename to `schnauzer`), which emits rendered
templates as JSON strings compatible with the Bottlerocket API.
The setting generator allows the user to specify any template requirements via the CLI, rather than requiring
frontmatter, e.g.
`schnauzer-v2 render --requires 'settings@v1(helpers=[myhelper])' --template 'foo-{{ myhelper settings.bar }}'`.

### schnauzer v1
schnauzer was originally written to render simpler templates which always had the full scope of Bottlerocket
settings and helper functions available to them, making it incompatible with the concept of Out-of-Tree Builds.

The original schnauzer library is deprecated, but continues to be made available under `schnauzer::v1` until it can
be safely removed. The original schnauzer settings generator is still provided as `schnauzer`, until it can be
removed and replaced with the `schnauzer-v2` generator.

#### Migrating Templates from v1 to v2
To migrate a template from schnauzer v1 to schnauzer v2, you must first identify each of the
settings and helpers used in the template. With the introduction of settings extensions to
Bottlerocket, all settings and helpers are owned by a settings extension, and each used extension
(including the version used) must be explicitly specified in frontmatter.

For each setting, the top-level key in the setting name will be the same as the extension name.
The version of the extension can be discovered via that extension's documentation.

For example, if your template uses `settings.foo.bar` and `settings.foo.baz`, then the extension
must be imported as `foo`:

```toml
[required-extensions]
foo = "v1"
+++
# The rest of the template now goes after the `+++` frontmatter delimiter.
```

For each helper used in your v1 template, you must determine the extension that owns it, and
then similarly declare them in frontmatter.

To expand on the previous example, suppose we are using `barify` and `bazify` helpers in our v1
template, and we know that these are both owned by the `fooify` extension. We can modify the
frontmatter like so:

```toml
[required-extensions]
foo = "v1"
fooify = { version = "v1", helpers = ["barify", "bazify"] }
+++
# The rest of the template now goes after the `+++` frontmatter delimiter.
```

NOTE: `schnauzer-v2` was merged into Bottlerocket prior to the complete introduction of settings
extensions. Until these are merged, all extensions will be imported as "v1". Determining the
ownership of a helper can be discovered by checking `schnauzer`'s [`v2::import::helpers`]
module.


## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.
