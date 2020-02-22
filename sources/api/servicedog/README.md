# servicedog

Current version: 0.1.0

## Background

servicedog is a simple systemd unit supervisor.
Its job is to start/stop and enable/disable systemd units based on a setting value it is told to query.

When a setting changes, thar-be-settings does its job and renders configuration files and calls all restart-commands for any affected services.
For settings that represent the desire state of a service, servicedog can be included in the list of restart-commands to manipulate the state of the service based on the value of the setting.
It's provided the name of a setting to query, as well as the systemd unit to act on.
First it queries the value of the setting; the only supported values at this time are "true" and "false".
If the setting is true, servicedog attempts to start and enable the given systemd unit. If the setting is false, it stops and disables the unit.
As its very last step, service dog calls `systemd daemon-reload` to ensure all changes take affect.


## Colophon

This text was generated using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/main.rs`.