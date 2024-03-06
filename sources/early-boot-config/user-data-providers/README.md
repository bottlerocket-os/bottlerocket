# user-data-providers

## Introduction

user-data-providers contains the user data provider binaries used by early-boot-config to set settings on boot. These binaries implement the interface defined in early-boot-config-provider.

When installed, these binaries should be linked to in `/usr/libexec/early-boot-config/data-providers.d/`. The binaries will be executed by early-boot-config in order based on the two numbers at the start of the link name, e.g.:

1. `10-local-defaults`
2. `20-local-file`
3. `99-local-overrides`
