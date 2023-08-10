use schnauzer::v2::{template, Template};

macro_rules! assert_parse_error {
    ($error:ident, $template:expr) => {
        let parsed = include_str!($template).parse::<Template>();
        println!("Parsed '{}' as {:?}", $template, parsed);
        match parsed {
            Err(template::error::Error::$error { .. }) => (),
            _ => panic!("Did not encounter expected error while parsing template."),
        };
    };
}

#[test]
fn fails_00_invalid_toml() {
    // Given a template with invalid toml in frontmatter,
    // when the template is parsed,
    // then a GrammarParse error is returned.
    assert_parse_error!(GrammarParse, "./templates/fails/00_invalid_toml.template");
}

#[test]
fn fails_01_valid_toml_missing_fields() {
    // Given a template with missing fields in frontmatter,
    // when the template is parsed,
    // then a FrontmatterParse error is returned
    assert_parse_error!(
        FrontmatterParse,
        "./templates/fails/01_valid_toml_missing_fields.template"
    );
}

#[test]
fn fails_02_valid_toml_extra_fields() {
    // Given a template with frontmatter containing extra fields,
    // when the template is parsed,
    // then a FrontmatterParse error is returned
    assert_parse_error!(
        FrontmatterParse,
        "./templates/fails/02_valid_toml_extra_fields.template"
    );
}

#[test]
fn fails_03_invalid_frontmatter_delim() {
    // Given a template with incorrect frontmatter delimiters,
    // when the template is parsed,
    // then a GrammarParse error is returned.
    assert_parse_error!(
        GrammarParse,
        "./templates/fails/03_invalid_frontmatter_delim.template"
    );
}

#[test]
fn fails_04_duplicate_helpers() {
    // Given a template with the same helper name imported from multiple extensions,
    // when the template is parsed,
    // then an error is returned.
    assert_parse_error!(
        HelperNameCollision,
        "./templates/fails/04_duplicate_helpers.template"
    );
}
