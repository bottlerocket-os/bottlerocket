use crate::error::{self, Result};

use chrono::{DateTime, Duration, FixedOffset, Utc};
use snafu::{ensure, ResultExt};

/// Parses a user-specified datetime, either in full RFC 3339 format, or a shorthand like "in 7
/// days"
pub(crate) fn parse_datetime(input: &str) -> Result<DateTime<Utc>> {
    // If the user gave an absolute date in a standard format, accept it.
    let try_dt: std::result::Result<DateTime<FixedOffset>, chrono::format::ParseError> =
        DateTime::parse_from_rfc3339(input);
    if let Ok(dt) = try_dt {
        let utc = dt.into();
        return Ok(utc);
    }

    // Otherwise, pull apart a request like "in 5 days" to get an exact datetime.
    let mut parts: Vec<&str> = input.split_whitespace().collect();
    ensure!(
        parts.len() == 3,
        error::DateArgInvalid {
            input,
            msg: "expected RFC 3339, or something like 'in 7 days'"
        }
    );
    let unit_str = parts.pop().unwrap();
    let count_str = parts.pop().unwrap();
    let prefix_str = parts.pop().unwrap();

    ensure!(
        prefix_str == "in",
        error::DateArgInvalid {
            input,
            msg: "expected RFC 3339, or prefix 'in', something like 'in 7 days'",
        }
    );

    let count: u32 = count_str.parse().context(error::DateArgCount { input })?;

    let duration = match unit_str {
        "hour" | "hours" => Duration::hours(i64::from(count)),
        "day" | "days" => Duration::days(i64::from(count)),
        "week" | "weeks" => Duration::weeks(i64::from(count)),
        _ => {
            return error::DateArgInvalid {
                input,
                msg: "date argument's unit must be hours/days/weeks",
            }
            .fail();
        }
    };

    let now = Utc::now();
    let then = now + duration;
    Ok(then)
}
