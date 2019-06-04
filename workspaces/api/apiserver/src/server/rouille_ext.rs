//! This module extends the rouille web server in various helpful ways.

use serde::Serialize;
use std::error::Error;

/// This is like rouille's ErrJson, in that we use it to serialize errors for a rouille Response,
/// but has these changes:
/// * err.description replaced with a Display of err - description() is deprecated and produces
/// unhelpful strings
/// * 'cause' removed, because we include source error information in the description, via snafu.
#[derive(Serialize)]
pub(crate) struct ErrJson {
    description: String,
}
impl<'a> ErrJson {
    pub(crate) fn from_err<E: ?Sized + Error>(err: &'a E) -> ErrJson {
        ErrJson {
            description: format!("{}", err),
        }
    }
}

/// This is rouille's try_or_400, but you can configure the response code.
macro_rules! try_or {
    ($code:expr, $result:expr) => {
        match $result {
            Ok(r) => r,
            Err(err) => {
                let json = $crate::server::rouille_ext::ErrJson::from_err(&err);
                return rouille::Response::json(&json).with_status_code($code);
            }
        }
    };
}
