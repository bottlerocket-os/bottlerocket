//! This module extends the rouille web server in various helpful ways.

use serde::Serialize;
use std::error::Error;

/// This is rouille's ErrJson, except with err.description replaced with a Display of err;
/// description() is deprecated and produces unhelpful strings.  (It still uses the deprecated
/// cause() because its replacement, source(), doesn't return the underlying error properly.)
#[derive(Serialize)]
pub(crate) struct ErrJson {
    description: String,
    cause: Option<Box<ErrJson>>,
}
impl<'a> ErrJson {
    pub(crate) fn from_err<E: ?Sized + Error>(err: &'a E) -> ErrJson {
        #[allow(deprecated)] // cause(); source returns nothing?
        let cause = err.cause().map(ErrJson::from_err).map(Box::new);
        ErrJson {
            description: format!("{}", err),
            cause,
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
