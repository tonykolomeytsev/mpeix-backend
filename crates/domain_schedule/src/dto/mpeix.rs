use std::fmt::Display;

use anyhow::bail;
use common_errors::errors::CommonError;
use domain_schedule_models::ScheduleType;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref VALID_GROUP_NAME_PATTERN: Regex = Regex::new(r#"[а-яА-Я0-9-]{5,20}"#).unwrap();
    static ref SHORTENED_GROUP_NAME_PATTERN: Regex = Regex::new(r".*-\d[^0-9]*-.*").unwrap();
    static ref VALID_PERSON_NAME_PATTERN: Regex = Regex::new(r"([а-яА-Я]+(\s|[-])?){1,5}").unwrap();
    static ref SPACES_PATTERN: Regex = Regex::new(r"\s+").unwrap();
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
                Ok(Self(name))
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

/// Type for schedule search query representation.
/// Contains only valid queries.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct ScheduleSearchQuery(String);

const MIN_QUERY_LENGTH: usize = 2;

impl ScheduleSearchQuery {
    /// Create valid search query from string.
    pub fn new(query: String) -> anyhow::Result<Self> {
        let length = query.chars().count();
        if length < MIN_QUERY_LENGTH {
            bail!(CommonError::user(format!(
                "The search query must be {MIN_QUERY_LENGTH} characters or more"
            )));
        }
        if length > 50 {
            bail!(CommonError::user("Too long search query"));
        }
        let query = SPACES_PATTERN.replace_all(query.trim(), " ");

        let length = query.chars().count();
        if length < MIN_QUERY_LENGTH {
            bail!(CommonError::user(format!(
                "The search query without trailing and leading spaces must be {MIN_QUERY_LENGTH} characters or more"
            )));
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

#[cfg(test)]
mod tests {
    use domain_schedule_models::ScheduleType;

    use super::{ScheduleName, ScheduleSearchQuery};

    #[test]
    fn test_valid_group_names() {
        assert!(ScheduleName::new("С-12-16".to_string(), ScheduleType::Group).is_ok());
        assert!(ScheduleName::new("Сэ-12-21".to_string(), ScheduleType::Group).is_ok());
        assert!(ScheduleName::new("А-08М-22".to_string(), ScheduleType::Group).is_ok());
    }

    #[test]
    fn test_valid_person_names() {
        assert!(
            ScheduleName::new("Адамов Борис Игоревич".to_string(), ScheduleType::Person).is_ok()
        );
        assert!(ScheduleName::new(
            "Кули-Заде Турал Аладдинович".to_string(),
            ScheduleType::Person
        )
        .is_ok());
        assert!(ScheduleName::new("Иванко Влада".to_string(), ScheduleType::Person).is_ok());
    }

    #[test]
    fn test_valid_search_query() {
        assert!(ScheduleSearchQuery::new("abcdef".to_string()).is_ok());
        assert!(ScheduleSearchQuery::new("12345".to_string()).is_ok());
        assert!(ScheduleSearchQuery::new("Куликова".to_string()).is_ok());
    }

    #[test]
    fn test_invalid_search_query() {
        assert!(ScheduleSearchQuery::new("К".to_string()).is_err());
        assert!(ScheduleSearchQuery::new("  К  ".to_string()).is_err());
        assert!(ScheduleSearchQuery::new(
            "123456789_123456789_123456789_123456789_123456789_1".to_string()
        )
        .is_err());
    }
}
