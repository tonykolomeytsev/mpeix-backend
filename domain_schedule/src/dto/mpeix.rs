use std::fmt::Display;

use anyhow::bail;
use common_errors::errors::CommonError;
use domain_schedule_models::dto::v1::ScheduleType;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref VALID_GROUP_NAME_PATTERN: Regex = Regex::new(r#"[а-яА-Я0-9-]{5,20}"#).unwrap();
    static ref SHORTENED_GROUP_NAME_PATTERN: Regex = Regex::new(r#".*-\\d[^0-9]*-.*"#).unwrap();
    static ref VALID_PERSON_NAME_PATTERN: Regex = Regex::new(r#"([а-яА-Я]+\\s?){1,3}"#).unwrap();
    static ref SPACES_PATTERN: Regex = Regex::new(r#"\\s+"#).unwrap();
}

/// Type for schedule owner's name representation.
/// Contains only valid names inside.
#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub struct ScheduleName(String);

impl ScheduleName {
    /// Create valid schedule name from string.
    ///
    /// Name validation logic is inherited from kotlin backend.
    /// Maybe we should improve this algorithm.
    pub fn new(name: String, r#type: ScheduleType) -> anyhow::Result<Self> {
        match r#type {
            ScheduleType::Group => {
                if !VALID_GROUP_NAME_PATTERN.is_match(&name) {
                    bail!(CommonError::user("Invalid group name"));
                }
                let name = name.to_uppercase();
                if SHORTENED_GROUP_NAME_PATTERN.is_match(&name) {
                    Ok(Self(name.replacen('-', "-0", 1)))
                } else {
                    Ok(Self(name))
                }
            }
            ScheduleType::Person => {
                if !VALID_PERSON_NAME_PATTERN.is_match(&name) {
                    bail!(CommonError::user("Invalid person name"));
                }
                return Ok(Self(
                    SPACES_PATTERN
                        .split(&name.to_lowercase())
                        .map(capitalize_first_char)
                        .collect::<Vec<String>>()
                        .join(" "),
                ));
            }
            ScheduleType::Room => bail!(CommonError::internal(
                "Room name validation is not implemented yet"
            )),
        }
    }

    pub fn as_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for ScheduleName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for ScheduleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Solution taken from:
/// https://stackoverflow.com/questions/38406793/why-is-capitalizing-the-first-letter-of-a-string-so-convoluted-in-rust
fn capitalize_first_char(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

/// Type for schedule search query representation.
/// Contains only valid queries.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ScheduleSearchQuery(String);

impl ScheduleSearchQuery {
    /// Create valid search query from string.
    pub fn new(query: String) -> anyhow::Result<Self> {
        if query.len() < 3 {
            bail!(CommonError::user(
                "The search query must be 3 characters or more"
            ));
        }
        if query.len() > 50 {
            bail!(CommonError::user("Too long search query"));
        }
        let query = SPACES_PATTERN.replace_all(query.trim(), " ");
        if query.len() < 3 {
            bail!(CommonError::user(
                "The search query without trailing and leading spaces must be 3 characters or more"
            ));
        }
        Ok(Self(query.to_string()))
    }
}

impl AsRef<str> for ScheduleSearchQuery {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for ScheduleSearchQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
