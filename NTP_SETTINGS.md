# NTP settings

Fresh installations and upgrades retain the existing URL-list defaults.
Existing custom lists and explicit empty lists stay unchanged during upgrade.

```toml
[settings.ntp]
time-servers = ["169.254.169.123", "2.amazon.pool.ntp.org"]
options = ["iburst"]
```

Each legacy address renders as a `pool` with the shared `options`, including
fixed IP addresses.

To opt in to per-source directives and options, replace `time-servers` with
an object list. TOML arrays of tables express this list:

```toml
[[settings.ntp.time-servers]]
address = "169.254.169.123"
directive = "server"
options = ["prefer", "iburst", "minpoll 4", "maxpoll 4"]

[[settings.ntp.time-servers]]
address = "time.aws.com"
directive = "pool"
options = ["iburst"]
```

This renders:

```text
server 169.254.169.123 prefer iburst minpoll 4 maxpoll 4
pool time.aws.com iburst
```

Each object requires an address. An omitted directive defaults to `server`;
omitted options add no options. Object entries do not inherit the legacy
shared options. Options use chrony syntax, such as `"minpoll 4"`, without `=`.

Both list formats occupy one datastore value. API updates replace the entire
list, so include every source when editing one entry. Mixed string/object lists
are rejected. Setting `time-servers = []` explicitly disables all sources.
Replacing the object list with a URL list restores legacy rendering.

Complete named-map input is also accepted, but it replaces the entire list.
Entries are ordered by name, names are discarded, and readback returns an
object list. Partial named updates are not supported. The experimental named
datastore format was never shipped in an official Bottlerocket OS release;
existing named datastores are unsupported and have no transition migration.

## Optional logging

```toml
[settings.ntp]
logging = ["tracking", "statistics"]
```

This adds `log tracking statistics`. Logs are written to `/var/log/chrony`.
Omitted or empty logging adds no `log` directive. This setting works with
either list format and is removed during rollback to the old model.

## Rollback

Rollback converts object lists into address lists. The old template renders
every address as a `pool`; it cannot preserve per-source directives or unique
options. Only complete options shared by all sources are retained.

Option names are compared case-insensitively and split arguments are joined,
but argument text must match. For example:

- Sources with `["iburst", "minpoll 4"]` and `["iburst", "minpoll 04"]`
  retain only `["iburst"]`.
- One source containing both `"minpoll 4"` and `"minpoll 04"` is conflicting;
  the shared projection becomes `[]`.

Unknown, incomplete, or conflicting option syntax also produces an explicit
`settings.ntp.options = []`, preventing default shared options from being added.
This conservative projection does not promise to preserve semantically
equivalent argument spellings.

Invalid source entries, such as an object without a string address, return a
clear migration error and stop rollback instead of silently removing sources.
Legacy lists and explicit empty lists remain unchanged.
