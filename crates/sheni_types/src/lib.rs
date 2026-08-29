//! Sheni -- the type layer over SYON.
//!
//! SYON parses; it does not interpret. A successful parse hands back
//! [`syon_parser::Value::Scalar`] -- the text between the delimiters,
//! unchanged and untyped. Sheni is the layer that says what that text means.
//!
//! Types fall into four groups ([`TypeGroup`]), each built on the one before
//! it:
//!
//! 1. **primitive** -- booleans, numbers, characters, text. Implemented, see
//!    [`PrimitiveType`], and [`SoftPrimitiveType`] for the twin of each that
//!    admits an unknown.
//! 2. **simple** -- an interpretation over a primitive carrier: a date, an
//!    email address, a currency code. Implemented, see [`SimpleType`].
//! 3. **complex** -- composition: enums and structs. Not implemented yet.
//! 4. **collection** -- lists and maps. Not implemented yet.
//!
//! Reading a literal is checked rather than guessed: the accepted text forms
//! are a closed set, and anything outside it is a [`TypeError`] with a stable
//! [`SheniCode`]. That is what keeps an unquoted `no` in a string field a
//! string.
//!
//! The two implemented groups differ in what a read preserves. A primitive
//! value **is** its text, so the text survives exactly. A simple value is the
//! thing its text denotes, so the meaning survives and the canonical spelling
//! is what comes back -- see [`simple`].
//!
//! ```
//! use sheni_types::{PrimitiveType, PrimitiveValue, SheniCode, TypeGroup};
//!
//! let u8_type = PrimitiveType::from_name("u8").unwrap();
//! assert_eq!(u8_type.group(), TypeGroup::Primitive);
//! assert_eq!(
//!     u8_type.read("42"),
//!     Ok(PrimitiveValue::Unsigned {
//!         width: sheni_types::IntWidth::W8,
//!         value: 42,
//!     })
//! );
//!
//! // 300 does not fit, and says so rather than wrapping.
//! assert_eq!(u8_type.read("300").unwrap_err().code(), SheniCode::IntegerOutOfRange);
//!
//! // `no` is a boolean only where a boolean was declared.
//! assert_eq!(PrimitiveType::Boolean.read("no"), Ok(PrimitiveValue::Boolean(false)));
//! assert_eq!(
//!     PrimitiveType::String.read("no"),
//!     Ok(PrimitiveValue::String("no".to_string()))
//! );
//! ```
//!
//! Every primitive except `string` has a soft twin, which accepts its strict
//! twin's literals and the word `unknown` and nothing else. That is what lets
//! a field be optional without a sentinel: an absent `soft_u32` reads as
//! unknown rather than as a count of zero.
//!
//! ```
//! use sheni_types::{PrimitiveType, SoftPrimitiveType};
//!
//! let count = SoftPrimitiveType::from_name("soft_u32").unwrap();
//! assert_eq!(count.fallback().to_string(), "unknown");
//! assert_ne!(count.fallback(), count.read("0").unwrap());
//!
//! // `string` takes any text verbatim, so it has no word left to mean
//! // "not known" and therefore no soft twin.
//! assert_eq!(SoftPrimitiveType::new(PrimitiveType::String), None);
//! ```
//!
//! Simple types delegate to the crate that implements their standard, and
//! read back canonically:
//!
//! ```
//! use sheni_types::{SheniCode, SimpleType, TypeGroup};
//!
//! let ip = SimpleType::from_name("ip_address").unwrap();
//! assert_eq!(ip.group(), TypeGroup::Simple);
//! // RFC 5952 says the canonical text form is lowercase and compressed.
//! assert_eq!(ip.read("2001:0DB8::1").unwrap().to_string(), "2001:db8::1");
//!
//! // A URL has to be absolute, and says so specifically.
//! assert_eq!(
//!     SimpleType::Url.read("/relative").unwrap_err().code(),
//!     SheniCode::RelativeUrl
//! );
//! ```

pub mod error;
pub mod error_code;
pub mod group;
pub mod primitives;
pub mod simple;
pub mod soft;

/// Re-exported from the parser: a read takes the node the parser produced,
/// so a consumer of this crate needs the node type without depending on
/// `syon-parser` by name.
pub use syon_parser::Value;

/// Re-exported from `edtf-core`: the shapes `soft_date` and
/// `soft_date_range` hand back, so a caller can name a precision or walk
/// an interval's endpoints without depending on the delegate by name.
pub use edtf_core::{Edtf, Interval, IntervalEndpoint, Precision};
pub use error::TypeError;
pub use error_code::SheniCode;
pub use group::TypeGroup;
pub use primitives::{FloatWidth, IntWidth, PrimitiveType, PrimitiveValue};
pub use simple::{LanguageTag, SimpleType, SimpleValue};
pub use soft::{SoftPrimitiveType, SoftPrimitiveValue, UNKNOWN_LITERAL};
