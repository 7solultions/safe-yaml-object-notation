//! The seven-day week, which every calendar here shares.
//!
//! The week is the one cycle that runs underneath all of them without
//! reference to sun or moon: it has never been interrupted or renumbered, so
//! it is a property of the day itself rather than of the calendar the day is
//! written in. That makes it a [`crate::FixedDay`] operation, and every
//! calendar gets it for free.
//!
//! Monday is the first day, per ISO 8601.

use std::fmt;

/// A day of the week, Monday first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Weekday {
    Monday = 1,
    Tuesday = 2,
    Wednesday = 3,
    Thursday = 4,
    Friday = 5,
    Saturday = 6,
    Sunday = 7,
}

impl Weekday {
    /// The ISO 8601 number, Monday `1` through Sunday `7`.
    pub fn number(self) -> u8 {
        self as u8
    }

    /// The weekday `n` days after this one. Negative `n` counts backwards.
    pub fn add_days(self, n: i64) -> Weekday {
        let index = (self.number() as i64 - 1 + n).rem_euclid(7);
        Weekday::from_iso(index as u8 + 1).expect("index is 1..=7 by construction")
    }

    /// The weekday with the given ISO number, or `None` outside `1..=7`.
    pub fn from_iso(n: u8) -> Option<Weekday> {
        match n {
            1 => Some(Weekday::Monday),
            2 => Some(Weekday::Tuesday),
            3 => Some(Weekday::Wednesday),
            4 => Some(Weekday::Thursday),
            5 => Some(Weekday::Friday),
            6 => Some(Weekday::Saturday),
            7 => Some(Weekday::Sunday),
            _ => None,
        }
    }

    /// The English name, capitalised.
    pub fn name(self) -> &'static str {
        match self {
            Weekday::Monday => "Monday",
            Weekday::Tuesday => "Tuesday",
            Weekday::Wednesday => "Wednesday",
            Weekday::Thursday => "Thursday",
            Weekday::Friday => "Friday",
            Weekday::Saturday => "Saturday",
            Weekday::Sunday => "Sunday",
        }
    }
}

impl fmt::Display for Weekday {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_days_wraps_in_both_directions() {
        assert_eq!(Weekday::Monday.add_days(7), Weekday::Monday);
        assert_eq!(Weekday::Monday.add_days(1), Weekday::Tuesday);
        assert_eq!(Weekday::Monday.add_days(-1), Weekday::Sunday);
        assert_eq!(Weekday::Sunday.add_days(1), Weekday::Monday);
        assert_eq!(Weekday::Wednesday.add_days(-10), Weekday::Sunday);
    }

    #[test]
    fn iso_numbering_round_trips() {
        for n in 1..=7 {
            assert_eq!(Weekday::from_iso(n).unwrap().number(), n);
        }
        assert_eq!(Weekday::from_iso(0), None);
        assert_eq!(Weekday::from_iso(8), None);
    }
}
