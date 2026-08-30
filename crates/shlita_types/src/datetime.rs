//! DATE, TIME_OF_DAY and DATE_AND_TIME, in both widths.
//!
//! Three point-in-time types where TIME is a span, and the standard keeps
//! them apart because subtracting two of them gives a TIME while adding two
//! of them gives nothing at all.
//!
//! The calendar itself is delegated. `time::Date` implements the proleptic
//! Gregorian calendar, and looking the answer up rather than deciding it is
//! the habit ADR sheni_10 named -- the same reason `hodesh_calendar` exists
//! next door for the calendar nobody had implemented.
//!
//! The epoch is 1970-01-01, which is the standard's own earliest date, and a
//! DATE before it is out of range rather than negative.

use std::fmt;

use crate::elementary::{ElementaryType, ElementaryValue};
use crate::error::{Result, ShlitaError};
use crate::error_code::ShlitaCode;
use crate::numeric::strip_type_prefix;

/// 1970-01-01 as a Julian day number, so a day count and a calendar date
/// convert in both directions through one constant.
const UNIX_EPOCH_JULIAN_DAY: i32 = 2_440_588;

const NANOS_PER_SECOND: u64 = 1_000_000_000;
const NANOS_PER_DAY: i128 = 86_400 * NANOS_PER_SECOND as i128;

/// The last day each width can name.
///
/// DATE runs to the end of the four-digit years. LDATE counts nanoseconds
/// since the epoch in 64 bits, and 2262-04-11 is where that count ends --
/// the range is the representation's, and saying so here is cheaper than
/// discovering it in an overflow.
pub(crate) fn last_day(ty: ElementaryType) -> i32 {
    if ty.is_long() {
        106_751 // 2262-04-11
    } else {
        2_932_896 // 9999-12-31
    }
}

/// Read DATE or LDATE.
pub(crate) fn read_date(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = require_prefix(ty, literal, ShlitaCode::MalformedDate)?;
    let days = read_calendar_date(ty, literal, body, ShlitaCode::MalformedDate)?;
    Ok(ElementaryValue::Date { ty, days })
}

/// Read TIME_OF_DAY or LTIME_OF_DAY.
pub(crate) fn read_time_of_day(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = require_prefix(ty, literal, ShlitaCode::MalformedTimeOfDay)?;
    let nanos = read_clock(ty, literal, body, ShlitaCode::MalformedTimeOfDay)?;
    Ok(ElementaryValue::TimeOfDay { ty, nanos })
}

/// Read DATE_AND_TIME or LDATE_AND_TIME.
///
/// The standard joins the two halves with a hyphen -- `DT#2026-08-29-12:00:00`
/// -- so the split is at the fourth hyphen and not at a space.
pub(crate) fn read_date_and_time(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = require_prefix(ty, literal, ShlitaCode::MalformedDateAndTime)?;
    let mut hyphens = body.match_indices('-');
    let at = hyphens
        .nth(2)
        .map(|(at, _)| at)
        .ok_or_else(|| malformed(ty, literal, ShlitaCode::MalformedDateAndTime))?;
    let days = read_calendar_date(ty, literal, &body[..at], ShlitaCode::MalformedDateAndTime)?;
    let nanos = read_clock(
        ty,
        literal,
        &body[at + 1..],
        ShlitaCode::MalformedDateAndTime,
    )?;
    Ok(ElementaryValue::DateAndTime {
        ty,
        nanos: i128::from(days) * NANOS_PER_DAY + i128::from(nanos),
    })
}

/// Every date type's literal carries its type, and the prefix is not
/// optional the way an integer's is.
fn require_prefix(ty: ElementaryType, literal: &str, code: ShlitaCode) -> Result<&str> {
    if !literal.contains('#') {
        let short = ty.aliases().first().copied().unwrap_or(ty.name());
        return Err(ty.err(
            code,
            literal,
            format!("a {ty} literal opens with its type, as in `{short}#...`"),
        ));
    }
    strip_type_prefix(ty, literal)
}

fn malformed(ty: ElementaryType, literal: &str, code: ShlitaCode) -> ShlitaError {
    let shape = match code {
        ShlitaCode::MalformedDate => "YYYY-MM-DD",
        ShlitaCode::MalformedTimeOfDay => "hh:mm:ss",
        _ => "YYYY-MM-DD-hh:mm:ss",
    };
    ty.err(code, literal, format!("expected {shape}"))
}

/// `YYYY-MM-DD` to a count of days since the epoch.
fn read_calendar_date(
    ty: ElementaryType,
    literal: &str,
    body: &str,
    code: ShlitaCode,
) -> Result<i32> {
    let mut parts = body.split('-');
    let (Some(year), Some(month), Some(day), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(malformed(ty, literal, code));
    };
    let year: i32 = number(year, 1, 4).ok_or_else(|| malformed(ty, literal, code))?;
    let month: u8 = number(month, 1, 2).ok_or_else(|| malformed(ty, literal, code))?;
    let day: u8 = number(day, 1, 2).ok_or_else(|| malformed(ty, literal, code))?;

    let month = time::Month::try_from(month).map_err(|_| {
        ty.err(
            ShlitaCode::DateOutOfRange,
            literal,
            format!("there is no month {month}"),
        )
    })?;
    let date = time::Date::from_calendar_date(year, month, day).map_err(|_| {
        ty.err(
            ShlitaCode::DateOutOfRange,
            literal,
            "that day does not exist in that month",
        )
    })?;
    let days = date.to_julian_day() - UNIX_EPOCH_JULIAN_DAY;
    if days < 0 || days > last_day(ty) {
        return Err(ty.err(
            ShlitaCode::DateOutOfRange,
            literal,
            format!(
                "{ty} runs from 1970-01-01 to {}",
                civil(last_day(ty))
                    .map(|d| format!("{:04}-{:02}-{:02}", d.0, d.1, d.2))
                    .unwrap_or_default()
            ),
        ));
    }
    Ok(days)
}

/// `hh:mm:ss` or `hh:mm:ss.fff` to nanoseconds since midnight.
fn read_clock(ty: ElementaryType, literal: &str, body: &str, code: ShlitaCode) -> Result<u64> {
    let mut parts = body.split(':');
    let (Some(hours), Some(minutes), Some(seconds), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(malformed(ty, literal, code));
    };
    let hours: u64 = number(hours, 1, 2).ok_or_else(|| malformed(ty, literal, code))?;
    let minutes: u64 = number(minutes, 1, 2).ok_or_else(|| malformed(ty, literal, code))?;
    let (seconds, fraction) = match seconds.split_once('.') {
        Some((seconds, fraction)) => (seconds, Some(fraction)),
        None => (seconds, None),
    };
    let seconds: u64 = number(seconds, 1, 2).ok_or_else(|| malformed(ty, literal, code))?;
    if hours > 23 || minutes > 59 || seconds > 59 {
        return Err(ty.err(
            code,
            literal,
            "a time of day runs from 00:00:00 to 23:59:59",
        ));
    }

    let mut nanos = ((hours * 60 + minutes) * 60 + seconds) * NANOS_PER_SECOND;
    if let Some(fraction) = fraction {
        if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
            return Err(malformed(ty, literal, code));
        }
        if fraction.len() > 9 {
            return Err(ty.err(code, literal, "a fraction of a second stops at nanoseconds"));
        }
        let scaled: u64 = fraction.parse().expect("digits only, at most nine of them");
        let places = 9 - fraction.len() as u32;
        let scaled = scaled * 10u64.pow(places);
        if !ty.is_long() && !scaled.is_multiple_of(1_000_000) {
            return Err(ty.err(code, literal, format!("{ty} resolves to milliseconds")));
        }
        nanos += scaled;
    }
    Ok(nanos)
}

/// A fixed-width run of decimal digits.
fn number<T: std::str::FromStr>(text: &str, least: usize, most: usize) -> Option<T> {
    if text.len() < least || text.len() > most || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// A day count back to year, month and day.
fn civil(days: i32) -> Option<(i32, u8, u8)> {
    let date = time::Date::from_julian_day(days + UNIX_EPOCH_JULIAN_DAY).ok()?;
    Some((date.year(), date.month() as u8, date.day()))
}

pub(crate) fn format_date(
    ty: ElementaryType,
    days: i32,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let short = ty.aliases().first().copied().unwrap_or(ty.name());
    let (year, month, day) = civil(days).ok_or(fmt::Error)?;
    write!(f, "{short}#{year:04}-{month:02}-{day:02}")
}

pub(crate) fn format_time_of_day(
    ty: ElementaryType,
    nanos: u64,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let short = ty.aliases().first().copied().unwrap_or(ty.name());
    write!(f, "{short}#")?;
    write_clock(ty, nanos, f)
}

pub(crate) fn format_date_and_time(
    ty: ElementaryType,
    nanos: i128,
    f: &mut fmt::Formatter<'_>,
) -> fmt::Result {
    let short = ty.aliases().first().copied().unwrap_or(ty.name());
    let days = nanos.div_euclid(NANOS_PER_DAY) as i32;
    let rest = nanos.rem_euclid(NANOS_PER_DAY) as u64;
    let (year, month, day) = civil(days).ok_or(fmt::Error)?;
    write!(f, "{short}#{year:04}-{month:02}-{day:02}-")?;
    write_clock(ty, rest, f)
}

/// The clock half, with a fraction only when there is one to write.
fn write_clock(ty: ElementaryType, nanos: u64, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let seconds = nanos / NANOS_PER_SECOND;
    let fraction = nanos % NANOS_PER_SECOND;
    write!(
        f,
        "{:02}:{:02}:{:02}",
        seconds / 3_600,
        (seconds / 60) % 60,
        seconds % 60
    )?;
    if fraction == 0 {
        return Ok(());
    }
    if ty.is_long() {
        let text = format!("{fraction:09}");
        write!(f, ".{}", text.trim_end_matches('0'))
    } else {
        write!(f, ".{:03}", fraction / 1_000_000)
    }
}

/// Build a date from a computed day count, refusing one that leaves the
/// range the type can name.
pub(crate) fn checked_date(
    ty: ElementaryType,
    days: i128,
    context: &str,
) -> Result<ElementaryValue> {
    if days < 0 || days > i128::from(last_day(ty)) {
        return Err(ShlitaError::new(
            ShlitaCode::ArithmeticOverflow,
            context,
            days.to_string(),
            format!("the result leaves the range of {ty}"),
        ));
    }
    Ok(ElementaryValue::Date {
        ty,
        days: days as i32,
    })
}

/// Build a date and time of day from a computed nanosecond count.
pub(crate) fn checked_date_and_time(
    ty: ElementaryType,
    nanos: i128,
    context: &str,
) -> Result<ElementaryValue> {
    let last = i128::from(last_day(ty)) * NANOS_PER_DAY + NANOS_PER_DAY - 1;
    if nanos < 0 || nanos > last {
        return Err(ShlitaError::new(
            ShlitaCode::ArithmeticOverflow,
            context,
            nanos.to_string(),
            format!("the result leaves the range of {ty}"),
        ));
    }
    Ok(ElementaryValue::DateAndTime { ty, nanos })
}

/// Add a span to a time of day.
///
/// A time of day is a reading of a clock rather than a point on a line, so
/// it wraps at midnight instead of overflowing. The day it wrapped into is
/// not part of the type, which is the reason DATE_AND_TIME exists.
pub(crate) fn wrapped_time_of_day(ty: ElementaryType, nanos: i128) -> ElementaryValue {
    ElementaryValue::TimeOfDay {
        ty,
        nanos: nanos.rem_euclid(NANOS_PER_DAY) as u64,
    }
}

/// The nanoseconds in one day, for the callers that convert between the
/// three date types.
pub(crate) const fn nanos_per_day() -> i128 {
    NANOS_PER_DAY
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(ty: ElementaryType, literal: &str) -> ShlitaCode {
        ty.read(literal).unwrap_err().code()
    }

    fn show(ty: ElementaryType, literal: &str) -> String {
        ty.read(literal).unwrap().to_string()
    }

    #[test]
    fn a_date_counts_days_from_the_epoch() {
        assert_eq!(
            ElementaryType::Date.read("D#1970-01-01"),
            Ok(ElementaryValue::Date {
                ty: ElementaryType::Date,
                days: 0
            })
        );
        assert_eq!(
            ElementaryType::Date.read("DATE#1970-01-02"),
            Ok(ElementaryValue::Date {
                ty: ElementaryType::Date,
                days: 1
            })
        );
        assert_eq!(show(ElementaryType::Date, "D#2026-08-29"), "D#2026-08-29");
    }

    /// The delegate keeps the calendar honest, leap years included.
    #[test]
    fn a_day_that_does_not_exist_is_refused() {
        assert!(ElementaryType::Date.read("D#2024-02-29").is_ok());
        assert_eq!(
            code(ElementaryType::Date, "D#2023-02-29"),
            ShlitaCode::DateOutOfRange
        );
        assert_eq!(
            code(ElementaryType::Date, "D#2026-13-01"),
            ShlitaCode::DateOutOfRange
        );
        assert_eq!(
            code(ElementaryType::Date, "D#1969-12-31"),
            ShlitaCode::DateOutOfRange
        );
    }

    #[test]
    fn a_malformed_date_is_told_apart_from_an_impossible_one() {
        assert_eq!(
            code(ElementaryType::Date, "D#2026/08/29"),
            ShlitaCode::MalformedDate
        );
        assert_eq!(
            code(ElementaryType::Date, "2026-08-29"),
            ShlitaCode::MalformedDate
        );
        assert_eq!(
            code(ElementaryType::Date, "D#2026-08"),
            ShlitaCode::MalformedDate
        );
    }

    #[test]
    fn a_time_of_day_is_a_clock_and_not_a_duration() {
        assert_eq!(
            ElementaryType::TimeOfDay.read("TOD#00:00:00"),
            Ok(ElementaryValue::TimeOfDay {
                ty: ElementaryType::TimeOfDay,
                nanos: 0
            })
        );
        assert_eq!(
            show(ElementaryType::TimeOfDay, "TIME_OF_DAY#23:59:59.999"),
            "TOD#23:59:59.999"
        );
        assert_eq!(
            code(ElementaryType::TimeOfDay, "TOD#24:00:00"),
            ShlitaCode::MalformedTimeOfDay
        );
        assert_eq!(
            code(ElementaryType::TimeOfDay, "TOD#12:60:00"),
            ShlitaCode::MalformedTimeOfDay
        );
    }

    #[test]
    fn the_long_types_carry_the_finer_fraction() {
        assert_eq!(
            code(ElementaryType::TimeOfDay, "TOD#00:00:00.000001"),
            ShlitaCode::MalformedTimeOfDay
        );
        assert_eq!(
            show(ElementaryType::LtimeOfDay, "LTOD#00:00:00.000000001"),
            "LTOD#00:00:00.000000001"
        );
    }

    #[test]
    fn a_date_and_time_splits_at_the_third_hyphen() {
        let value = ElementaryType::DateAndTime
            .read("DT#2026-08-29-12:30:00")
            .unwrap();
        assert_eq!(value.to_string(), "DT#2026-08-29-12:30:00");
        assert_eq!(
            show(
                ElementaryType::DateAndTime,
                "DATE_AND_TIME#1970-01-01-00:00:01"
            ),
            "DT#1970-01-01-00:00:01"
        );
        assert_eq!(
            code(ElementaryType::DateAndTime, "DT#2026-08-29"),
            ShlitaCode::MalformedDateAndTime
        );
    }

    /// LDATE counts nanoseconds in 64 bits, and the range says where that
    /// ends rather than wrapping there.
    #[test]
    fn each_width_stops_where_its_representation_does() {
        assert!(ElementaryType::Date.read("D#9999-12-31").is_ok());
        assert_eq!(
            code(ElementaryType::Ldate, "LD#9999-12-31"),
            ShlitaCode::DateOutOfRange
        );
        assert!(ElementaryType::Ldate.read("LD#2262-04-11").is_ok());
    }
}
