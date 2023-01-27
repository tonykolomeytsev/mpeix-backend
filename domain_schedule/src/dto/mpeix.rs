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
                "Room name validation not implemented"
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

impl ToString for ScheduleName {
    fn to_string(&self) -> String {
        self.0.to_owned()
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
