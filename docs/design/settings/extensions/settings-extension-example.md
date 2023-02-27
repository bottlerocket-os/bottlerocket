# Settings Extensions Example
This document walks through a typical use-case for Bottlerocket's settings extension system.
Technical details of the design of the settings extension system are described elsewhere.

## Background
Suppose you are creating a Bottlerocket variant which adds some new daemon software that you've created.
The daemon is configured via a YAML configuration file with a web address.
When the daemon starts, it parses its configuration file for the target address and then begins periodically querying that address for updates.

You'd like the following behavior to be accomplished:
* You can configure the daemon using the Bottlerocket settings API, the value will be stored in `settings.mydaemon.query-url`.
* Whenever the daemon is reconfigured using the Bottlerocket API, it is automatically restarted with the new configuration.

In order to do this, we need to create the following resources in our Bottlerocket variant:
* A setting extension crate called `mydaemon-settings` to add the new value `settings.mydaemon.query-url` to the API.
* A `template` which will be rendered using the setting value and used as a configuration for the daemon.
* A `service` file, which tells Bottlerocket how to handle the lifecycle of our daemon process as its settings change.

## Creating the Settings Extension
The easiest way to create an extension is to create a new Rust package in your project which utilizes the `bottlerocket-settings-sdk` crate.

```
├── packages
├── sources
│   └── mydaemon-settings
│       ├── Cargo.toml
│       └── src
│           └── main.rs
└── variants


# sources/mydaemon/Cargo.toml
[package]
name = "mydaemon-settings"
...
[dependencies]
bottlerocket-settings-sdk = 1.0
...
```

Implementing the `bottlerocket_settings_sdk::Extension` trait provides a straightforward path to tell Bottlerocket how to validate and store the desired `query-url` setting.

```rust
#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Clone)]
struct MyDaemonSettingV1 {
    query_url: Url,
}

impl SettingsModel for MyDaemonSettingV1 {
    fn get_version() -> &'static str { /* */ }

    fn set(current_value: Option<Self>, new_value: Self) -> Result<Self> { /* */ }
    
    // etc
}
```

You can then refer to this model when invoking the `SettingsExtension` entrypoint to the SDK:

```rust
fn main() -> Result<()> {
    bottlerocket_settings_sdk::SettingsExtension::with_models(vec![
        BottlerocketSetting::<MyDaemonSettingsV1>::model(),
    ])
    .run_extension()
    .context("Settings extension encountered an error.")
}
```


## Configuration File Template
In your variant repository, you will need to create an RPM package which installs `mydaemon` into your custom Bottlerocket variant.
In this package, you will also want to include a templated configuration file, which is what the settings system interacts with:

```
├── packages
│   └── mydaemon
│       ├── build.rs
│       ├── Cargo.toml
│       ├── mydaemon.spec
│       ├── mydaemon.template.yaml
│       └── pkg.rs
├── sources
└── variants

# mydaemon.template.toml
---
[required-extensions]
mydaemon: v1
---
query-url: {{ mydaemon.query-url }}
```
The configuration template includes "frontmatter" which informs the settings system to use settings values owned by the `mydaemon` extension, then the rest of the template is used to render the configuration file.

## Service File
Changing settings typically results in changes to configuration file templates, which results in systemd services being restarted to trigger the change in behavior.
You should specify a `mydaemon.service` systemd service file which will be installed into the Bottlerocket system and used to manage the lifecycle of your daemon.

```
├── packages
│   └── mydaemon
│       └── ...
│       └── mydaemon.service
├── sources
└── variants
```

## Resulting Disk Layout
We've discussed constructing a setting, using a configuration file template, and associating that with a service.
The Bottlerocket build system must arrange these files on disk to produce an image file.
This section discusses how the various artifacts will be arranged on a running system.

### Settings Extension Binaries
Extension binaries are stored under `sys-root/usr/libexec` and symlinked into `sys-root/usr/libexec/settings`.
`sys-root/usr/share/settings` will also hold the config file for the settings extensions, which will be named `${SETTING_NAME}.toml`
The setting owned by the extension will share a name with the symlink.


For example, the extension for `my-daemon` may be installed like so:
```
sys-root
└── usr
    ├── libexec
    │   ├── my-daemon-settings-extension
    │   └── settings
    │       └── my-daemon -> ../my-daemon-settings-extension
    └── share
        └── settings
            └── my-daemon.toml
```

In this case, Bottlerocket will use the extension symlinked to `sys-root/usr/libexec/settings/my-daemon` to control all settings named `my-daemon.$SETTING_NAME`.

### Configuration Templates

Templates will continue to be stored in their location in current Bottlerocket under `sys-root/usr/share/templates`; however, the format will change to support arbitrarily associating these templates to services.

In the below case, the configuration file for `my-daemon` is templated into a file called `my-daemon-conf.template`.
We've configured, via symlink, the `my-daemon.service` service to be restarted whenever the `my-daemon-conf` template is re-rendered.

```
sys-root
└── usr
    └── share
        └── templates
            ├── my-daemon-conf.template
            ├── my-daemon-conf.template.affected-services
            │       └── my-daemon.service -> /etc/systemd/system/my-daemon.service
            └── my-daemon-conf.template.rendered-to
                    └── render-my-daemon-conf.conf
```

`my-daemon-conf.template.rendered-to/render-my-daemon-conf.conf` contains instructions similar to `systemd-tmpfiles.d` explaining locations to which the rendered template will be placed.
The format of this file dictates the resulting file location, mode, UID, and GID of the resulting file:

```
/etc/mydaemon/mydaemon.json - - -
/etc/mydeamon.env 0755 root root
```
