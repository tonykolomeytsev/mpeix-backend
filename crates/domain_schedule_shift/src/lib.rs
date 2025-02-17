use std::{collections::HashMap, fmt::Display, path::Path, str::FromStr};

use anyhow::{bail, ensure};
use chrono::{Datelike, NaiveDate};
use tokio::{fs::File, io::AsyncReadExt};
use toml::Table;

/// Structure, remembering exceptions to the rules in the numbering of academic weeks.
/// Used in `domain_schedule` crate to make it easier to work with non-standard study week numbers.
#[derive(Debug, PartialEq, Eq)]
pub struct ScheduleShift(HashMap<(Year, ShiftedSemester), ShiftRule>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Year(i32);

impl Year {
    pub fn new(year: i32) -> Year {
        Self(year)
    }
}

/// Type of semester with non-standard study week numbers
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ShiftedSemester {
    Spring,
    Fall,
}

/// A rule that describes the shift in the numbers of academic weeks of a particular semester
#[derive(Debug, PartialEq, Eq)]
pub struct ShiftRule {
    /// Points to first study day of semester
    pub first_day: NaiveDate,
    /// Number of the academic week with the first study day
    pub week_number: Option<i8>,
}

const SEMESTERS: &[ShiftedSemester] = &[ShiftedSemester::Spring, ShiftedSemester::Fall];

impl ScheduleShift {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mut file = File::open(path).await?;
        let mut serialized_value = String::with_capacity(4096);
        file.read_to_string(&mut serialized_value).await?;
        ScheduleShift::from_str(&serialized_value)
    }

    /// Get shift rule for specified year and semester
    pub fn get(&self, year: Year, semester: ShiftedSemester) -> Option<&ShiftRule> {
        self.0.get(&(year, semester))
    }
}

impl FromStr for ScheduleShift {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> anyhow::Result<Self> {
        let shifts_table = string.parse::<Table>()?;

        let mut rules_map = HashMap::new();
        for (year, semester_rule) in shifts_table {
            let year = Year::new(year.parse()?);
            for semester in SEMESTERS {
                if let Some(rule) = semester_rule.get(semester.to_string()) {
                    let first_day = rule
                        .get("first-day")
                        .and_then(|it| it.as_str())
                        .and_then(|it| NaiveDate::parse_from_str(it, "%Y-%m-%d").ok());
                    let week_number = rule
                        .get("week-number")
                        .and_then(|it| it.as_integer())
                        .map(|it| it as i8);

                    if let Some(first_day) = first_day {
                        ensure!(
                            first_day.year() == year.0,
                            format!(
                                "Shift rule for {year} {semester} semester has field 'first-day' with wrong year specified: '{first_day}'"
                            )
                        );

                        rules_map.insert(
                            (year.clone(), semester.clone()),
                            ShiftRule {
                                first_day,
                                week_number,
                            },
                        );
                    } else {
                        bail!("Invalid shift rule for {year} {semester} semester: the 'first-day' field is required")
                    }
                }
            }
        }
        Ok(Self(rules_map))
    }
}

impl Display for ShiftedSemester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Spring => write!(f, "spring"),
            Self::Fall => write!(f, "fall"),
        }
    }
}

impl Display for Year {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, str::FromStr};

    use chrono::NaiveDate;

    use crate::{ScheduleShift, ShiftRule, ShiftedSemester, Year};

    #[test]
    fn from_str_valid_test() {
        let toml_content = include_str!("../res/default_schedule_shift.toml");
        let shift = ScheduleShift::from_str(toml_content);
        assert!(shift.is_ok());
        assert_eq!(
            ScheduleShift(HashMap::from([
                (
                    (Year::new(2021), ShiftedSemester::Spring),
                    ShiftRule {
                        first_day: NaiveDate::from_ymd_opt(2021, 2, 15).unwrap(),
                        week_number: Some(0),
                    }
                ),
                (
                    (Year::new(2023), ShiftedSemester::Spring),
                    ShiftRule {
                        first_day: NaiveDate::from_ymd_opt(2023, 2, 8).unwrap(),
                        week_number: Some(0),
                    }
                ),
                (
                    (Year::new(2025), ShiftedSemester::Spring),
                    ShiftRule {
                        first_day: NaiveDate::from_ymd_opt(2025, 2, 10).unwrap(),
                        week_number: Some(1),
                    }
                )
            ])),
            shift.unwrap(),
        );
    }

    #[test]
    fn from_str_invalid_year_test() {
        let toml_content = r#"
        [2022]
        fall = { first-day = "2021-09-16" }
        "#;
        let shift = ScheduleShift::from_str(toml_content);
        assert!(shift.is_err());
    }

    #[test]
    fn from_str_invalid_first_day_test() {
        let toml_content = r#"
        [2022]
        fall = { week-number = 0 }
        "#;
        let shift = ScheduleShift::from_str(toml_content);
        assert!(shift.is_err());
    }
}
