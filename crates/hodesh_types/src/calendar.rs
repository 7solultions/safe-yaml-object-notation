//! What a calendar has to say about itself, and what it gets for free.
//!
//! A calendar implements two directions -- a date to a [`FixedDay`] and back
//! -- and everything else in this module follows from those two. Weekday,
//! day arithmetic, ordering across calendars, and conversion to every other
//! calendar are provided here rather than written once per calendar, which is
//! the whole reason the day count exists.

use crate::fixed::FixedDay;
use crate::weekday::Weekday;

/// A date in some calendar.
///
/// The two required methods must round-trip: for every representable date,
/// `Self::from_fixed(d.to_fixed()) == d`. That is the contract the free
/// methods lean on, and the one every implementation here tests.
pub trait CalendarDate: Sized {
    /// The calendar's name, as it appears in a [`crate::DateError`].
    const CALENDAR: &'static str;

    /// The day this date names.
    fn to_fixed(&self) -> FixedDay;

    /// The date naming this day.
    ///
    /// Total by construction: every day has a date in every calendar here, so
    /// there is no failure to report. Constructing a date from *parts* can
    /// fail, and that is where [`crate::DateError`] appears instead.
    fn from_fixed(fixed: FixedDay) -> Self;

    /// Which day of the week this date falls on.
    fn weekday(&self) -> Weekday {
        self.to_fixed().weekday()
    }

    /// The same day, read in another calendar.
    ///
    /// This is the point of the day count: `hodesh.convert::<GregorianDate>()`
    /// needs no code in either calendar that mentions the other.
    fn convert<T: CalendarDate>(&self) -> T {
        T::from_fixed(self.to_fixed())
    }

    /// The date `n` days after this one. Negative `n` counts backwards.
    fn add_days(&self, n: i64) -> Option<Self> {
        self.to_fixed().checked_add_days(n).map(Self::from_fixed)
    }

    /// How many days from this date to `other`, which may be in a different
    /// calendar entirely.
    fn days_until<T: CalendarDate>(&self, other: &T) -> i64 {
        self.to_fixed().days_until(other.to_fixed())
    }
}
