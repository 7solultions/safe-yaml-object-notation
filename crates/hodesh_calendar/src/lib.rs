//! Hodesh calendar -- the calendars themselves, and the conversions between
//! them.
//!
//! Two calendars so far:
//!
//! - [`gregorian`] -- the proleptic Gregorian calendar, present because it is
//!   what every existing record uses, and so the anchor every conversion goes
//!   through.
//! - [`hodesh`] -- a lunisolar calendar with numbered months, months that
//!   follow the mean lunation and years that follow the Metonic cycle. This
//!   is the calendar the crate is named for.
//!
//! Conversion is not written per pair. Each calendar states only how to reach
//! a [`luach_types::FixedDay`] and how to come back from one, and
//! [`luach_types::CalendarDate::convert`] does the rest, so adding a third
//! calendar connects it to both of these without touching either.
//!
//! ```
//! use hodesh_calendar::{GregorianDate, HodeshDate};
//! use luach_types::CalendarDate;
//!
//! // Hodesh year 0 begins at the first new moon of the year 2000.
//! let start = HodeshDate::new(0, 1, 1).unwrap();
//! assert_eq!(start.to_gregorian().to_string(), "2000-01-06");
//!
//! // Conversion is by day, so the weekday is necessarily the same one.
//! let today = GregorianDate::new(2026, 8, 29).unwrap();
//! let same_day: HodeshDate = today.convert();
//! assert_eq!(same_day.weekday(), today.weekday());
//! assert_eq!(same_day.to_gregorian(), today);
//! ```

pub mod gregorian;
pub mod hodesh;

pub use gregorian::GregorianDate;
pub use hodesh::HodeshDate;

#[cfg(test)]
mod tests {
    use super::*;
    use luach_types::CalendarDate;

    #[test]
    fn every_day_converts_both_ways_between_the_two_calendars() {
        let start = GregorianDate::new(1900, 1, 1).unwrap().to_fixed();
        let end = GregorianDate::new(2100, 1, 1).unwrap().to_fixed();
        for n in start.get()..=end.get() {
            let gregorian = GregorianDate::from_fixed(luach_types::FixedDay::new(n));
            let hodesh: HodeshDate = gregorian.convert();
            assert_eq!(hodesh.convert::<GregorianDate>(), gregorian);
            assert_eq!(hodesh.weekday(), gregorian.weekday());
        }
    }

    #[test]
    fn hodesh_new_year_stays_within_a_month_of_the_gregorian_one() {
        // A lunisolar year cannot start on a fixed Gregorian day, but the
        // Metonic cycle bounds how far it wanders. If this ever exceeds a
        // lunation the intercalation rule is wrong.
        for year in 0..190 {
            let new_year = HodeshDate::new(year, 1, 1).unwrap().to_gregorian();
            assert!(
                new_year.ordinal() <= 31 || new_year.ordinal() >= 335,
                "hodesh year {year} began on {new_year}, adrift of January"
            );
        }
    }
}
