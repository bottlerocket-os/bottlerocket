use maplit::hashset;
use schnauzer::v2::{ExtensionRequirement, Template};
use std::collections::HashSet;

#[test]
fn succeeds_00_basictemplate() {
    // Given a template,
    // when the template is parsed,
    // then the frontmatter and body will be read correctly.
    let template: Template = include_str!("./templates/succeeds/00_basic.template")
        .parse()
        .expect("Could not parse template file as template");

    let expected_requirements = hashset! {
        ExtensionRequirement {
            name: "motd".to_string(),
            version: "v1".to_string(),
            ..Default::default()
        }
    };
    let expected_body = "{{ settings.motd }}\n".to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}

#[test]
fn succeeds_01_loose_frontmatter_whitespace() {
    // Given a template with extraneous whitespace in the frontmatter,
    // when the template is parsed,
    // then the parsed frontmatter will disregard the whitespace.
    let template: Template =
        include_str!("./templates/succeeds/01_loose_frontmatter_whitespace.template")
            .parse()
            .expect("Could not parse template file as template");

    let expected_requirements = hashset! {
        ExtensionRequirement {
            name: "a".to_string(),
            version: "3".to_string(),
            ..Default::default()
        },
        ExtensionRequirement {
            name: "b".to_string(),
            version: "4".to_string(),
            ..Default::default()
        },
    };
    let expected_body = "body\n".to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}

#[test]
fn succeeds_02_strict_body_whitespace() {
    // Given a template with extraneous whitespace in the body,
    // when the template is parsed,
    // then the parsed body will maintain the whitespace.
    let template: Template =
        include_str!("./templates/succeeds/02_strict_body_whitespace.template")
            .parse()
            .expect("Could not parse template file as template");

    let expected_requirements = hashset! {
        ExtensionRequirement {
            name: "kubernetes".to_string(),
            version: "v2".to_string(),
            ..Default::default()
        },
    };
    let expected_body = "\n  body\n  \n".to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}

#[test]
fn succeeds_03_at_least_3_delims() {
    // Given a template with more than three '+' frontmatter delimiters,
    // when the template is parsed,
    // then the frontmatter and body will parse correctly.
    let template: Template = include_str!("./templates/succeeds/03_at_least_3_delims.template")
        .parse()
        .expect("Could not parse template file as template");

    let expected_requirements = hashset! {
        ExtensionRequirement {
            name: "extension".to_string(),
            version: "version".to_string(),
            ..Default::default()
        },
    };
    let expected_body = "config-value: {{ settings.extension.attribute }}\n".to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}

#[test]
fn succeeds_04_comments() {
    // Given a template with comments,
    // when the template is parsed,
    // then frontmatter comments will be ignore, body comments will be included in the output.
    let template: Template = include_str!("./templates/succeeds/04_comments.template")
        .parse()
        .expect("Could not parse template file as template");

    let expected_requirements = hashset! {
        ExtensionRequirement {
            name: "labrador".to_string(),
            version: "v1".to_string(),
            helpers: vec!["woof".to_string()],
            ..Default::default()
        },
    };
    let expected_body = "# comments are included in template\n{{ woof }}\n".to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}

#[test]
fn succeeds_05_unambiguous_delims() {
    // Given a template with frontmatter delimiters inside of TOML strings,
    // when the template is parsed,
    // then the parser will correctly determine which delimiters separate the frontmatter from the body.
    let template: Template = include_str!("./templates/succeeds/05_unambiguous_delims.template")
        .parse()
        .expect("Could not parse template file as template");

    let expected_requirements = hashset! {
        ExtensionRequirement {
            name: "beagle".to_string(),
            version: "+++\n".to_string(),
            helpers: vec![],
            ..Default::default()
        }, ExtensionRequirement {
            name: "std".to_string(),
            version: "v1".to_string(),
            helpers: vec!["join_map".to_string()],
            ..Default::default()
        }
    };
    let expected_body = r#"{{ join_map "=" "," "fail-if-missing" settings.beagle }}
"#
    .to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}

#[test]
fn succeeds_06_empty_frontmatter() {
    // Given a template with frontmatter delimiters inside of TOML strings,
    // when the template is parsed,
    // then the parser will correctly determine which delimiters separate the frontmatter from the body.
    let template: Template = include_str!("./templates/succeeds/06_empty_frontmatter.template")
        .parse()
        .expect("Could not parse template file as template");

    let expected_requirements = hashset! {};
    let expected_body = "Hello\n".to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}

#[test]
fn succeeds_07_aws_config() {
    // Given an existing template file,
    // when the template is parsed,
    // Then the template will be parsed correctly.
    let template: Template = include_str!("./templates/succeeds/07_aws-config.template")
        .parse()
        .expect("Could not parse template file as template");

    let expected_requirements = hashset! {
        ExtensionRequirement {
            name: "aws".to_string(),
            version: "v1".to_string(),
            helpers: vec![],
            ..Default::default()
        },
        ExtensionRequirement {
            name: "std".to_string(),
            version: "v1".to_string(),
            helpers: vec!["base64_decode".to_string()],
            ..Default::default()
        },
    };
    let expected_body =
        "{{~#if settings.aws.config~}}\n{{base64_decode settings.aws.config}}\n{{~/if~}}\n"
            .to_string();

    assert_eq!(
        template
            .frontmatter
            .extension_requirements()
            .collect::<HashSet<_>>(),
        expected_requirements
    );
    assert_eq!(template.body, expected_body);
}
