//! The day number every calendar converts through.
//!
//! Converting between two calendars directly needs a routine per pair, which
//! is quadratic in the number of calendars and wrong in a new way each time.
//! Converting through a single count of days is linear: each calendar states
//! how to reach the count and how to come back from it, and every pair of
//! calendars is then connected without either one knowing the other exists.
//!
//! The count is the *rata die* of Reingold and Dershowitz: day 1 is
//! 0001-01-01 in the proleptic Gregorian calendar, days before it are
//! negative, and there is no year zero problem because the count does not use
//! years. It is deliberately not the Unix epoch, which starts at a day that
//! is arbitrary in every calendar including its own, and deliberately not the
//! Julian Day Number, which begins at noon and so makes a *day* a thing that
//! straddles two dates.
//!
//! A [`FixedDay`] is a whole day, not an instant. Where the day begins is a
//! separate question, answered once for all calendars here: at midnight UTC,
//! with no local offset and no summer time. A calendar that means to be a
//! standard cannot have a date that depends on where the reader is standing.

use std::fmt;
use std::ops::{Add, Sub};

use crate::weekday::Weekday;

/// A count of days, with day 1 at proleptic Gregorian 0001-01-01.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FixedDay(i64);

impl FixedDay {
    /// Day 1: proleptic Gregorian 0001-01-01, a Monday.
    pub const EPOCH: FixedDay = FixedDay(1);

    /// The day this count names.
    pub const fn new(day: i64) -> FixedDay {
        FixedDay(day)
    }

    /// The raw count.
    pub const fn get(self) -> i64 {
        self.0
    }

    /// Which day of the week this is.
    ///
    /// Day 1 is a Monday and the week has never been interrupted since, so
    /// this is a remainder and nothing more.
    pub fn weekday(self) -> Weekday {
        let index = (self.0 - 1).rem_euclid(7);
        Weekday::from_iso(index as u8 + 1).expect("index is 1..=7 by construction")
    }

    /// The day `n` days later, or `None` on overflow.
    pub fn checked_add_days(self, n: i64) -> Option<FixedDay> {
        self.0.checked_add(n).map(FixedDay)
    }

    /// How many days from this day to `other`, positive when `other` is
    /// later.
    pub fn days_until(self, other: FixedDay) -> i64 {
        other.0 - self.0
    }
}

impl fmt::Display for FixedDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Add<i64> for FixedDay {
    type Output = FixedDay;

    fn add(self, n: i64) -> FixedDay {
        FixedDay(self.0 + n)
    }
}

impl Sub<i64> for FixedDay {
    type Output = FixedDay;

    fn sub(self, n: i64) -> FixedDay {
        FixedDay(self.0 - n)
    }
}

impl Sub<FixedDay> for FixedDay {
    type Output = i64;

    fn sub(self, other: FixedDay) -> i64 {
        self.0 - other.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_one_is_a_monday() {
        assert_eq!(FixedDay::EPOCH.weekday(), Weekday::Monday);
    }

    #[test]
    fn weekday_is_defined_before_the_epoch_too() {
        assert_eq!(FixedDay::new(0).weekday(), Weekday::Sunday);
        assert_eq!(FixedDay::new(-1).weekday(), Weekday::Saturday);
        assert_eq!(FixedDay::new(-6).weekday(), Weekday::Monday);
    }

    #[test]
    fn difference_and_offset_agree() {
        let a = FixedDay::new(730120);
        let b = a + 41;
        assert_eq!(a.days_until(b), 41);
        assert_eq!(b - a, 41);
        assert_eq!(b - 41, a);
    }
}
