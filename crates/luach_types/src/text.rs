//! The canonical text form, shared by every calendar rather than written once
//! per calendar.
//!
//! A date is written `YYYY-MM-DD`, zero-padded, with a leading minus for
//! years before the epoch, and it is parsed strictly: `2000-1-6` is a
//! [`LuachCode::MalformedDate`] and not a lenient success. ADR hodesh_04
//! settled that for hodesh and the reasoning is not hodesh's -- a format
//! meant for interchange has exactly one spelling, and accepting a second
//! means every consumer has to handle both, until one of them handles only
//! one.
//!
//! It lives here rather than in a calendar crate because it is the same
//! grammar for all of them. What differs between calendars is which
//! `(year, month, day)` triples are *legal*, which is the calendar's own
//! business and is checked after parsing, not during it.

use crate::error::{DateError, LuachCode};

/// Render a date in the canonical `YYYY-MM-DD` form.
///
/// The year is padded to four digits *after* the sign, so year -44 is
/// `-0044-03-15` and not `-044-03-15`. A plain `{:04}` counts the minus
/// toward the width and produces the latter, which [`parse_ymd`] rejects --
/// a round trip that fails on every year before the epoch, which for hodesh
/// is nearly every date anyone actually holds.
pub fn format_ymd(year: i64, month: u8, day: u8) -> String {
    let sign = if year < 0 { "-" } else { "" };
    let magnitude = year.unsigned_abs();
    format!("{sign}{magnitude:04}-{month:02}-{day:02}")
}

/// Parse the canonical `YYYY-MM-DD` text form.
///
/// Returns the three fields without judging them: a month of `14` parses
/// here and is then accepted by shavua and rejected by hodesh, because which
/// months exist is the calendar's question and not the grammar's.
pub fn parse_ymd(s: &str, calendar: &'static str) -> Result<(i64, u8, u8), DateError> {
    let malformed = || {
        DateError::new(
            LuachCode::MalformedDate,
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

    #[test]
    fn parse_rejects_slack_spellings() {
        for bad in ["2000-1-06", "2000-01-6", "20000-01-06", "2000/01/06", ""] {
            assert_eq!(
                parse_ymd(bad, "test").unwrap_err().code(),
                LuachCode::MalformedDate,
                "{bad} should not parse"
            );
        }
    }

    #[test]
    fn round_trips_through_the_canonical_form() {
        for (y, m, d) in [(2000, 1, 6), (26, 5, 17), (-44, 3, 15), (0, 14, 1)] {
            let text = format_ymd(y, m, d);
            assert_eq!(parse_ymd(&text, "test").unwrap(), (y, m, d));
        }
    }

    #[test]
    fn negative_years_keep_four_digits_after_the_sign() {
        assert_eq!(format_ymd(-44, 3, 15), "-0044-03-15");
        assert_eq!(parse_ymd("-0044-03-15", "test").unwrap(), (-44, 3, 15));
    }
}
