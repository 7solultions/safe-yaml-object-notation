//! Luach types -- the calendar-agnostic base every calendar here stands on.
//!
//! This crate knows what a *day* is and nothing about what a *date* is. It
//! holds the day count every calendar converts through ([`FixedDay`]), the
//! week that runs underneath all of them ([`Weekday`]), the contract a
//! calendar implements ([`CalendarDate`]), and the error a date raises when
//! it cannot exist ([`DateError`]).
//!
//! *Luach* (לוּחַ) is the board a calendar is written on, which is what this
//! crate is: the surface, not any of the calendars drawn on it. The calendars
//! themselves live above -- `hodesh_calendar` for the lunisolar one and the
//! proleptic Gregorian, `shavua_calendar` for the perennial one -- and each
//! depends on this crate without either depending on the other.
//!
//! The split is the one the workspace already uses between `sheni` and
//! `shelishi`: a lower layer that defines the shapes, an upper one that
//! supplies the instances. A third-party calendar can implement
//! [`CalendarDate`] against this crate alone and convert to and from every
//! calendar in the workspace without either side importing the other.
//!
//! ```
//! use luach_types::{CalendarDate, FixedDay, Weekday};
//!
//! // A day is a count, and the week is a property of the count.
//! let day = FixedDay::new(730120); // proleptic Gregorian 2000-01-01
//! assert_eq!(day.weekday(), Weekday::Saturday);
//! assert_eq!((day + 5).weekday(), Weekday::Thursday);
//! ```

pub mod calendar;
pub mod error;
pub mod fixed;
pub mod text;
pub mod weekday;

pub use calendar::CalendarDate;
pub use error::{DateError, LuachCode};
pub use fixed::FixedDay;
pub use text::{format_ymd, parse_ymd};
pub use weekday::Weekday;
