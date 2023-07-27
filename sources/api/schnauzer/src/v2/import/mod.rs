pub mod settings;

/// Utility that Boxes an error type to be returned by a generic trait interface.
///
/// Intended to be called as e.g. `fallible().map_err(as_std_err)`
fn as_std_err<'a, E: std::error::Error + 'a>(err: E) -> Box<dyn std::error::Error + 'a> {
    Box::new(err) as Box<dyn std::error::Error>
}
