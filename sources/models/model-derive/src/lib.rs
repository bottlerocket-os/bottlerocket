/*!
# Overview

This module provides a attribute-style procedural macro, `model`, that makes sure a struct is
ready to be used as an API model.

The goal is to reduce cognitive overhead when reading models.
We do this by automatically specifying required attributes on structs and fields.

Several arguments are available to override default behavior; see below.

# Changes it makes

## Visibility

All types must be public, so `pub` is added.
Override this (at a per-struct or per-field level) by specifying your own visibility.

## Derives

All structs must serde-`Serializable` and -`Deserializable`, and comparable via `PartialEq`.
`Debug` is added for convenience.
`Default` can also be added by specifying the argument `impl_default = true`.

## Serde

Structs have a `#[serde(...)]` attribute added to deny unknown fields and rename fields to kebab-case.
The struct can be renamed (for ser/de purposes) by specifying the argument `rename = "bla"`.

Fields have a `#[serde(...)]` attribute added to skip `Option` fields that are `None`.
This is because we accept updates in the API that are structured the same way as the model, but we don't want to require users to specify fields they aren't changing.
This can be disabled by specifying the argument `add_option = false`.

## Option

Fields are all wrapped in `Option<...>`.
Similar to the `serde` attribute added to fields, this is because we don't want users to have to specify fields they aren't changing, and can be disabled the same way, by specifying `add_option = false`.
*/

extern crate proc_macro;

use darling::FromMeta;
use proc_macro::TokenStream;
use quote::ToTokens;
use syn::visit_mut::{self, VisitMut};
use syn::{
    parse_macro_input, parse_quote, Attribute, AttributeArgs, Field, ItemStruct, Visibility,
};

/// Define a `#[model]` attribute that can be placed on structs to be used in an API model.
/// Model requirements are automatically applied to the struct and its fields.
/// (The attribute must be placed on sub-structs; it can't be recursively applied to structs
/// referenced in the given struct.)
#[proc_macro_attribute]
pub fn model(args: TokenStream, input: TokenStream) -> TokenStream {
    // Parse args
    let attr_args = parse_macro_input!(args as AttributeArgs);
    let args =
        ParsedArgs::from_list(&attr_args).expect("Unable to parse arguments to `model` macro");
    let mut helper = ModelHelper::from(args);

    // Parse and modify source
    let mut ast: ItemStruct =
        syn::parse(input).expect("Unable to parse item `model` was placed on - is it a struct?");
    helper.visit_item_struct_mut(&mut ast);
    ast.into_token_stream().into()
}

/// Store any args given by the user inside `#[model(...)]`.
#[derive(Debug, Default, FromMeta)]
#[darling(default)]
struct ParsedArgs {
    rename: Option<String>,
    impl_default: Option<bool>,
    add_option: Option<bool>,
}

/// Stores the user's requested options, plus any defaults for unspecified options.
#[derive(Debug)]
struct ModelHelper {
    rename: Option<String>,
    impl_default: bool,
    add_option: bool,
}

/// Takes the user's requested options and sets default values for anything unspecified.
impl From<ParsedArgs> for ModelHelper {
    fn from(args: ParsedArgs) -> Self {
        // Add any default values
        ModelHelper {
            rename: args.rename,
            impl_default: args.impl_default.unwrap_or(false),
            add_option: args.add_option.unwrap_or(true),
        }
    }
}

/// VisitMut helps us modify the node types we want without digging through the huge token trees
/// need to represent them.
impl VisitMut for ModelHelper {
    // Visit struct definitions.
    fn visit_item_struct_mut(&mut self, node: &mut ItemStruct) {
        match node.vis {
            // If unset, make pub.
            Visibility::Inherited => node.vis = parse_quote!(pub),
            // Leave alone anything the user set.
            _ => {}
        }

        // Add our serde attribute, if the user hasn't set one
        if !is_attr_set("serde", &node.attrs) {
            // Rename the struct, if the user requested
            let attr = if let Some(ref rename_to) = self.rename {
                parse_quote!(
                    #[serde(deny_unknown_fields, rename_all = "kebab-case", rename = #rename_to)]
                )
            } else {
                parse_quote!(
                    #[serde(deny_unknown_fields, rename_all = "kebab-case")]
                )
            };
            node.attrs.push(attr);
        }

        // Add our derives, if the user hasn't set any
        if !is_attr_set("derive", &node.attrs) {
            // Derive Default, if the user requested
            let attr = if self.impl_default {
                parse_quote!(#[derive(Debug, Default, PartialEq, Serialize, Deserialize)])
            } else {
                parse_quote!(#[derive(Debug, PartialEq, Serialize, Deserialize)])
            };
            // Rust 1.52 added a legacy_derive_helpers warning (soon to be an error) that yells if
            // you use an attribute macro before the derive macro that introduces it.  We should
            // always put derive macros at the start of the list to avoid this.
            node.attrs.insert(0, attr);
        }

        // Let the default implementation do its thing, recursively.
        visit_mut::visit_item_struct_mut(self, node);
    }

    // Visit field definitions in structs.
    fn visit_field_mut(&mut self, node: &mut Field) {
        match node.vis {
            // If unset, make pub.
            Visibility::Inherited => node.vis = parse_quote!(pub),
            // Leave alone anything the user set.
            _ => {}
        }

        // Add our serde attribute, if the user hasn't set one
        if self.add_option {
            if !is_attr_set("serde", &node.attrs) {
                node.attrs.push(parse_quote!(
                    #[serde(skip_serializing_if = "Option::is_none")]
                ));
            }

            // Wrap each field's type in `Option<...>`
            let ty = &node.ty;
            node.ty = parse_quote!(Option<#ty>);
        }

        // Let the default implementation do its thing, recursively.
        visit_mut::visit_field_mut(self, node);
    }
}

/// Checks whether an attribute named `attr_name` (e.g. "serde") is set in the given list of
/// `syn::Attribute`s.
fn is_attr_set(attr_name: &'static str, attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if let Some(name) = attr.path.get_ident() {
            if name == attr_name {
                return true;
            }
        }
    }
    return false;
}
