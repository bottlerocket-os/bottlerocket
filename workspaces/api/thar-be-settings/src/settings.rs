use std::path::Path;

use crate::client;
use crate::Result;

/// Using the template registry, gather all keys and request
/// their values from the API
pub fn get_settings_from_template<P>(
    socket_path: P,
) -> Result<model::Settings>
where
    P: AsRef<Path>,
{
    debug!("Querying API for settings data");
    let settings: model::Settings =
        client::get_json(socket_path, "/settings", None as Option<(String, String)>)?;

    trace!("Settings values: {:?}", settings);
    Ok(settings)
}
