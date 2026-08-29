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
//! a [`hodesh_types::FixedDay`] and how to come back from one, and
//! [`hodesh_types::CalendarDate::convert`] does the rest, so adding a third
//! calendar connects it to both of these without touching either.
//!
//! ```
//! use hodesh_calendar::{GregorianDate, HodeshDate};
//! use hodesh_types::CalendarDate;
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

use hodesh_types::{DateError, HodeshCode};

/// Parse the canonical `YYYY-MM-DD` text form shared by every calendar here.
///
/// Shared rather than written twice, and strict on purpose: the field widths
/// are fixed, so `2000-1-6` is a [`HodeshCode::MalformedDate`] and not a
/// lenient success. A calendar meant as an interchange standard has exactly
/// one spelling for a date, on the same reasoning that made sheni's `date`
/// reject the same string.
pub(crate) fn parse_ymd(s: &str, calendar: &'static str) -> Result<(i64, u8, u8), DateError> {
    let malformed = || {
        DateError::new(
            HodeshCode::MalformedDate,
            calendar,
            format!("expected a date in the form `YYYY-MM-DD`, found `{s}`"),
        )
    };

    let (negative, body) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };

    let bytes = body.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(malformed());
    }
    if !bytes
        .iter()
        .enumerate()
        .all(|(i, b)| matches!(i, 4 | 7) || b.is_ascii_digit())
    {
        return Err(malformed());
    }

    let year: i64 = body[0..4].parse().map_err(|_| malformed())?;
    let month: u8 = body[5..7].parse().map_err(|_| malformed())?;
    let day: u8 = body[8..10].parse().map_err(|_| malformed())?;

    Ok((if negative { -year } else { year }, month, day))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hodesh_types::CalendarDate;

    #[test]
    fn every_day_converts_both_ways_between_the_two_calendars() {
        let start = GregorianDate::new(1900, 1, 1).unwrap().to_fixed();
        let end = GregorianDate::new(2100, 1, 1).unwrap().to_fixed();
        for n in start.get()..=end.get() {
            let gregorian = GregorianDate::from_fixed(hodesh_types::FixedDay::new(n));
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

    #[test]
    fn parse_rejects_slack_spellings() {
        for bad in ["2000-1-06", "2000-01-6", "20000-01-06", "2000/01/06", ""] {
            assert_eq!(
                parse_ymd(bad, "test").unwrap_err().code(),
                HodeshCode::MalformedDate,
                "{bad} should not parse"
            );
        }
        assert_eq!(parse_ymd("-0044-03-15", "test").unwrap(), (-44, 3, 15));
    }
}
