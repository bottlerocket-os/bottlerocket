/*!

A convenience macro for Bottlerocket model types.

## Description

The `Scalar` macro can be used for strings or numbers that need to be validated in the Bottlerocket
model. It uses a trait, also named `Scalar`, to treat a `struct` as a thin wrapper around an
internal scalar type.

The macro expects your inner scalar type to implement `Display`, `PartialEq`, `Serialize` and
`Deserialize`. It then implements these traits on the wrapper type by passing them through to
the inner type.

You are also expected to implement the `Validate` trait on your `Scalar` type (the wrapper, not
the inner type). This macro will call `<YourType as Validate>::validate(some_value)` when
implementing `YourType::new`.

## Parameters

The macro can take the following input parameters (in most cases you will not need to use these; the
defaults will "just work"):
- `as_ref_str: bool`: Set to `true` if need the macro to treat your inner type as a `String`.
   This will happen automatically if your inner type is named `String`.
- `inner`: The name of the field that holds your `inner` type. Defaults to `inner`.

# Examples

## Simple Usage

This is an example of a very common use-case in Bottlerocket. We have a string, but we want to
validate it. In this example we want to return an error if the string is "pineapple".

```
use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;

// We create a struct with an inner type in a field named `inner`. We derive `Scalar`.
#[derive(Debug, PartialEq, Scalar)]
struct Pizza {
    inner: String
}

// We must implement the `Validate` trait for out type.
impl Validate for Pizza {
    fn validate<S: Into<String>>(input: S) -> Result<Pizza, ValidationError> {
        let input: String = input.into();
        if input == "pineapple" {
            Err(ValidationError::new("pineapple is gross on pizza"))
        } else {
            Ok(Self{ inner: input })
        }
    }
}

// The `Scalar` derive macro has made it so that we can use `Pizza` as if it were a `String`,
// but we know that the value has been validated.

let pizza = Pizza::new("pepperoni").unwrap();
// `pizza` behaves like a string!
assert!("pepperoni" == pizza);

let err = Pizza::new("pineapple");
// no that's gross!
assert!(err.is_err());
```

## Validating a Number

Here we use the Scalar macro with a numeric inner type. The inner value is constrained to be less
than 4.

```
use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;

#[derive(Debug, PartialEq, Scalar)]
struct CatQuantity {
    inner: i32
}

impl Validate for CatQuantity {
    fn validate<I: Into<i32>>(input: I) -> Result<CatQuantity, ValidationError> {
        let input: i32 = input.into();
        if input > 4 {
            Err(ValidationError::new("that's too many cats"))
        } else {
            Ok(Self{ inner: input })
        }
    }
}

let cat_quantity = CatQuantity::new(2).unwrap();
// `cat_quantity` can be compared to a i32
assert!(2 == cat_quantity);

let err = CatQuantity::new(23);
// no that's too many!
assert!(err.is_err());
```

## Using Macro Parameters

In this example we will show how to use the parameters `as_ref_str` and `inner`. This example is
contrived, but demonstrates how to pass parameters to the derive macro.

```
use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
struct WeirdType;

// We need `AsRef<str>` to treat it like a string with `as_ref_str`.
impl AsRef<str> for WeirdType {
    fn as_ref(&self) -> &str {
        "i'm a weird type"
    }
}

// We also need `From<&str>` to treat it like a string with `as_ref_str`.
impl From<&str> for WeirdType {
    fn from(_: &str) -> WeirdType {
        WeirdType
    }
}

// We also need `Deref` to treat it like a string with `as_ref_str`.
impl std::ops::Deref for WeirdType {
    type Target = str;
    fn deref(&self) -> &str {
        self.as_ref()
    }
}

// We also need `Borrow` to treat it like a string with `as_ref_str`.
impl core::borrow::Borrow<str> for WeirdType {
    fn borrow(&self) -> &str {
        self.as_ref()
    }
}

// We also need `Display` to work.
impl std::fmt::Display for WeirdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

// Here we create a struct that doesn't use the default name for the inner field. We also tell
// the `Scalar` macro to treat the inner type like a string.
#[derive(Debug, PartialEq, Scalar)]
#[scalar(as_ref_str = true, inner = "some_field")]
struct MyWrapper {
    some_field: WeirdType,
}

impl Validate for MyWrapper {
    fn validate<T: Into<WeirdType>>(input: T) -> Result<MyWrapper, ValidationError> {
        Ok(Self{ some_field: input.into() })
    }
}
```

*/

use darling::FromAttributes;
use proc_macro::TokenStream;
use quote::{format_ident, ToTokens};
use quote::{quote, TokenStreamExt};
use syn::__private::TokenStream2;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Fields};

/// A convenience macro for implementing Bottlerocket model types. See description in the lib
/// documentation or README.
#[proc_macro_derive(Scalar, attributes(scalar))]
pub fn scalar(input: TokenStream) -> TokenStream {
    // Parse the input tokens.
    let derive_input = parse_macro_input!(input as DeriveInput);
    let settings = RawSettings::from_attributes(derive_input.attrs.as_slice())
        .expect("Unable to parse `scalar` macro arguments");

    // Further parse the input.
    let struct_info = StructInfo::new(&derive_input, settings);

    // Write impls.
    let mut ast2 = TokenStream2::new();
    struct_info.write_impls(&mut ast2);
    ast2.into_token_stream().into()
}

/// Store args given by the user inside `#[scalar(as_ref_str=false, ...)]` using `darling` (it's
/// kind of like `clap` for derive macro arguments).
#[derive(Debug, Clone, Default, FromAttributes)]
#[darling(attributes(scalar))]
#[darling(default)]
struct RawSettings {
    /// Whether or not `AsRef<str>` (and similar impls) should be created. If the inner type is
    /// `String` (or implements `AsRef<str>`) then it is more convenient when using the `Scalar` if
    /// it treats references to its inner type as `&str` instead of `&String`. This defaults to
    /// `true` when the inner type is `String` and `false` otherwise.
    as_ref_str: Option<bool>,
    /// The name of the field that holds the inner value in the struct. Defaults to "inner".
    inner: Option<String>,
}

/// Once we parse the incoming AST and see what our struct is named, see what its inner type is,
/// and introspect the macro arguments, we save the resultant information in this structure for
/// use during code generation.
#[derive(Debug, Clone)]
struct StructInfo {
    /// The typename of the struct, that the `Scalar` derive macro was called on.
    scalar: String,
    /// The name of the field, inside the `scalar` struct, that holds the "inner" value.
    inner_field: String,
    /// The type of the `inner_field`.
    inner_type: String,
    /// Whether or not we should treat the inner reference type as `&str`.
    as_ref_str: bool,
}

impl StructInfo {
    fn new(derive_input: &DeriveInput, settings: RawSettings) -> Self {
        let scalar = derive_input.ident.to_string();

        let (inner_field, inner_type) = match derive_input.data.clone() {
            Data::Struct(s) => find_inner_field(s, settings.inner.as_ref().map(|s| s.as_str())),
            Data::Enum(_) => panic!("A Scalar cannot be an enum, it must be a struct"),
            Data::Union(_) => panic!("A Scalar cannot be an union, it must be a struct"),
        };

        // Automatically impl AsRef<str> when unspecified by the user but the inner type is String.
        // Note, this might not work if String is not what we think it is. We assume that anything
        // named `String`, `string::String`, or `std::string::String` is, in fact, a
        // `std::string::String`.
        let as_ref_str = settings.as_ref_str.unwrap_or(is_string(&inner_type));

        Self {
            scalar,
            inner_field,
            inner_type,
            as_ref_str,
        }
    }

    /// Returns an `Ident` named `str` when we want to treat the inner type as a `String` (i.e. we
    /// want to impl `AsRef<str>` and such). Otherwise returns the inner typename as an `Ident`.
    fn inner_ref_type(&self) -> proc_macro2::Ident {
        format_ident!(
            "{}",
            if self.as_ref_str {
                String::from("str")
            } else {
                format!("{}", self.inner_type)
            }
        )
    }

    fn write_impls(&self, stream: &mut TokenStream2) {
        // We need to store our information in local variables for the quote macro.
        let scalar = format_ident!("{}", &self.scalar);
        let inner_type = format_ident!("{}", &self.inner_type);
        let inner_ref_type = self.inner_ref_type();

        // Create the Scalar trait implementation, which is different based on whether the inner
        // field is named or unnamed (i.e. in a tuple struct like this `MyStruct(String)`
        let trait_impl = if let Ok(index) = self.inner_field.parse::<usize>() {
            // If the inner field is unnamed, i.e. `0` or `1`, etc., we need to cast it as an index
            // so that quote will do the right thing with it.
            let index = syn::Index::from(index);
            quote!(
                impl scalar::traits::Scalar for #scalar {
                    type Inner = #inner_type;

                    fn new<T: Into<Self::Inner>>(inner: T) -> Result<Self, scalar::ValidationError>
                    where
                        Self: Sized,
                    {
                        Ok(<#scalar as scalar::traits::Validate>::validate(inner.into())?)
                    }

                    fn inner(&self) -> &Self::Inner { &self.#index }

                    fn unwrap(self) -> Self::Inner { self.#index }
                }
            )
        } else {
            // If the inner field is named, we cast it as an identifier.
            let inner_field = format_ident!("{}", &self.inner_field);
            quote!(
                impl scalar::traits::Scalar for #scalar {
                    type Inner = #inner_type;

                    fn new<T: Into<Self::Inner>>(inner: T) -> Result<Self, scalar::ValidationError>
                    where
                        Self: Sized,
                    {
                        Ok(<#scalar as scalar::traits::Validate>::validate(inner.into())?)
                    }

                    fn inner(&self) -> &Self::Inner { &self.#inner_field }

                    fn unwrap(self) -> Self::Inner { self.#inner_field }
                }
            )
        };

        // Generate code.
        let impls = quote!(
            #trait_impl

            impl std::convert::TryFrom<<#scalar as scalar::traits::Scalar>::Inner> for #scalar {
                type Error = ValidationError;
                fn try_from(input: <#scalar as scalar::traits::Scalar>::Inner) -> Result<Self, ValidationError> {
                    Self::new(input)
                }
            }

            impl<'de> serde::de::Deserialize<'de> for #scalar {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::de::Deserializer<'de>,
                {
                    let original = <#scalar as scalar::traits::Scalar>::Inner::deserialize(deserializer)?;
                    // We need to make sure the serde Error trait is in scope.
                    use serde::de::Error as _;
                    let scalar = #scalar::new(original).map_err(|e| {
                        D::Error::custom(format!("Unable to deserialize into {}: {}", stringify!(#scalar), e))
                    })?;
                    Ok(scalar)
                }
            }

            impl serde::ser::Serialize for #scalar {
                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: serde::ser::Serializer,
                {
                    self.inner().serialize(serializer)
                }
            }

            impl std::fmt::Display for #scalar {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}", self.inner())
                }
            }

            impl std::ops::Deref for #scalar {
                type Target = #inner_ref_type;
                fn deref(&self) -> &Self::Target {
                    self.inner()
                }
            }

            impl core::borrow::Borrow<#inner_ref_type> for #scalar {
                fn borrow(&self) -> &#inner_ref_type {
                    self.inner()
                }
            }

            impl AsRef<#inner_ref_type> for #scalar {
                fn as_ref(&self) -> &#inner_ref_type {
                    self.inner()
                }
            }

            impl PartialEq<<#scalar as scalar::traits::Scalar>::Inner> for #scalar {
                fn eq(&self, other: &<#scalar as scalar::traits::Scalar>::Inner) -> bool {
                    (*(&self.inner())).eq(other)
                }
            }

            impl PartialEq<#scalar> for <#scalar as scalar::traits::Scalar>::Inner {
                fn eq(&self, other: &#scalar) -> bool {
                    self.eq(*(&other.inner()))
                }
            }

            impl From<#scalar> for <#scalar as scalar::traits::Scalar>::Inner {
                fn from(scalar: #scalar) -> Self {
                    scalar.unwrap()
                }
            }
        );

        stream.append_all(impls.into_iter());

        // Generate code that is only applicable if the inner type can be treated like a String.
        if self.as_ref_str {
            let code = quote!(
                impl std::convert::TryFrom<&#inner_ref_type> for #scalar {
                    type Error = ValidationError;
                    fn try_from(input: &#inner_ref_type) -> Result<Self, ValidationError> {
                        Self::new(input)
                    }
                }

                impl PartialEq<#inner_ref_type> for #scalar {
                    fn eq(&self, other: &#inner_ref_type) -> bool {
                        let self_as_str: &str = self.as_ref();
                        let other_as_str: &str = other.as_ref();
                        self_as_str.eq(other_as_str)
                    }
                }

                impl PartialEq<#scalar> for #inner_ref_type {
                    fn eq(&self, other: &#scalar) -> bool {
                        let self_as_str: &str = self.as_ref();
                        let other_as_str: &str = other.as_ref();
                        self_as_str.eq(other_as_str)
                    }
                }

                impl PartialEq<#scalar> for &#inner_ref_type {
                    fn eq(&self, other: &#scalar) -> bool {
                        let other_as_str: &str = other.inner().as_ref();
                        let self_as_str: &str = self.as_ref();
                        self_as_str.eq(other_as_str)
                    }
                }

                impl PartialEq<&#inner_ref_type> for #scalar {
                    fn eq(&self, other: &&#inner_ref_type) -> bool {
                        let other_as_str: &str = (*other);
                        let self_as_str: &str = self.as_ref();
                        self_as_str.eq(other_as_str)
                    }
                }
            );
            stream.append_all(code.into_iter());
        }

        // If the inner type is String, then we already have this implemented. If not, we can add
        // this as a convenience since we know the inner type implements `Display`.
        if !is_string(&self.inner_type) {
            let code = quote!(
                impl From<#scalar> for String {
                    fn from(scalar: #scalar) -> Self {
                        scalar.unwrap().to_string()
                    }
                }
            );
            stream.append_all(code.into_iter());
        }
    }
}

/// Given a type `t`, what is its `typename`?
fn typename(t: &syn::Type) -> String {
    let mut stream = proc_macro2::TokenStream::new();
    t.to_tokens(&mut stream);
    stream.to_string()
}

/// Whether or not the type assumed to be `std::string::String`
fn is_string(t: &str) -> bool {
    t == "String" || t == "string::String" || t == "std::string::String"
}

/// This function finds the "inner" field of the struct. The most common example is:
///
/// ```no_run
/// struct SomeStruct {
///     inner: String
/// }
/// ```
///
/// In the above example, this function would return the field ("inner", "String")
fn find_inner_field(data_struct: DataStruct, field_name: Option<&str>) -> (String, String) {
    match &data_struct.fields {
        Fields::Named(named_fields) => {
            let field_name = field_name.unwrap_or("inner");
            for field in &named_fields.named {
                if let Some(field_ident) = &field.ident {
                    let field_ident: &syn::Ident = &field_ident;
                    if field_ident == field_name {
                        return (field_name.to_string(), typename(&field.ty));
                    }
                }
            }
            panic!(
                "The Scalar derive macro could not find a field named '{}', in this struct",
                field_name
            );
        }
        Fields::Unnamed(unnamed_field) => {
            let field_name = field_name.unwrap_or("0");
            return (
                field_name.to_string(),
                typename(
                    &unnamed_field
                        .unnamed
                        .iter()
                        .next()
                        .expect(
                            "The Scalar macro could not parse the unnamed fields of this struct",
                        )
                        .ty,
                ),
            );
        }
        Fields::Unit => {
            panic!(
                "The Scalar derive macro does not work on 'unit' types, it should have one or more \
                fields"
            )
        }
    }
}
