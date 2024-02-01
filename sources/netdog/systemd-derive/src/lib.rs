/*!

A macro to serialize structs to `systemd` unit file format

## Description

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

## Parameters

The `SystemdUnit` macro takes no parameters.

The `SystemdUnitSection` macro requires the following input parameters.
- `section`: The name of the section this struct represents.  This parameter is set on the struct.
- `entry`: The configuration entry name (which may be different than the struct member name).  This parameter must be set on each struct member.

The `SystemdUnitSection` macro has the following optional input parameters:
- `space_separated`: Meant for use with `Vec`s, this boolean parameter will join the items in the `Vec` with a space.  If used on an `Option`, the parameter has no effect and the field is displayed normally without changes.

# Example

This is an abbreviated set of structs that could represent a `systemd-networkd` .network file.

```ignore
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

```ignore
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
```ignore
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
*/

use darling::{ast, FromDeriveInput, FromField, ToTokens};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

/// A macro to simplify serializing a unit file.  See the description in the lib documentation
/// or the README.
#[proc_macro_derive(SystemdUnit)]
pub fn derive_systemd_unit(input: TokenStream) -> TokenStream {
    // Parse the AST and "deserialize" into SystemdUnit
    let ast = parse_macro_input!(input as DeriveInput);
    let n =
        SystemdUnit::from_derive_input(&ast).expect("Unable to parse `systemd` macro arguments");

    quote!(#n).into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
struct SystemdUnit {
    pub ident: Ident,
    pub data: ast::Data<(), SystemdSection>,
}

#[derive(Debug, FromField)]
struct SystemdSection {
    ident: Option<Ident>,
}

impl ToTokens for SystemdUnit {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let SystemdUnit { ident, data } = self;

        let sections: Vec<Ident> = data
            .as_ref()
            .take_struct()
            // The annotation supports(struct_named) ensures our input will always be a struct
            .expect("Will never be anything but a struct")
            .fields
            .iter()
            .filter_map(|f| f.ident.clone())
            .collect();

        tokens.extend(quote! {
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    // `Vec`s and `Option`s are both iterators, allowing us to write the same code
                    // for both
                    #(for section in self.#sections.iter() {

                            write!(f, "{}", section.to_string())?;
                    })*
                    Ok(())
                }
            }
        });
    }
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

/// A macro to simplify serializing a section of a unit file.  See the description in the lib
/// documentation or the README.
#[proc_macro_derive(SystemdUnitSection, attributes(systemd))]
pub fn derive_systemd_unit_section(input: TokenStream) -> TokenStream {
    // Parse the AST and "deserialize" into SystemdUnitSection
    let ast = parse_macro_input!(input as DeriveInput);
    let n = SystemdUnitSection::from_derive_input(&ast)
        .expect("Unable to parse `systemd` macro arguments");

    quote!(#n).into()
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
#[darling(attributes(systemd))]
struct SystemdUnitSection {
    pub ident: Ident,
    pub data: ast::Data<(), SystemdUnitSectionField>,
    #[darling(rename = "section")]
    pub section_name: String,
}

#[derive(Debug, FromField)]
#[darling(attributes(systemd))]
struct SystemdUnitSectionField {
    ident: Option<Ident>,
    entry: String,
    #[darling(default)]
    space_separated: bool,
}

impl ToTokens for SystemdUnitSection {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let SystemdUnitSection {
            ident,
            data,
            section_name,
        } = self;

        let entries = data
            .as_ref()
            .take_struct()
            // supports(struct_named) ensures our input will always be a struct
            .expect("Will never be anything but a struct")
            .fields;

        // Defensively remove any brackets from the name, since we do that for the user
        let section_name = section_name.replace(['[', ']'], "");

        tokens.extend(quote! {
            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "[{}]\n", #section_name)?;

                    #(#entries)*
                    Ok(())
                }
            }

        });
    }
}

impl ToTokens for SystemdUnitSectionField {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let struct_field_name = self.ident.as_ref().expect("Should always have a name");
        let systemd_entry_name = &self.entry;
        // `Vec`s and `Option`s are both iterators, allowing us to write the same code for both
        let field = if self.space_separated {
            // Be friendly and don't choke if this attribute is placed on an `Option` rather than a
            // Vec.  Turn it into an interator, and because all fields must implement Display we
            // can call `to_string()` on them.  Having a `Vec<String>` guarantees we can use
            // `join()`.  `join()` is implemented for `Vec<String>` but not guaranteed for all
            // other types.
            quote! {
                let joined = self.#struct_field_name.clone().into_iter().map(|t| t.to_string()).collect::<Vec<String>>().join(" ");
                if !joined.is_empty() {
                    write!(f, "{}={}\n", #systemd_entry_name, joined)?;
                }
            }
        } else {
            quote! {
                for field in self.#struct_field_name.iter() {
                    write!(f, "{}={}\n", #systemd_entry_name, field)?;

                }
            }
        };

        tokens.extend(field)
    }
}
