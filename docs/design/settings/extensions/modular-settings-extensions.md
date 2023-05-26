# Out-of-Tree Builds and Modular Settings Extensions
## Overview
The settings API is a fundamental aspect of Bottlerocket as a Linux distribution.
Therefore, as a design requirement for out-of-tree Bottlerocket builds, variants must be able to extend Bottlerocket’s settings system with custom settings.
Rather than build a separate system for handling out-of-tree settings, Bottlerocket will move all settings to a modular system -- each setting will have a module installed on the host at runtime.
These modules will be called "Settings Extensions".
Extensions will allow the same ergonomics and features of settings in current Bottlerocket, including the ability to migrate data between versions, render data into service configuration files, and interact with system service restarts

This document provides an overview of the overhauled settings system, including details about how settings data is stored and communicated between Bottlerocket components, and how settings extensions are developed.
Changes related to how settings extensions are packaged via RPM will be discussed in a separate document.

## Requirements
In an out-of-tree builds world, settings extensions from multiple different sources (Bottlerocket's core, settings defined by other variants, or your own settings) must all work harmoniously in a built system.
This makes it crucial that all interactions with settings resources adhere to versioned APIs, and that dependencies between settings extensions must be modeled in the build system in a similar way to dependencies on dynamically linked libraries.
Behaviorally, settings changes in other repositories should not cause breaking changes to other variant source trees, since these would constitute an API change and be vended as a new version.
Most of the more specific requirements of the system fall out of these principles:

* The settings API must support extensions through a series of stable, versioned, documented APIs that can be implemented by binaries and configurations maintained out-of-tree.
* Settings behavior must be determined dynamically at runtime based on the set of installed settings extensions.
* All autonomous interactions with the settings API must explicitly state the settings extension being interacted with, and at what version.
* Migrations between settings representations as we know them today must cease to exist -- settings extensions must independently expose settings at supported interface versions and implement data migrations between them.
* Settings extension interfaces, services, and configurations must all be represented in the build system, with the ability to express dependencies on these artifacts.

## Settings Extensions
This document provides an overview on the following:
* Storage
* Validation
* Retrieval
* Model migrations
* System configuration via templating
* Defaults generation
* Extensions development with the Settings SDK

### Setting Representation
Today, Bottlerocket's `apiserver` utilizes models to perform strict validation of settings inputs, which are then serialized to JSON and stored in Bottlerocket's datastore.
The models are defined using Rust structures and are statically compiled into the operating system, using Rust's type system and compiler to subsequently check interactions with these modelled objects.

In principle, this will remain true -- Bottlerocket will continue to own settings objects which it stores as JSON, and changes will be validated against models on update.
A primary difference here is that the settings objects will be opaque to Bottlerocket's core.
Bottlerocket will understand that settings are JSON objects and be able to make modifications requested when, for example, calling `apiclient set`; however, the changes are then submitted to the settings extensions installed on the system in order to validate the proposed changes.

### Extension Binaries
All settings in a Bottlerocket host will be described by their own settings extension, which are defined by a binary packaged onto the system and symlinked into a common directory, as well as a configuration file placed in a corresponding `settings.d`-style directory.
Extension binaries must respond to command line arguments following a specified protocol spoken by the Bottlerocket `apiserver`.
The protocol is versioned, so all commands to an extension binary begin with the protocol version (in this document, all commands fall under the initial `proto1` protocol version -- so-named to avoid conflation with settings versions, which will be explained later).
Command line arguments are used to signify the "request" being made of the extension.
Exit codes will be used to signal the status of the request, with formatted output being delivered on `stdout`, and logs delivered on `stderr`.

As an example, suppose a user wants to set the "message of the day" for their instance:
```bash
$ apiclient set motd="Hello, Bottlerocket!"
```

The apiserver would load the current value of `motd` and then create a new object based on the inputted string.
It would then find the extension responsible for `motd` and make a request to determine whether or not the change should proceed:

```bash
$ motd proto1 set --setting-version v1 --value '"Hello, Bottlerocket!"'  # Values are passed as JSON, hence the quoting
```

### Extension Naming & Setting Ownership
Every top-level setting in Bottlerocket's API must be owned by a singular extension, and that extension will share a name with the setting.
This name will also be shared by the extension's configuration file and binary components.
This means that whenever the system must interact with a setting (be it via user action on the API, or rendering templates to be used as configuration files), Bottlerocket can identify the setting extension to invoke in order to satisfy that interaction.

As an example, the following suggests the filesystem layout for a settings extension for the `settings.network` settings:

```
usr
└── lib
    └── bottlerocket
        └── settings
            ├── config.d
            │   └── network.toml
            └── extensions.d
                └── network -> /usr/bin/network-extension
```

#### Transactional Writes and Cross-Validation
Bottlerocket today is missing a mechanism for cross-validation of settings.
As an example, suppose we want to represent a range of integers in settings by allowing a `setting.min-value` and `setting.max-value` to be set -- it should be possible for settings extensions to set both simultaneously, while also verifying that the state from setting both is valid (`setting.min-value <= setting.max-value`).

Let us cast this requirement in a different light: imagine an OS administrator wishes to restrict what values can be written to an existing setting -- for example, they may wish to only allow settings values to `network.hosts` so long as they include as a subset the entries mandated by their organization.
To allow this, settings extensions will be capable of registering themselves as a *validator* for any other setting on the host, including settings that they do not own.
When handling write requests for multiple settings in a single transaction, the `apiserver` will first gather the resulting writes that would occur under each settings extension involved in the writes.
Then the `apiserver` will provide the provisional settings state to each settings extension which has been registered as a validator of any of the target settings.
Any one settings extension has the power to halt the transaction by returning a non-zero exit-code on validation.

Extensions must specify the setting which triggers the validation in their configuration file:

```toml
# auditor.toml -- setting extension config file for `settings.auditor`
[extension.validates]
network = "v1"
```

It is technically valid for an extension to specify that it validates a setting which does not exist; however, such a specification can never be triggered.
In the case that the setting exists but the specified version does not, all validations will result in a failure.
Settings extensions should model their dependency requirements (including validations) in RPM so to avoid cases where the resulting system is missing settings extensions with the appropriate version.

#### Datastore Layout
Once the `apiserver` has a target set of data to write for a given `set` request and it has been validated by all required validators, it must persist the new settings to the datastore.
Much like the current datastore, stored settings are stored to the filesystem via a particular pattern.

```bash
datastore/
└── motd
    ├── v1
    │   └── motd.json
    └── v2
        └── motd.json
```
Any data written to directories created during a `set` transaction is moved into the datastore.
Despite only one version of the data being provided, the `apiserver` will populate all versions supported by the settings extension at this time by requesting that the settings extension perform all necessary migrations.
Failure to populate a version listed as "supported" will result in a failed transaction.
See the next section about migrations for more details.

### Extension Versioning and Migrations
Bottlerocket's settings data model changes frequently: sometimes to support new features, or sometimes to correct mistaken model shapes.
In order to support changes to the settings model, settings must be capable of exposing versioning information about the settings extension, as well as providing faculties for migrating settings data to new versions.
Our versioning scheme will take heavy influence from Kubernetes' scheme for [versioning CustomResourceDefinitions](https://kubernetes.io/docs/tasks/extend-kubernetes/custom-resources/custom-resource-definition-versioning/).
Settings extension versioning information is surfaced through the extension config file:

```toml
# hostname.toml -- setting extension config file for settings.hostname
[extension]
supported-versions = [
     "v1",
     "v2"
]
default-version = "v2"
```

Any setting can be written or read at any of the `supported-versions` listed by the extension.
Unversioned requests to read or write this setting are assumed to be the `default-version` and should only take place when a user makes a request using the CLI -- all automated process *should* refer to an explicit version.
During a `set` transaction, the `apiserver` will consider the provided version to be the `canonical-version`, and will ask the settings extension to perform migrations from the `canonical` version to all supported versions using the `migrate` command.
These migrations always begin at the `canonical` version, and settings extensions are free to perform the migrations however they wish; however, Bottlerocket's Settings SDK will provide utilities to help settings developers support a series of linear setting upgrades and downgrades.
Read on for more information about the Settings SDK.

All migrations are performed and stored at setting-write-time because Bottlerocket rollbacks may result in running an OS with a settings extension which does not support the version at which a given setting is stored.
If Bottlerocket boots and finds that a setting is stored at a value which is not supported by its installed setting extension, it will allow that version to persist unless a new value is `set` by the `apiserver`, at which point it is removed -- this is to attempt to prevent data loss in the case that the image can eventually update to a new image which supports the stored setting version.

### Configuration Templating
Once Bottlerocket settings are stored in the datastore, they must somehow then be used to influence the behavior of the system.
Fetching settings from the Bottlerocket API/datastore is currently done via two mechanisms:
* Fetching data directly from the API using `apiclient`
* Rendering config files (or just strings) using `schnauzer`

For open-source tools, Bottlerocket prefers the rendered config file approach, as it doesn't require patching upstream code in order to consume the Bottlerocket API.
First-party tools have no such constraints, and so today there exists  tools that take either approach.

Because settings are discovered at runtime on the system, and indeed may no longer be handled by models written in Rust, it will no longer be feasible to create an `apiclient` with the same strong typing as exists today.
It will instead be preferred that system services, including first-party services, all route their fetching of API settings through their own configuration files, which will be rendered by Bottlerocket whenever the related settings are changed.

Today, configuration templates are rendered by `schnauzer` using [handlebars](https://handlebarsjs.com/) templates.
`schnauzer` fetches the entirety of the settings tree, defines some useful helper functions, and then uses these as context for rendering the template.
With the introduction of settings extensions, this will change such that templates specify `(name, version)` tuples for all extensions needed to render the template.
These extension specifications will be given within the template as "front matter" -- a TOML block specified at the head of the document and delineated with `+++` as such:

```toml
+++
[required-extensions]
hostname = v1
kubernetes = v1
+++
# Below here the template is written as usual
```

This will allow Bottlerocket to find the owning extensions for the data needed to render the template.
It also tightly binds the template to a given implementation of the requisite settings, making it much more difficult to accidentally break "downstream" templates with "upstream" changes.

`schnauzer` can also continue to be invoked as a standalone binary without the need for a templated configuration file.
In those circumstances, you are still required to list your extension dependencies and their version, but these can be done as command line arguments:

```bash
$ schnauzer --required-extension 'hostname=v1' '{{ network.hostname }}'
```

#### Extending Handlebars Helpers
Handlebars has a few utilities for applying simple processes to data in order to format it appropriately for your document; however, you often need to implement your own "helpers" for more complex text formatting.
This is currently done in Bottlerocket by adding helper implementations to the `schnauzer` tool directly.
Bottlerocket settings extensions will expose versioned handlebar helpers:

```bash
# hostname.toml setting extension config file
[templating.v1] # template helpers are versioned
helpers = [
    "localhost_aliases"
]


$ ./hostname proto1 template-helper --version v1 --helper localhost_aliases 'arg1' 'arg2'
#  rendered output returned as a string on stdout
```

Within the handlerbars template, the helper name will be prepended with `$EXTENSION_NAME.` to avoid collisions with other settings extensions.
As an example, to use the `localhost_aliases` helper above from a template, your template may look something like this:

```
{{ hostname.localhost_aliases(arg1, arg2) }}
```

### Settings Generation
Some settings have default values which cannot be known statically and must be computed after the system is running.
Bottlerocket settings currently have the ability to compute a default value via something called a "settings generator".
During boot, settings extensions are given a chance to populate settings with default values that are then persisted, effectively determining the setting value for the lifetime of that instance.

Settings generation often depends on the presence of other settings.
It's also possible for settings to have interdependencies on generation order, for example:

* `foo.bar` depends on `baz.bot`
* `boz.blop` depends on `foo.bar`

In order to resolve this, settings generators are allowed to require that they be provided whatever data is currently applied to dependent settings.
The generators are "re-entrant" in that they return a status dictating whether or not they are finished generating, or if they need more data to proceed.
Bottlerocket will iteratively invoke generators until all generation has completed, or in cases where it seems likely that a deadlock has occurred.

```toml
# setting.toml requires the network and kubernetes settings before it can emit its own
[generation.required]
network = v1
kubernetes = v2
```

## Settings Extension Development and the Settings SDK
Settings extensions can be written in any programming language, so long as a symlink to a binary speaking the "extension protocol" lands in the right spot on the filesystem.
That said, many settings extensions need to tackle the same challenges: How do we represent and implement settings versions? Migrations? Validation? Template utilities? Rather than forcing all extensions to implement this in isolation, we will create a shared library, called the "Settings SDK", which will be published to crates.io.

The goal of the settings SDK is to provide standard functions and macros for interfacing with the settings extension protocol.
While our model here provides quite a lot of power to settings extension developers, the SDK would allow us to exert some pressure in the form of opinions on simple ways to manage data or perform migrations.

As an example, the SDK will provide a Rust trait which can be implemented for each model in order to adequately implement all settings extensions methods on a command line interface.
Once those traits are implemented, you might implement this CLI as such:

```rust
fn main() -> Result<()> {
    bottlerocket_settings_sdk::SettingsExtension::with_models(vec![
        BottlerocketSetting::<v1::MotdV1>::model(),
        BottlerocketSetting::<v2::MotdV2>::model(),
    ])
    .run_extension()
    .context("Settings extension encountered an error.")
}
```

## Asked and Anticipated Questions
### Why not use some existing schema or modeling language to define the shape of settings?
Existing modeling languages are powerful, but even in our existing Bottlerocket settings we have validations which we have implemented as functions in the Rust code (or otherwise wished we had a simple way to do so.) Rather than strip power from settings extensions by dictating that they use one of these modeling languages, we leave it entirely up to the binary to decide how best to validate inputs.
In many cases, this may mean that the binaries carry with them their own implementation of a popular schema language, like JSONschema.

### How will the `apiserver` handle concurrent requests in the face of transactional writes?
The `apiserver` will use a [filesystem-based](https://man7.org/linux/man-pages/man2/flock.2.html) reader-writer lock to ensure that the datastore is only accessed by concurrent readers or a single writer. Writes which are sent to the `apiserver` while the file is lock are placed in a queue.
