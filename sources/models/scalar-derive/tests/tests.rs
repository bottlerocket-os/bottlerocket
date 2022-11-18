use scalar::traits::{Scalar, Validate};
use scalar::ValidationError;
use scalar_derive::Scalar;

// By default the `0` field will be our `inner` field.
#[derive(Debug, PartialEq, Scalar)]
struct SimpleString(String);

impl Validate for SimpleString {
    fn validate<T>(input: T) -> Result<Self, ValidationError>
    where
        T: Into<<Self as Scalar>::Inner>,
    {
        // No validation
        Ok(Self(input.into()))
    }
}

#[test]
fn simple_string() {
    let s = SimpleString::new("foo").unwrap();
    // Check that a few dereferencing conveniences compile
    let eq1 = "foo" == s;
    let eq2 = String::from("foo") == s;
    let eq3 = "foo" == &s;
    // Assert these in a way that doesn't use assert_eq and doesn't make my IDE mad
    assert!(eq1);
    assert!(eq2);
    assert!(eq3);
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// By default the `0` field will be our `inner` field.
#[derive(Debug, PartialEq, Scalar)]
struct UnnamedFields(u16, String);

impl Validate for UnnamedFields {
    fn validate<T>(input: T) -> Result<Self, ValidationError>
    where
        T: Into<<Self as Scalar>::Inner>,
    {
        let input = input.into();
        // Contrived weirdness
        if input == 0 {
            return Err(ValidationError::new("never zero"));
        } else if input == 1 {
            Ok(Self(2, "it was 1 but I changed it to 2".to_string()))
        } else {
            Ok(Self(input, "".to_string()))
        }
    }
}

#[test]
fn unnamed_fields() {
    let i = UnnamedFields::new(1u16).unwrap();
    let eq1 = 2u16 == i;
    let eq2 = &2u16 == &i;
    assert!(eq1);
    assert!(eq2);
}

// =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=   =^..^=

// We will make `1` the inner field
#[derive(Debug, PartialEq, Scalar)]
#[scalar(inner = "1")]
struct SecondField(u16, u16);

impl Validate for SecondField {
    fn validate<T>(input: T) -> Result<Self, ValidationError>
    where
        T: Into<<Self as Scalar>::Inner>,
    {
        Ok(Self(100u16, input.into()))
    }
}

#[test]
fn second_field() {
    let i = SecondField::new(3u16).unwrap();
    let eq1 = 3u16 == i;
    let eq2 = &3u16 == &i;
    assert!(eq1);
    assert!(eq2);
}
