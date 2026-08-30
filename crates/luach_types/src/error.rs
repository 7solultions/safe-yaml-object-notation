//! The error returned when a date does not exist in the calendar it was
//! written for.
//!
//! The discipline is the one `syon-parser` established for parse errors and
//! `sheni` carried into the type layer: a caller asks "is this specifically a
//! day past the end of the month?" by matching a number, not by matching
//! message text. The code is API; the wording is not.
//!
//! Codes are three digits in the `601-699` band, which the calendar crates
//! take for themselves. The bands below it are spoken for -- `1-499` by
//! `sheni`'s four type groups and `501-599` by `shelishi_schema` -- so a
//! calendar code and a sheni code never collide even where the two are
//! reported side by side.
//!
//! One band covers every calendar rather than one band each. A month out of
//! range is the same failure whichever calendar rejected it, and the calendar
//! that did is carried on the [`DateError`] itself.

use std::fmt;

/// A stable numeric identifier for a date that cannot exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum LuachCode {
    /// The month number is outside `1..=months_in_year` for that year. In a
    /// lunisolar calendar the bound depends on the year, so month 13 is legal
    /// in a leap year and this error in a common one.
    MonthOutOfRange = 601,
    /// The day number is outside `1..=days_in_month` for that month.
    DayOutOfRange = 602,
    /// The year is outside the range the calendar's arithmetic is defined
    /// over.
    YearOutOfRange = 603,
    /// The text is not in the calendar's canonical `YYYY-MM-DD` form.
    MalformedDate = 604,
    /// A conversion or an offset ran past the range a [`crate::FixedDay`] can
    /// hold. Reported rather than wrapped, on the same reasoning as sheni's
    /// integer overflow.
    DayOutOfRepresentableRange = 605,
}

impl LuachCode {
    /// The numeric value, for a caller that wants to store or transmit it.
    pub fn number(self) -> u16 {
        self as u16
    }
}

impl fmt::Display for LuachCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.number())
    }
}

/// A date failure: what went wrong, in which calendar.
///
/// The calendar name is carried so a message can say which one rejected the
/// date -- the same day number is legal in one and impossible in another, and
/// a message that does not name the calendar makes that confusing rather than
/// obvious.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DateError {
    code: LuachCode,
    calendar: &'static str,
    message: String,
}

impl DateError {
    pub fn new(code: LuachCode, calendar: &'static str, message: impl Into<String>) -> Self {
        DateError {
            code,
            calendar,
            message: message.into(),
        }
    }

    /// The numeric code, stable across message rewordings.
    pub fn code(&self) -> LuachCode {
        self.code
    }

    /// The name of the calendar that rejected the date, e.g. `hodesh`.
    pub fn calendar(&self) -> &'static str {
        self.calendar
    }

    /// The human-readable explanation. Not API -- match on
    /// [`DateError::code`] instead.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for DateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}: {}", self.code, self.calendar, self.message)
    }
}

impl std::error::Error for DateError {}
