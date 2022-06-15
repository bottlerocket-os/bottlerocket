# schnauzer

Current version: 0.1.0

## Introduction

schnauzer is called by sundog as a setting generator.
Its sole parameter is the name of the setting to generate.

The setting we're generating is expected to have a metadata key already set: "template".
"template" is an arbitrary string with mustache template variables that reference other settings.

For example, if we're generating "settings.x" and we have template "foo-{{ settings.bar }}", we look up the value of "settings.bar" in the API.
If the returned value is "baz", our generated value will be "foo-baz".

When dealing with settings that should return a non-string value (ex. booleans), just specify the JSON data type by adding `--type <json-data-type>` to the setting-generator line.
The resulting setting values will still be returned as stdout to sundog, but will not be wrapped in quotes.
For example, "schnauzer settings.foo.bar --type bool" could return false and the value in the API would be a proper boolean.

(The name "schnauzer" comes from the fact that Schnauzers are search and rescue dogs (similar to this search and replace task) and because they have mustaches.)

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.