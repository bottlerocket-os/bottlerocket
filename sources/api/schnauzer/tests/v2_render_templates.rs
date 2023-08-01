use handlebars::{Context, Handlebars, Helper, Output, RenderContext, RenderError};
use schnauzer::v2::fake::FakeImporter;
use serde_json::json;

#[tokio::test]
async fn render_00_basictemplate() {
    let template = include_str!("./templates/succeeds/00_basic.template");

    let expected_render = include_str!("./templates/succeeds_rendered/00_basic.rendered");

    let importer = FakeImporter::new(
        json!({
            "settings": {
                "motd": "Bottlerocket!",
            },
        }),
        vec![],
    );

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}

#[tokio::test]
async fn render_01_loose_frontmatter_whitespace() {
    let template = include_str!("./templates/succeeds/01_loose_frontmatter_whitespace.template");

    let expected_render =
        include_str!("./templates/succeeds_rendered/01_loose_frontmatter_whitespace.rendered");

    let importer = FakeImporter::new(json!({}), vec![]);

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}

#[tokio::test]
async fn render_02_strict_body_whitespace() {
    let template = include_str!("./templates/succeeds/02_strict_body_whitespace.template");

    let expected_render =
        include_str!("./templates/succeeds_rendered/02_strict_body_whitespace.rendered");

    let importer = FakeImporter::new(json!({}), vec![]);

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}

#[tokio::test]
async fn render_03_at_least_3_delims() {
    let template = include_str!("./templates/succeeds/03_at_least_3_delims.template");

    let expected_render =
        include_str!("./templates/succeeds_rendered/03_at_least_3_delims.rendered");

    let importer = FakeImporter::new(
        json!({
            "settings": {
                "extension": {
                    "attribute": "configured"
                },
            },
        }),
        vec![],
    );

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}

fn labrador_helper(
    _: &Helper<'_, '_>,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext<'_, '_>,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    out.write("bark bork ruff").unwrap();
    Ok(())
}

#[tokio::test]
async fn render_04_comments() {
    let template = include_str!("./templates/succeeds/04_comments.template");

    let expected_render = include_str!("./templates/succeeds_rendered/04_comments.rendered");

    let importer = FakeImporter::new(json!({}), vec![("woof", labrador_helper)]);

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}

#[tokio::test]
async fn render_05_unambiguous_delims() {
    let template = include_str!("./templates/succeeds/05_unambiguous_delims.template");

    let expected_render =
        include_str!("./templates/succeeds_rendered/05_unambiguous_delims.rendered");

    let importer = FakeImporter::new(
        json!({
            "settings": {
                "beagle": {
                    "howl": "awoo"
                }
            }
        }),
        vec![("join_map", schnauzer::helpers::join_map)],
    );

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}

#[tokio::test]
async fn render_06_empty_frontmatter() {
    let template = include_str!("./templates/succeeds/06_empty_frontmatter.template");

    let expected_render =
        include_str!("./templates/succeeds_rendered/06_empty_frontmatter.rendered");

    let importer = FakeImporter::new(json!({}), vec![]);

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}

#[tokio::test]
async fn render_07_aws_config() {
    let template = include_str!("./templates/succeeds/07_aws-config.template");

    let expected_render = include_str!("./templates/succeeds_rendered/07_aws-config.rendered");

    let importer = FakeImporter::new(
        json!({
            "settings": {
                "aws": {
                    "config": "c2V0dGluZ3MK"
                }
            }
        }),
        vec![("base64_decode", schnauzer::helpers::base64_decode)],
    );

    assert_eq!(
        schnauzer::v2::render_template_str(&importer, template)
            .await
            .unwrap(),
        expected_render
    );
}
