//! The proleptic Gregorian calendar -- the anchor, not a peer.
//!
//! Hodesh exists partly to get away from this calendar, and it is implemented
//! here anyway, for one reason: it is what every existing system, record and
//! human already uses. A hodesh date nobody can turn into a Gregorian one is
//! a date nobody can act on.
//!
//! *Proleptic* means the Gregorian rules are run backwards past 1582 rather
//! than switching to Julian at the reform. That makes the calendar a single
//! uniform rule over all of history, which is what a conversion target needs
//! to be; it also means a date before 1582 here is not the date a contemporary
//! would have written. The Julian calendar is a separate calendar and belongs
//! in its own module if it is ever wanted.
//!
//! There is no year zero in the common era, but there is one here: year 0 is
//! 1 BCE, year -1 is 2 BCE, and so on. Arithmetic that special-cases a missing
//! zero is arithmetic that is wrong somewhere.

use std::fmt;
use std::str::FromStr;

use hodesh_types::{CalendarDate, DateError, FixedDay, HodeshCode};

/// The calendar's name, as it appears in a [`DateError`].
pub const CALENDAR: &str = "gregorian";

const MONTH_LENGTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// A date in the proleptic Gregorian calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GregorianDate {
    year: i64,
    month: u8,
    day: u8,
}

impl GregorianDate {
    /// A date from its parts, checked.
    pub fn new(year: i64, month: u8, day: u8) -> Result<GregorianDate, DateError> {
        if !(1..=12).contains(&month) {
            return Err(DateError::new(
                HodeshCode::MonthOutOfRange,
                CALENDAR,
                format!("month {month} does not exist; the Gregorian year has 12 months"),
            ));
        }
        let last = days_in_month(year, month);
        if day < 1 || day > last {
            return Err(DateError::new(
                HodeshCode::DayOutOfRange,
                CALENDAR,
                format!(
                    "day {day} does not exist in month {month} of {year}, which has {last} days"
                ),
            ));
        }
        Ok(GregorianDate { year, month, day })
    }

    /// The year. Year 0 is 1 BCE.
    pub fn year(self) -> i64 {
        self.year
    }

    /// The month, `1..=12`.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, `1..=31`.
    pub fn day(self) -> u8 {
        self.day
    }

    /// Whether this date's year is a leap year.
    pub fn is_leap_year(self) -> bool {
        is_leap_year(self.year)
    }

    /// The day of the year, `1..=366`.
    pub fn ordinal(self) -> u16 {
        let jan_1 = GregorianDate {
            year: self.year,
            month: 1,
            day: 1,
        };
        (self.to_fixed() - jan_1.to_fixed() + 1) as u16
    }
}

/// Whether `year` is a Gregorian leap year: every fourth, except every
/// hundredth, except every four-hundredth.
pub fn is_leap_year(year: i64) -> bool {
    year.rem_euclid(4) == 0 && !matches!(year.rem_euclid(400), 100 | 200 | 300)
}

/// How many days month `month` of `year` has. Returns 0 for a month outside
/// `1..=12`.
pub fn days_in_month(year: i64, month: u8) -> u8 {
    match month {
        2 if is_leap_year(year) => 29,
        1..=12 => MONTH_LENGTHS[(month - 1) as usize],
        _ => 0,
    }
}

/// How many days `year` has, 365 or 366.
pub fn days_in_year(year: i64) -> u16 {
    if is_leap_year(year) {
        366
    } else {
        365
    }
}

impl CalendarDate for GregorianDate {
    const CALENDAR: &'static str = CALENDAR;

    fn to_fixed(&self) -> FixedDay {
        let prior = self.year - 1;
        // Days in whole years before this one, then whole months before this
        // one, then days before this one in its month. The month term is the
        // standard closed form for the 31/30 pattern; the correction repairs
        // February, which is the only month whose length is not a function of
        // its number alone.
        let correction = if self.month <= 2 {
            0
        } else if is_leap_year(self.year) {
            -1
        } else {
            -2
        };
        let days = 365 * prior + prior.div_euclid(4) - prior.div_euclid(100)
            + prior.div_euclid(400)
            + (367 * self.month as i64 - 362).div_euclid(12)
            + correction
            + self.day as i64;
        FixedDay::new(days)
    }

    fn from_fixed(fixed: FixedDay) -> GregorianDate {
        let year = year_from_fixed(fixed);
        let jan_1 = GregorianDate {
            year,
            month: 1,
            day: 1,
        }
        .to_fixed();
        let march_1 = GregorianDate {
            year,
            month: 3,
            day: 1,
        }
        .to_fixed();
        let prior_days = fixed - jan_1;
        let correction = if fixed < march_1 {
            0
        } else if is_leap_year(year) {
            1
        } else {
            2
        };
        let month = ((12 * (prior_days + correction) + 373).div_euclid(367)) as u8;
        let month_1 = GregorianDate {
            year,
            month,
            day: 1,
        }
        .to_fixed();
        let day = (fixed - month_1 + 1) as u8;
        GregorianDate { year, month, day }
    }
}

/// Which Gregorian year a day falls in, found by peeling off 400-, 100-, 4-
/// and 1-year blocks in turn.
fn year_from_fixed(fixed: FixedDay) -> i64 {
    let d0 = fixed.get() - 1;
    let n400 = d0.div_euclid(146_097);
    let d1 = d0.rem_euclid(146_097);
    let n100 = d1.div_euclid(36_524);
    let d2 = d1.rem_euclid(36_524);
    let n4 = d2.div_euclid(1_461);
    let d3 = d2.rem_euclid(1_461);
    let n1 = d3.div_euclid(365);
    let year = 400 * n400 + 100 * n100 + 4 * n4 + n1;
    // n100 == 4 or n1 == 4 means the day is the last of a 400- or 4-year
    // block, which the division has already counted as the next year.
    if n100 == 4 || n1 == 4 {
        year
    } else {
        year + 1
    }
}

impl fmt::Display for GregorianDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for GregorianDate {
    type Err = DateError;

    fn from_str(s: &str) -> Result<GregorianDate, DateError> {
        let (year, month, day) = crate::parse_ymd(s, CALENDAR)?;
        GregorianDate::new(year, month, day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodesh_types::Weekday;

    #[test]
    fn known_fixed_days() {
        // The rata die anchors, from Reingold and Dershowitz.
        assert_eq!(
            GregorianDate::new(1, 1, 1).unwrap().to_fixed(),
            FixedDay::new(1)
        );
        assert_eq!(
            GregorianDate::new(1970, 1, 1).unwrap().to_fixed(),
            FixedDay::new(719_163)
        );
        assert_eq!(
            GregorianDate::new(2000, 1, 1).unwrap().to_fixed(),
            FixedDay::new(730_120)
        );
    }

    #[test]
    fn known_weekdays() {
        assert_eq!(
            GregorianDate::new(2000, 1, 1).unwrap().weekday(),
            Weekday::Saturday
        );
        assert_eq!(
            GregorianDate::new(2026, 8, 29).unwrap().weekday(),
            Weekday::Saturday
        );
        assert_eq!(
            GregorianDate::new(1969, 7, 20).unwrap().weekday(),
            Weekday::Sunday
        );
    }

    #[test]
    fn round_trips_across_four_centuries() {
        let start = GregorianDate::new(1600, 1, 1).unwrap().to_fixed();
        let end = GregorianDate::new(2400, 1, 1).unwrap().to_fixed();
        for n in start.get()..=end.get() {
            let fixed = FixedDay::new(n);
            let date = GregorianDate::from_fixed(fixed);
            assert_eq!(date.to_fixed(), fixed, "{date} did not round-trip");
            assert!(GregorianDate::new(date.year(), date.month(), date.day()).is_ok());
        }
    }

    #[test]
    fn round_trips_before_the_epoch() {
        for n in -200_000..-199_000 {
            let fixed = FixedDay::new(n);
            assert_eq!(GregorianDate::from_fixed(fixed).to_fixed(), fixed);
        }
    }

    #[test]
    fn leap_years_follow_the_four_hundred_year_rule() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2100));
        assert!(is_leap_year(0)); // 1 BCE, leap under the proleptic rule
        assert!(!is_leap_year(-1));
        assert!(is_leap_year(-4));
    }

    #[test]
    fn february_29_exists_only_in_a_leap_year() {
        assert!(GregorianDate::new(2024, 2, 29).is_ok());
        assert_eq!(
            GregorianDate::new(2023, 2, 29).unwrap_err().code(),
            HodeshCode::DayOutOfRange
        );
        assert_eq!(
            GregorianDate::new(2024, 13, 1).unwrap_err().code(),
            HodeshCode::MonthOutOfRange
        );
    }

    #[test]
    fn ordinal_counts_from_january_1() {
        assert_eq!(GregorianDate::new(2024, 1, 1).unwrap().ordinal(), 1);
        assert_eq!(GregorianDate::new(2024, 12, 31).unwrap().ordinal(), 366);
        assert_eq!(GregorianDate::new(2023, 12, 31).unwrap().ordinal(), 365);
    }

    #[test]
    fn text_form_round_trips() {
        let date = GregorianDate::new(2000, 1, 6).unwrap();
        assert_eq!(date.to_string(), "2000-01-06");
        assert_eq!("2000-01-06".parse::<GregorianDate>().unwrap(), date);
        assert_eq!(
            "2000-1-6".parse::<GregorianDate>().unwrap_err().code(),
            HodeshCode::MalformedDate
        );
    }
}
