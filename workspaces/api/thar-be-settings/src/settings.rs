use itertools::join;

use crate::client::ReqwestClientExt;
use crate::template::TemplateKeys;
use crate::{Result, API_SETTINGS_URI};

use apiserver::model;

/// Using the template registry, gather all keys and request
/// their values from the API
pub fn get_settings_from_template(
    client: &reqwest::Client,
    registry: &handlebars::Handlebars,
) -> Result<model::Settings> {
    // Using the template registry, pull the keys out of the templates
    // and query the API to get a structure of Settings which we can
    // use to render the templates
    debug!("Gathering keys from configuration file templates");
    let settings_to_query = registry.get_all_template_keys()?;

    debug!("Requesting settings values for template keys");
    let query = join(&settings_to_query, ",");

    // Query the settings
    debug!("Querying API for settings data");
    let settings: model::Settings =
        client.get_json(API_SETTINGS_URI.to_string(), "keys".to_string(), query)?;

    trace!("Settings values: {:?}", settings);
    Ok(settings)
}
