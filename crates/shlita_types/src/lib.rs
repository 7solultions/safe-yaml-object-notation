//! Shlita -- the IEC 61131-3 type vocabulary and its standard functions.
//!
//! Shlita is control, or mastery, and it names the domain the way `hodesh`
//! names its own. This is the lower of the two crates ADR shlita_01 split the
//! standard into: the shapes a controller's values take, and the operations
//! that are pure functions of their arguments. The timers, counters,
//! bistables and edge detectors are stateful, only mean anything under a
//! cyclic execution model, and live in `shlita_runtime`.
//!
//! Three things are here:
//!
//! 1. **[`ElementaryType`] and [`ElementaryValue`]** -- the twenty-seven
//!    elementary types of the third edition, and reading a literal at one of
//!    them.
//! 2. **[`functions`]** -- the standard functions: bitwise, selection,
//!    comparison, arithmetic, numeric and string.
//! 3. **[`convert`]** -- the explicit conversions, inside the vocabulary and
//!    across to sheni's primitives.
//!
//! ```
//! use shlita_types::{ElementaryType, ElementaryValue, ShlitaCode};
//!
//! // A literal is read at a type, and the type is part of the value.
//! let count = ElementaryType::Int.read("16#7FFF").unwrap();
//! assert_eq!(count.to_string(), "32767");
//!
//! // It does not wrap when it does not fit.
//! assert_eq!(
//!     ElementaryType::Sint.read("128").unwrap_err().code(),
//!     ShlitaCode::IntegerOutOfRange
//! );
//! ```
//!
//! The vocabulary is this crate's own rather than sheni's, because three of
//! the standard's types have no honest counterpart there. `BYTE` is an
//! eight-bit string that may be ANDed, where sheni's `byte` is a character;
//! the bit strings are types distinct from the unsigned integers the standard
//! keeps them apart from; and TIME admits a sign, a fraction and an
//! overflowing leading unit that sheni's `duration` refuses:
//!
//! ```
//! use shlita_types::{ElementaryType, ShlitaCode};
//!
//! // The IEC duration literal, with everything sheni's `duration` will not take.
//! assert_eq!(ElementaryType::Time.read("T#-1d25h30.5m").is_ok(), false); // 25h is not the leading unit
//! assert!(ElementaryType::Time.read("T#-25h30.5m").is_ok());
//!
//! // A bit string may be ANDed; the unsigned integer of the same width may not.
//! let a = ElementaryType::Byte.read("2#1100").unwrap();
//! let b = ElementaryType::Byte.read("2#1010").unwrap();
//! assert_eq!(shlita_types::call("AND", &[a, b]).unwrap().to_string(), "16#08");
//!
//! let n = ElementaryType::Usint.read("12").unwrap();
//! assert_eq!(
//!     shlita_types::call("AND", &[n.clone(), n]).unwrap_err().code(),
//!     ShlitaCode::NotABitString
//! );
//! ```
//!
//! Errors carry a [`ShlitaCode`] from the `701-799` band ADR shlita_01
//! reserved, so a caller matches a number rather than a message.

pub mod convert;
pub mod datetime;
pub mod duration;
pub mod elementary;
pub mod error;
pub mod error_code;
pub mod functions;
pub mod numeric;
pub mod text;

/// Re-exported through sheni from the parser: a read takes the node the
/// parser produced, so a consumer needs the node type without depending on
/// `syon-parser` by name.
pub use sheni_types::Value;

pub use elementary::{ElementaryClass, ElementaryType, ElementaryValue};
pub use error::{Result, ShlitaError};
pub use error_code::ShlitaCode;
pub use functions::{call, Arity, StandardFunction, MAX_STRING_LENGTH};
