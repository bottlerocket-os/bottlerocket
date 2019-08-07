use itertools::join;
use std::path::Path;

use crate::client;
use crate::template::TemplateKeys;
use crate::Result;

use apiserver::model;

/// Using the template registry, gather all keys and request
/// their values from the API
pub fn get_settings_from_template<P>(
    socket_path: P,
    registry: &handlebars::Handlebars,
) -> Result<model::Settings>
where
    P: AsRef<Path>,
{
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
        client::get_json(socket_path, "/settings", Some(("keys", query)))?;

    trace!("Settings values: {:?}", settings);
    Ok(settings)
}
