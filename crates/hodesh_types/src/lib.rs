//! Hodesh types -- the calendar-agnostic half of the hodesh crates.
//!
//! This crate knows what a *day* is and nothing about what a *date* is. It
//! holds the day count every calendar converts through ([`FixedDay`]), the
//! week that runs underneath all of them ([`Weekday`]), the contract a
//! calendar implements ([`CalendarDate`]), and the error a date raises when
//! it cannot exist ([`DateError`]).
//!
//! The calendars themselves live in `hodesh_calendar`, which depends on this
//! crate. The split is the one the workspace already uses between `sheni` and
//! `shelishi`: a lower layer that defines the shapes, an upper one that
//! supplies the instances. A third-party calendar can implement
//! [`CalendarDate`] against this crate alone and convert to and from every
//! calendar in the workspace without either side importing the other.
//!
//! ```
//! use hodesh_types::{CalendarDate, FixedDay, Weekday};
//!
//! // A day is a count, and the week is a property of the count.
//! let day = FixedDay::new(730120); // proleptic Gregorian 2000-01-01
//! assert_eq!(day.weekday(), Weekday::Saturday);
//! assert_eq!((day + 5).weekday(), Weekday::Thursday);
//! ```

pub mod calendar;
pub mod error;
pub mod fixed;
pub mod weekday;

pub use calendar::CalendarDate;
pub use error::{DateError, HodeshCode};
pub use fixed::FixedDay;
pub use weekday::Weekday;
