use chrono::{DateTime, Datelike, Days, Local, Month, NaiveDate, TimeZone, Weekday};
use num_traits::FromPrimitive;
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

pub enum WeekOfSemester {
    Studying(u8),
    NonStudying,
}

pub trait NaiveDateExt {
    fn week_of_year(self) -> u8
    where
        Self: Sized;

    /// The first day of the fall semester is always the 1th of September, unless
    /// the 1th of September is Sunday. The first day of the spring semester is
    /// always the first Monday of February. The maximum number of weeks in a
    /// semester is 18. Summer time is from July to August. For this date range,
    /// and for school weeks greater than 18, -1 will be returned.
    fn week_of_semester(self) -> Option<WeekOfSemester>
    where
        Self: Sized;

    fn is_past_week(&self) -> bool
    where
        Self: Sized;
}

impl NaiveDateExt for NaiveDate {
    fn week_of_year(self) -> u8
    where
        Self: Sized,
    {
        self.iso_week().week() as u8
    }

    fn week_of_semester(self) -> Option<WeekOfSemester>
    where
        Self: Sized,
    {
        let week_of_year = match Month::from_u32(self.month()) {
            Some(Month::July | Month::January) => return Some(WeekOfSemester::NonStudying),
            Some(Month::February | Month::March | Month::April | Month::May | Month::June) => {
                let spr_sem_start = if self.year() == 2021 { 15 } else { 1 }; // todo: DI
                let first_of_february = self
                    .with_month(Month::February.number_from_month())
                    .and_then(|d| d.with_day(spr_sem_start))?;
                // if 1th of February is Monday, return it's week number, else return next week number
                if matches!(first_of_february.weekday(), Weekday::Mon) {
                    first_of_february.week_of_year()
                } else {
                    first_of_february.week_of_year() + 1
                }
            }
            _ => {
                let first_of_september = self
                    .with_month(Month::September.number_from_month())
                    .and_then(|d| d.with_day(1))?;
                // if 1th of September is not Sunday, it's a working day, then return it's week number
                // else return next week number
                if !matches!(first_of_september.weekday(), Weekday::Sun) {
                    first_of_september.week_of_year()
                } else {
                    first_of_september.week_of_year() + 1
                }
            }
        };
        let result = self.week_of_year() - week_of_year + 1;
        match result {
            0..=17 => Some(WeekOfSemester::Studying(result)),
            _ => Some(WeekOfSemester::NonStudying),
        }
    }

    fn is_past_week(&self) -> bool
    where
        Self: Sized,
    {
        self.checked_add_days(Days::new(6))
            .filter(|it| it < &Local::now().naive_local().date())
            .is_some()
    }
}
