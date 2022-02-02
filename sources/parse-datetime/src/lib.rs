/*!
# Background

This library parses a `DateTime<Utc>` from a string.

The string can be:

* an `RFC3339` formatted date / time
* a string with the form `"[in] <unsigned integer> <unit(s)>"` where 'in' is optional
   * `<unsigned integer>` may be any unsigned integer and
   * `<unit(s)>` may be either the singular or plural form of the following: `hour | hours`, `day | days`, `week | weeks`

Examples:

* `"in 1 hour"`
* `"in 2 hours"`
* `"in 6 days"`
* `"in 2 weeks"`
* `"1 hour"`
* `"7 days"`
*/

use chrono::{DateTime, Duration, FixedOffset, Utc};
use snafu::{ensure, ResultExt};

mod error {
    use snafu::Snafu;

    #[derive(Debug, Snafu)]
    #[snafu(visibility(pub(super)))]
    pub enum Error {
        #[snafu(display("Date argument '{}' is invalid: {}", input, msg))]
        DateArgInvalid { input: String, msg: &'static str },

        #[snafu(display(
            "Date argument had count '{}' that failed to parse as integer: {}",
            input,
            source
        ))]
        DateArgCount {
            input: String,
            source: std::num::ParseIntError,
        },
    }
}
pub use error::Error;
type Result<T> = std::result::Result<T, error::Error>;

/// Parses a user-specified datetime, either in full RFC 3339 format, or a shorthand like "in 7
/// days" that's taken as an offset from the time the function is run.
pub fn parse_datetime(input: &str) -> Result<DateTime<Utc>> {
    // If the user gave an absolute date in a standard format, accept it.
    let try_dt: std::result::Result<DateTime<FixedOffset>, chrono::format::ParseError> =
        DateTime::parse_from_rfc3339(input);
    if let Ok(dt) = try_dt {
        let utc = dt.into();
        return Ok(utc);
    }

    let offset = parse_offset(input)?;

    let now = Utc::now();
    let then = now + offset;
    Ok(then)
}

/// Parses a user-specified datetime offset in the form of a shorthand like "in 7 days".
pub fn parse_offset(input: &str) -> Result<Duration> {
    // Otherwise, pull apart a request like "in 5 days" to get an exact datetime.
    let mut parts: Vec<&str> = input.split_whitespace().collect();
    ensure!(
        parts.len() == 3 || parts.len() == 2,
        error::DateArgInvalidSnafu {
            input,
            msg: "expected RFC 3339, or something like 'in 7 days' or '7 days'"
        }
    );
    let unit_str = parts.pop().unwrap();
    let count_str = parts.pop().unwrap();

    // the prefix string 'in' is optional
    if let Some(prefix_str) = parts.pop() {
        ensure!(
            prefix_str == "in",
            error::DateArgInvalidSnafu {
                input,
                msg: "expected prefix 'in', something like 'in 7 days'",
            }
        );
    }

    let count: u32 = count_str
        .parse()
        .context(error::DateArgCountSnafu { input })?;

    let duration = match unit_str {
        "hour" | "hours" => Duration::hours(i64::from(count)),
        "day" | "days" => Duration::days(i64::from(count)),
        "week" | "weeks" => Duration::weeks(i64::from(count)),
        _ => {
            return error::DateArgInvalidSnafu {
                input,
                msg: "date argument's unit must be hours/days/weeks",
            }
            .fail();
        }
    };

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptable_strings() {
        let inputs = vec![
            "in 0 hours",
            "in 1 hour",
            "in 5000000 hours",
            "in 0 days",
            "in 1 day",
            "in 5000000 days",
            "in 0 weeks",
            "in 1 week",
            "in 5000000 weeks",
            "0 weeks",
            "1 week",
            "5000000 weeks",
        ];

        for input in inputs {
            assert!(parse_datetime(input).is_ok())
        }
    }

    #[test]
    fn test_unacceptable_strings() {
        let inputs = vec!["in", "0 hou", "hours", "in 1 month"];

        for input in inputs {
            assert!(parse_datetime(input).is_err())
        }
    }
}
