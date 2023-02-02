use chrono::{DateTime, Datelike, Days, Local, Month, NaiveDate, TimeZone, Weekday};
use domain_schedule_shift::{ScheduleShift, ShiftRule, ShiftedSemester, Year};
use std::cmp::Ordering;

pub trait DateTimeExt {
    fn with_days_offset(self, offset: i32) -> Option<Self>
    where
        Self: Sized;
}

impl<Tz: TimeZone> DateTimeExt for DateTime<Tz> {
    fn with_days_offset(self, offset: i32) -> Option<Self> {
        match offset.cmp(&0) {
            Ordering::Equal => Some(self),
            Ordering::Greater => self.checked_add_days(Days::new(offset as u64)),
            Ordering::Less => self.checked_sub_days(Days::new(-offset as u64)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum WeekOfSemester {
    Studying(u8),
    NonStudying,
}

pub trait NaiveDateExt {
    fn week_of_year(self) -> u8
    where
        Self: Sized;

    /// Get number of semester week.
    ///
    /// Important tenets:
    /// - The first day of the fall semester is always the 1th of September, unless
    ///   the 1th of September is Sunday.
    /// - The first day of the spring semester is always the first Monday of February.
    ///
    /// The maximum number of weeks in a
    /// semester is 18. Summer time is from July to August. For this date range,
    /// and for school weeks greater than 18, -1 will be returned.
    fn week_of_semester(self, shifts: Option<&ScheduleShift>) -> Option<WeekOfSemester>
    where
        Self: Sized;

    fn is_past_week(&self) -> bool
    where
        Self: Sized;
}

impl NaiveDateExt for NaiveDate {
    fn week_of_year(self) -> u8 {
        self.iso_week().week() as u8
    }

    fn week_of_semester(self, shifts: Option<&ScheduleShift>) -> Option<WeekOfSemester> {
        if let (1, 1..=7) = (self.month(), self.day()) {
            // January weekend
            return Some(WeekOfSemester::NonStudying);
        }
        vec![ShiftedSemester::Spring, ShiftedSemester::Fall]
            .into_iter()
            .filter_map(|semester| get_first_day_and_week_number(&self, shifts, semester))
            .filter_map(|(first_day, week_number)| {
                let has_zero_week = week_number == 0;
                let result =
                    self.week_of_year() as i8 - first_day.week_of_year() as i8 + week_number;
                match (result, has_zero_week) {
                    (0..=17, true) => Some(result as u8),
                    (1..=17, false) => Some(result as u8),
                    _ => None,
                }
            })
            .min()
            .map(WeekOfSemester::Studying)
            .or(Some(WeekOfSemester::NonStudying))
    }

    fn is_past_week(&self) -> bool {
        self.checked_add_days(Days::new(6))
            .filter(|it| it < &Local::now().naive_local().date())
            .is_some()
    }
}

fn get_first_day_and_week_number(
    now: &NaiveDate,
    shifts: Option<&ScheduleShift>,
    semester: ShiftedSemester,
) -> Option<(NaiveDate, i8)> {
    // look for 'shift' rule for this semester
    // in case the first study day is determined by non-standard rules
    let shift_rule_for_semester =
        shifts.and_then(|it| it.get(Year::new(now.year()), semester.clone()));

    if let Some(ShiftRule {
        first_day,
        week_number,
    }) = shift_rule_for_semester
    {
        // default number for first study week is 1, but we can provide any
        Some((*first_day, week_number.unwrap_or(1)))
    } else {
        let first_day = match semester {
            // first of September if it is not Sunday, either 2nd of September
            ShiftedSemester::Fall => {
                let first_of_september =
                    NaiveDate::from_ymd_opt(now.year(), Month::September.number_from_month(), 1)?;
                if matches!(first_of_september.weekday(), Weekday::Sun) {
                    // return 2nd of September (Monday)
                    NaiveDate::from_ymd_opt(now.year(), Month::September.number_from_month(), 2)?
                } else {
                    first_of_september
                }
            }
            // first monday of February
            ShiftedSemester::Spring => NaiveDate::from_weekday_of_month_opt(
                now.year(),
                Month::February.number_from_month(),
                Weekday::Mon,
                1,
            )?,
        };
        Some((first_day, 1))
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Display, str::FromStr};

    use chrono::{Days, Month, NaiveDate};
    use domain_schedule_shift::ScheduleShift;
    use lazy_static::lazy_static;

    use crate::time::NaiveDateExt;

    use super::WeekOfSemester;

    macro_rules! test_week_of_semester {
        ($name:tt, date = ($y:expr, $m:expr, $d:expr), shift = $sh:expr, result = $res:expr) => {
            #[test]
            fn $name() {
                let date = NaiveDate::from_ymd_opt($y, $m.number_from_month(), $d).unwrap();
                assert_eq!(date.week_of_semester($sh), Some($res));
            }
        };
    }

    test_week_of_semester!(
        september_1st_2019_without_shifts,
        date = (2019, Month::September, 1),
        shift = None,
        result = WeekOfSemester::NonStudying
    );

    test_week_of_semester!(
        september_2nd_2019_without_shifts,
        date = (2019, Month::September, 2),
        shift = None,
        result = WeekOfSemester::Studying(1)
    );

    test_week_of_semester!(
        february_1st_2020_without_shifts,
        date = (2020, Month::February, 1),
        shift = None,
        result = WeekOfSemester::NonStudying
    );

    test_week_of_semester!(
        february_2nd_2020_without_shifts,
        date = (2020, Month::February, 2),
        shift = None,
        result = WeekOfSemester::NonStudying
    );

    test_week_of_semester!(
        february_3rd_2020_without_shifts,
        date = (2020, Month::February, 3),
        shift = None,
        result = WeekOfSemester::Studying(1)
    );

    test_week_of_semester!(
        august_30th_2020_without_shifts,
        date = (2020, Month::August, 30),
        shift = None,
        result = WeekOfSemester::NonStudying
    );

    test_week_of_semester!(
        august_31th_2020_without_shifts,
        date = (2020, Month::August, 31),
        shift = None,
        result = WeekOfSemester::Studying(1)
    );

    test_week_of_semester!(
        september_1st_2020_without_shifts,
        date = (2020, Month::September, 1),
        shift = None,
        result = WeekOfSemester::Studying(1)
    );

    test_week_of_semester!(
        september_6th_2020_without_shifts,
        date = (2020, Month::September, 6),
        shift = None,
        result = WeekOfSemester::Studying(1)
    );

    test_week_of_semester!(
        september_7th_2020_without_shifts,
        date = (2020, Month::September, 7),
        shift = None,
        result = WeekOfSemester::Studying(2)
    );

    test_week_of_semester!(
        february_1st_2021_without_shifts,
        date = (2021, Month::February, 1),
        shift = None,
        result = WeekOfSemester::Studying(1)
    );

    lazy_static! {
        static ref TEST_SHIFTS: ScheduleShift = ScheduleShift::from_str(
            r#"
            [2021]
            spring = { first-day = "2021-02-15", week-number = 0 }
            fall = { first-day = "2021-09-06", week-number = 2 }

            [2022]
            spring = { first-day = "2022-02-16" }
            fall = { first-day = "2022-09-16" }
            "#
        )
        .unwrap();
    }

    test_week_of_semester!(
        february_1st_2021_with_shift,
        date = (2021, Month::February, 1),
        shift = Some(&TEST_SHIFTS),
        result = WeekOfSemester::NonStudying
    );

    test_week_of_semester!(
        february_8th_2021_with_shift,
        date = (2021, Month::February, 8),
        shift = Some(&TEST_SHIFTS),
        result = WeekOfSemester::NonStudying
    );

    test_week_of_semester!(
        february_15th_2021_with_shift,
        date = (2021, Month::February, 15),
        shift = Some(&TEST_SHIFTS),
        result = WeekOfSemester::Studying(0)
    );

    test_week_of_semester!(
        september_1st_2021_with_shift,
        date = (2021, Month::September, 1),
        shift = Some(&TEST_SHIFTS),
        result = WeekOfSemester::Studying(1)
    );

    test_week_of_semester!(
        september_6th_2021_with_shift,
        date = (2021, Month::September, 6),
        shift = Some(&TEST_SHIFTS),
        result = WeekOfSemester::Studying(2)
    );

    test_week_of_semester!(
        february_14th_2022_with_shift,
        date = (2022, Month::February, 14),
        shift = Some(&TEST_SHIFTS),
        result = WeekOfSemester::Studying(1)
    );

    test_week_of_semester!(
        february_16th_2022_with_shift,
        date = (2022, Month::February, 16),
        shift = Some(&TEST_SHIFTS),
        result = WeekOfSemester::Studying(1)
    );

    #[test]
    fn test_all_days_from_2019_to_2025() {
        let mut date =
            NaiveDate::from_ymd_opt(2019, Month::January.number_from_month(), 1).unwrap();
        let end_date =
            NaiveDate::from_ymd_opt(2025, Month::January.number_from_month(), 2).unwrap();
        let mut state = WeekOfSemester::NonStudying;

        while date != end_date {
            let new_state = date.week_of_semester(Some(&TEST_SHIFTS));
            assert!(new_state.is_some());
            let new_state = new_state.unwrap();

            match (&state, &new_state) {
                (WeekOfSemester::NonStudying, WeekOfSemester::Studying(i)) => {
                    print!("{date}: {i} -> ");
                }
                (WeekOfSemester::Studying(i), WeekOfSemester::NonStudying) => {
                    println!("{i}: {date}");
                }
                _ => (),
            }
            state = new_state;

            date = date.checked_add_days(Days::new(1)).unwrap();
        }
    }

    impl Display for WeekOfSemester {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Self::NonStudying => write!(f, "not-studying"),
                Self::Studying(i) => write!(f, "studying({i})"),
            }
        }
    }
}
