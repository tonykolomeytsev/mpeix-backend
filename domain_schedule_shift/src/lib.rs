use std::{collections::HashMap, path::Path};

use chrono::NaiveDate;
use tokio::{fs::File, io::AsyncReadExt};
use toml::Table;

#[derive(Debug, PartialEq, Eq)]
pub struct ScheduleShift(HashMap<(Year, ShiftedSemester), ShiftRule>);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Year(u16);

impl Year {
    pub fn new(year: u16) -> Year {
        Self(year)
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum ShiftedSemester {
    Spring,
    Fall,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ShiftRule {
    /// Points to first study day of semester
    first_day: NaiveDate,
    /// Number of the academic week with the first study day
    week_number: Option<i8>,
}

const SEMESTERS: &[&str] = &["spring", "fall"];

impl ScheduleShift {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let mut file = File::create(path).await?;
        let mut serialized_value = String::with_capacity(4096);
        file.read_to_string(&mut serialized_value).await?;
        ScheduleShift::from_str(serialized_value).await
    }

    pub async fn from_str<S: AsRef<str>>(string: S) -> anyhow::Result<Self> {
        let shifts_table = string.as_ref().parse::<Table>()?;

        let mut rules_map = HashMap::new();
        for (year, semester_rule) in shifts_table {
            let year = Year::new(year.parse::<u16>()?);
            for semester in SEMESTERS {
                if let Some(rule) = semester_rule.get(semester) {
                    let semester = ShiftedSemester::Spring;
                    let first_day = rule
                        .get("first-day")
                        .and_then(|it| it.as_str())
                        .and_then(|it| NaiveDate::parse_from_str(it, "%Y-%m-%d").ok());
                    let week_number = rule
                        .get("week-number")
                        .and_then(|it| it.as_integer())
                        .map(|it| it as i8);
                    if let Some(first_day) = first_day {
                        rules_map.insert(
                            (year.clone(), semester),
                            ShiftRule {
                                first_day,
                                week_number,
                            },
                        );
                    }
                }
            }
        }
        Ok(Self(rules_map))
    }

    pub fn get(&self, year: Year, semester: ShiftedSemester) -> Option<&ShiftRule> {
        self.0.get(&(year, semester))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::NaiveDate;

    use crate::{ScheduleShift, ShiftRule, ShiftedSemester, Year};

    macro_rules! await_blocking {
        { $e:expr } => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn from_str_valid_test() {
        let toml_content = include_str!("../res/schedule_shift_example.toml");
        let shift = await_blocking!(ScheduleShift::from_str(toml_content));
        assert!(shift.is_ok());
        assert_eq!(
            ScheduleShift(HashMap::from([(
                (Year::new(2021), ShiftedSemester::Spring),
                ShiftRule {
                    first_day: NaiveDate::from_ymd_opt(2021, 2, 15).unwrap(),
                    week_number: Some(0),
                }
            )])),
            shift.unwrap(),
        );
    }
}
