//! Compare complete chrony 4.8 source options, including their arguments.
//!
//! The arities below follow `CPS_ParseNTPSourceAdd` in chrony 4.8's cmdparse.c.
//! Both the target and rollback releases use that version. Unknown or ambiguous
//! input empties the shared projection. Names are case-insensitive, but arguments
//! match textually: numerically equivalent spellings are not normalized.

use serde_json::Value;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceOption {
    name: String,
    argument: Option<String>,
}

impl SourceOption {
    pub fn render(&self) -> String {
        match &self.argument {
            Some(argument) => format!("{} {argument}", self.name),
            None => self.name.clone(),
        }
    }
}

pub fn parse(value: Option<&Value>) -> Result<Vec<SourceOption>, &'static str> {
    let entries = match value {
        None => return Ok(Vec::new()),
        Some(Value::Array(entries)) => entries,
        _ => return Err("options is not an array"),
    };
    let strings = entries
        .iter()
        .map(|entry| entry.as_str().ok_or("option is not a string"))
        .collect::<Result<Vec<_>, _>>()?;
    let mut tokens = strings.iter().flat_map(|entry| entry.split_whitespace());
    let mut options: Vec<SourceOption> = Vec::new();
    while let Some(token) = tokens.next() {
        let name = token.to_ascii_lowercase();
        let argument = match name.as_str() {
            "certset" | "key" | "asymmetry" | "extfield" | "filter" | "maxdelay"
            | "maxdelayratio" | "maxdelaydevratio" | "maxdelayquant" | "maxpoll" | "maxsamples"
            | "maxsources" | "maxunreach" | "mindelay" | "minpoll" | "minsamples"
            | "minstratum" | "ntsport" | "offset" | "port" | "polltarget" | "presend"
            | "version" => Some(
                tokens
                    .next()
                    .ok_or("option is missing its argument")?
                    .to_owned(),
            ),
            "auto_offline" | "burst" | "copy" | "iburst" | "offline" | "ipv4" | "ipv6" | "nts"
            | "xleave" | "noselect" | "prefer" | "require" | "trust" => None,
            _ => return Err("unknown source option"),
        };
        let option = SourceOption { name, argument };
        if let Some(previous) = options.iter().find(|entry| entry.name == option.name) {
            if previous != &option {
                return Err("conflicting repeated source option");
            }
        } else {
            options.push(option);
        }
    }
    Ok(options)
}

pub fn common<'a>(values: impl IntoIterator<Item = Option<&'a Value>>) -> Vec<Value> {
    let mut shared: Option<Vec<SourceOption>> = None;
    for value in values {
        let options = match parse(value) {
            Ok(options) => options,
            Err(reason) => {
                println!("Cannot safely share NTP options on rollback: {reason}; writing []");
                return Vec::new();
            }
        };
        shared = Some(match shared {
            None => options,
            Some(previous) => previous
                .into_iter()
                .filter(|option| options.contains(option))
                .collect(),
        });
    }
    shared
        .unwrap_or_default()
        .into_iter()
        .map(|option| Value::String(option.render()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn intersection(left: Value, right: Value) -> Value {
        Value::Array(common([Some(&left), Some(&right)]))
    }

    #[test]
    fn complete_and_split_options_are_equivalent() {
        assert_eq!(
            intersection(
                json!(["minpoll", "4", "IBURST"]),
                json!(["minpoll 4", "iburst"])
            ),
            json!(["minpoll 4", "iburst"])
        );
    }

    #[test]
    fn repeated_numbers_stay_attached_to_their_option_names() {
        assert_eq!(
            intersection(
                json!(["minpoll 4", "maxpoll 4"]),
                json!(["minpoll", "4", "maxpoll", "6"])
            ),
            json!(["minpoll 4"])
        );
        assert_eq!(
            intersection(
                json!(["minpoll", "4", "maxpoll", "6"]),
                json!(["minpoll", "6", "maxpoll", "4"])
            ),
            json!([])
        );
    }

    #[test]
    fn malformed_unknown_and_conflicting_options_have_empty_projection() {
        for input in [
            json!(["minpoll"]),
            json!(["future-option", "iburst"]),
            json!(["minpoll 4", "minpoll 6"]),
            json!([4]),
        ] {
            assert_eq!(intersection(input, json!(["iburst"])), json!([]));
        }
    }

    #[test]
    fn identical_repeated_options_are_deduplicated() {
        assert_eq!(
            intersection(
                json!(["iburst", "minpoll 4", "iburst"]),
                json!(["minpoll", "4", "iburst"])
            ),
            json!(["iburst", "minpoll 4"])
        );
    }

    #[test]
    fn different_numeric_spellings_across_sources_drop_only_that_option() {
        assert_eq!(
            intersection(
                json!(["iburst", "minpoll 4"]),
                json!(["iburst", "minpoll 04"])
            ),
            json!(["iburst"])
        );
    }

    #[test]
    fn different_numeric_spellings_in_one_source_empty_the_projection() {
        assert_eq!(
            intersection(
                json!(["iburst", "minpoll 4", "minpoll 04"]),
                json!(["iburst", "minpoll 4"])
            ),
            json!([])
        );
    }

    #[test]
    fn missing_options_produce_an_explicit_empty_intersection() {
        let value = json!(["iburst"]);
        assert!(common([Some(&value), None]).is_empty());
    }

    #[test]
    fn output_is_a_complete_option_list() {
        let left = json!(["prefer", "minpoll", "4", "maxpoll 4", "iburst"]);
        let right = json!(["minpoll 4", "iburst"]);
        let output = Value::Array(common([Some(&left), Some(&right)]));
        assert_eq!(output, json!(["minpoll 4", "iburst"]));
        assert_eq!(parse(Some(&output)).unwrap().len(), 2);
    }
}
