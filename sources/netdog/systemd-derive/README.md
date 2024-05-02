# systemd-derive

Current version: 0.1.0


A macro to serialize structs to `systemd` unit file format

### Description

The `SystemdUnit` and `SystemdUnitSection` macros can be used to serialize structs representing
`systemd` unit files.

Under the hood, the macros implement `Display` for structs.  This allows converting the structs to
a string suitable for writing directly to a file.

The implementation is fairly rigid to the way the "INI-like" structure of `systemd` unit files is
represented. `systemd` differs from standard INI format in that duplicate sections are allowed,
along with duplicate keys within a section.  The macros expect there will be a "top-level" struct
that represents the unit file, with nested structs representing the sections of said file. These
nested structs representing sections have fields that are the configuration key/value pairs for
that section.

All struct fields must be either `Option`s or `Vec`s containing an object that implements
`Display`.  Fields that are `Vec`s will be iterated upon and each value of the `Vec` will be
serialized.  For structs deriving `SystemdUnit`, fields represent sections and therefore a `Vec`
field would serialize as a repeated section. Structs deriving `SystemdUnitSection` have fields
representing key/value pairs; a `Vec` here would serialize as a repeated entry within a section. An
example of both types is below.

### Parameters

The `SystemdUnit` macro takes no parameters.

The `SystemdUnitSection` macro requires the following input parameters.
- `section`: The name of the section this struct represents.  This parameter is set on the struct.
- `entry`: The configuration entry name (which may be different than the struct member name).  This parameter must be set on each struct member.

The `SystemdUnitSection` macro has the following optional input parameters:
- `space_separated`: Meant for use with `Vec`s, this boolean parameter will join the items in the `Vec` with a space.  If used on an `Option`, the parameter has no effect and the field is displayed normally without changes.

## Example

This is an abbreviated set of structs that could represent a `systemd-networkd` .network file.

```rust
use systemd_derive::{SystemdUnit, SystemdUnitSection};

// This top-level struct requires no parameters; it represents the file as a whole, and contains
// all the relevant sections.
//
// Pay special attention to the `route_sections` struct member.  It is a Vec, meaning that the
// section can be repeated multiple times within the file.
#[derive(Debug, Default, SystemdUnit)]
struct NetworkConfig {
    match: Option<MatchSection>,
    network: Option<NetworkSection>,
    route: Vec<RouteSection>,
}

// This struct represents the "Match" section.  The struct must be annoted with the section name
// ("Match"), and each of its fields must be annoted with the configuration entry name.
#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Match")]
struct MatchSection {
    #[systemd(entry = "Name")]
    name: Option<String>,
}

// This struct demonstrates the use of an entry ("Address") that can be repeated within a section.
// The "Address" entry will be serialized as multiple entries, each with a single value from the
// given Vec.
#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Network")]
struct NetworkSection {
    #[systemd(entry = "Address")]
    addresses: Vec<String>,
    #[systemd(entry = "DHCP")]
    dhcp: Option<String>,
}

#[derive(Debug, Default, SystemdUnitSection)]
#[systemd(section = "Route")]
struct RouteSection {
    #[systemd(entry = "Destination")]
    destination: Option<String>,
}
```

The following demonstrates instantiating an instance of the above structs and the resulting serialized form from calling `to_string()`.

```rust
let cfg = NetworkConfig {
    match: Some(MatchSection {
        name: Some("eno1".to_string()),
    }),
    network: Some(NetworkSection {
        addresses: vec!["1.2.3.4".to_string(), "2.3.4.5".to_string()],
        dhcp: Some("ipv4".to_string()),
    }),
    route: vec![
        RouteSection {
            destination: Some("10.0.0.1".to_string()),
        },
        RouteSection {
            destination: Some("11.0.0.1".to_string()),
        },
    ],
};

println!("{}", cfg.to_string());
```

Would result in the following being printed:
```rust
[Match]
Name=eno1

[Network]
Address=1.2.3.4
Address=2.3.4.5
DHCP=ipv4

[Route]
Destination=10.0.0.1

[Route]
Destination=11.0.0.1
```

## Colophon

This text was generated from `README.tpl` using [cargo-readme](https://crates.io/crates/cargo-readme), and includes the rustdoc from `src/lib.rs`.
