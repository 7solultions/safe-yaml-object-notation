//! The elementary types of IEC 61131-3, third edition, and their values.
//!
//! Twenty-seven types in eleven classes. The classes are the standard's own
//! generic type hierarchy -- ANY_INT, ANY_BIT, ANY_REAL and the rest -- kept
//! because the standard functions are defined over the generics rather than
//! over each concrete type, and a function that says "any bit string" needs
//! something to ask.
//!
//! The vocabulary is this crate's own rather than sheni's, for the three
//! reasons ADR shlita_01 gives: `byte` in sheni is a character and BYTE here
//! is an eight-bit string that may be ANDed; TIME admits a sign, a fraction
//! and an overflowing leading unit that sheni's `duration` refuses; and the
//! bit strings are types distinct from the unsigned integers of the same
//! width, which a mapping onto `u8`..`u64` would discard. The conversions
//! live in [`crate::convert`] instead.
//!
//! Reading a literal is checked, never guessed. The accepted forms are the
//! standard's, and anything outside them is a [`ShlitaError`] carrying a
//! [`ShlitaCode`].

use std::fmt;

use serde::{Deserialize, Serialize};
use sheni_types::Value;

use crate::datetime;
use crate::duration;
use crate::error::{Result, ShlitaError};
use crate::error_code::ShlitaCode;
use crate::numeric;
use crate::text;

/// The class an elementary type belongs to.
///
/// These are the leaves of the standard's generic hierarchy. The interior
/// nodes -- ANY_INT, ANY_NUM, ANY_MAGNITUDE, ANY_BIT, ANY_DATE -- are the
/// `is_any_*` predicates on [`ElementaryType`], because a class is what a
/// type *is* and a generic is a set a type belongs to, and those are not the
/// same question.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementaryClass {
    /// BOOL. A single bit, and the narrowest member of ANY_BIT.
    Bit,
    /// SINT, INT, DINT, LINT.
    SignedInteger,
    /// USINT, UINT, UDINT, ULINT.
    UnsignedInteger,
    /// BYTE, WORD, DWORD, LWORD.
    BitString,
    /// REAL, LREAL.
    Real,
    /// TIME, LTIME. A signed span, not a point.
    Duration,
    /// DATE, LDATE.
    Date,
    /// TIME_OF_DAY, LTIME_OF_DAY.
    TimeOfDay,
    /// DATE_AND_TIME, LDATE_AND_TIME.
    DateAndTime,
    /// CHAR, WCHAR.
    Character,
    /// STRING, WSTRING.
    CharacterString,
}

/// An elementary type.
///
/// Serialises as its canonical name -- `"DINT"`, `"TIME_OF_DAY"` -- so a
/// schema written in SYON names the type the way a PLC engineer writes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum ElementaryType {
    /// `BOOL`. `0`, `1`, `TRUE`, `FALSE`.
    Bool,
    /// `SINT`. 8-bit signed.
    Sint,
    /// `INT`. 16-bit signed.
    Int,
    /// `DINT`. 32-bit signed.
    Dint,
    /// `LINT`. 64-bit signed.
    Lint,
    /// `USINT`. 8-bit unsigned.
    Usint,
    /// `UINT`. 16-bit unsigned.
    Uint,
    /// `UDINT`. 32-bit unsigned.
    Udint,
    /// `ULINT`. 64-bit unsigned.
    Ulint,
    /// `BYTE`. An 8-bit string.
    Byte,
    /// `WORD`. A 16-bit string.
    Word,
    /// `DWORD`. A 32-bit string.
    Dword,
    /// `LWORD`. A 64-bit string.
    Lword,
    /// `REAL`. Single precision.
    Real,
    /// `LREAL`. Double precision.
    Lreal,
    /// `TIME`. A signed duration to millisecond resolution.
    Time,
    /// `LTIME`. A signed duration to nanosecond resolution.
    Ltime,
    /// `DATE`, alias `D`.
    Date,
    /// `LDATE`.
    Ldate,
    /// `TIME_OF_DAY`, alias `TOD`.
    TimeOfDay,
    /// `LTIME_OF_DAY`, alias `LTOD`.
    LtimeOfDay,
    /// `DATE_AND_TIME`, alias `DT`.
    DateAndTime,
    /// `LDATE_AND_TIME`, alias `LDT`.
    LdateAndTime,
    /// `CHAR`. One single-byte character.
    Char,
    /// `WCHAR`. One UTF-16 code unit.
    Wchar,
    /// `STRING`. Single-quoted, single-byte characters.
    String,
    /// `WSTRING`. Double-quoted, UTF-16 code units.
    WString,
}

impl ElementaryType {
    /// Every elementary type, in the order the standard's table lists them.
    pub const ALL: [ElementaryType; 27] = [
        ElementaryType::Bool,
        ElementaryType::Sint,
        ElementaryType::Int,
        ElementaryType::Dint,
        ElementaryType::Lint,
        ElementaryType::Usint,
        ElementaryType::Uint,
        ElementaryType::Udint,
        ElementaryType::Ulint,
        ElementaryType::Byte,
        ElementaryType::Word,
        ElementaryType::Dword,
        ElementaryType::Lword,
        ElementaryType::Real,
        ElementaryType::Lreal,
        ElementaryType::Time,
        ElementaryType::Ltime,
        ElementaryType::Date,
        ElementaryType::Ldate,
        ElementaryType::TimeOfDay,
        ElementaryType::LtimeOfDay,
        ElementaryType::DateAndTime,
        ElementaryType::LdateAndTime,
        ElementaryType::Char,
        ElementaryType::Wchar,
        ElementaryType::String,
        ElementaryType::WString,
    ];

    /// The type's canonical name, upper case and long form.
    pub const fn name(self) -> &'static str {
        match self {
            ElementaryType::Bool => "BOOL",
            ElementaryType::Sint => "SINT",
            ElementaryType::Int => "INT",
            ElementaryType::Dint => "DINT",
            ElementaryType::Lint => "LINT",
            ElementaryType::Usint => "USINT",
            ElementaryType::Uint => "UINT",
            ElementaryType::Udint => "UDINT",
            ElementaryType::Ulint => "ULINT",
            ElementaryType::Byte => "BYTE",
            ElementaryType::Word => "WORD",
            ElementaryType::Dword => "DWORD",
            ElementaryType::Lword => "LWORD",
            ElementaryType::Real => "REAL",
            ElementaryType::Lreal => "LREAL",
            ElementaryType::Time => "TIME",
            ElementaryType::Ltime => "LTIME",
            ElementaryType::Date => "DATE",
            ElementaryType::Ldate => "LDATE",
            ElementaryType::TimeOfDay => "TIME_OF_DAY",
            ElementaryType::LtimeOfDay => "LTIME_OF_DAY",
            ElementaryType::DateAndTime => "DATE_AND_TIME",
            ElementaryType::LdateAndTime => "LDATE_AND_TIME",
            ElementaryType::Char => "CHAR",
            ElementaryType::Wchar => "WCHAR",
            ElementaryType::String => "STRING",
            ElementaryType::WString => "WSTRING",
        }
    }

    /// The short names the standard also permits, if any.
    ///
    /// `TOD` and `TIME_OF_DAY` are one type spelled two ways, and a document
    /// that uses either has to name the same thing.
    pub const fn aliases(self) -> &'static [&'static str] {
        match self {
            ElementaryType::TimeOfDay => &["TOD"],
            ElementaryType::LtimeOfDay => &["LTOD"],
            ElementaryType::DateAndTime => &["DT"],
            ElementaryType::LdateAndTime => &["LDT"],
            ElementaryType::Date => &["D"],
            ElementaryType::Ldate => &["LD"],
            ElementaryType::Time => &["T"],
            ElementaryType::Ltime => &["LT"],
            _ => &[],
        }
    }

    /// The type named, or `None`.
    ///
    /// The lookup is case-insensitive because IEC 61131-3 keywords are, and
    /// it accepts the short spellings alongside the long ones.
    pub fn from_name(name: &str) -> Option<Self> {
        ElementaryType::ALL.into_iter().find(|t| {
            name.eq_ignore_ascii_case(t.name())
                || t.aliases().iter().any(|a| name.eq_ignore_ascii_case(a))
        })
    }

    /// The class this type belongs to.
    pub const fn class(self) -> ElementaryClass {
        match self {
            ElementaryType::Bool => ElementaryClass::Bit,
            ElementaryType::Sint
            | ElementaryType::Int
            | ElementaryType::Dint
            | ElementaryType::Lint => ElementaryClass::SignedInteger,
            ElementaryType::Usint
            | ElementaryType::Uint
            | ElementaryType::Udint
            | ElementaryType::Ulint => ElementaryClass::UnsignedInteger,
            ElementaryType::Byte
            | ElementaryType::Word
            | ElementaryType::Dword
            | ElementaryType::Lword => ElementaryClass::BitString,
            ElementaryType::Real | ElementaryType::Lreal => ElementaryClass::Real,
            ElementaryType::Time | ElementaryType::Ltime => ElementaryClass::Duration,
            ElementaryType::Date | ElementaryType::Ldate => ElementaryClass::Date,
            ElementaryType::TimeOfDay | ElementaryType::LtimeOfDay => ElementaryClass::TimeOfDay,
            ElementaryType::DateAndTime | ElementaryType::LdateAndTime => {
                ElementaryClass::DateAndTime
            }
            ElementaryType::Char | ElementaryType::Wchar => ElementaryClass::Character,
            ElementaryType::String | ElementaryType::WString => ElementaryClass::CharacterString,
        }
    }

    /// ANY_INT: the signed and unsigned integers, and nothing else. The bit
    /// strings are deliberately outside it.
    pub const fn is_any_int(self) -> bool {
        matches!(
            self.class(),
            ElementaryClass::SignedInteger | ElementaryClass::UnsignedInteger
        )
    }

    /// ANY_REAL.
    pub const fn is_any_real(self) -> bool {
        matches!(self.class(), ElementaryClass::Real)
    }

    /// ANY_NUM: the integers and the reals.
    pub const fn is_any_num(self) -> bool {
        self.is_any_int() || self.is_any_real()
    }

    /// ANY_MAGNITUDE: the numbers and TIME.
    pub const fn is_any_magnitude(self) -> bool {
        self.is_any_num() || matches!(self.class(), ElementaryClass::Duration)
    }

    /// ANY_BIT: BOOL and the bit strings.
    pub const fn is_any_bit(self) -> bool {
        matches!(
            self.class(),
            ElementaryClass::Bit | ElementaryClass::BitString
        )
    }

    /// ANY_DATE: the three date-and-time-of-day types, in both widths.
    pub const fn is_any_date(self) -> bool {
        matches!(
            self.class(),
            ElementaryClass::Date | ElementaryClass::TimeOfDay | ElementaryClass::DateAndTime
        )
    }

    /// ANY_STRING: STRING and WSTRING.
    pub const fn is_any_string(self) -> bool {
        matches!(self.class(), ElementaryClass::CharacterString)
    }

    /// ANY_CHAR: CHAR and WCHAR.
    pub const fn is_any_char(self) -> bool {
        matches!(self.class(), ElementaryClass::Character)
    }

    /// Whether this is one of the third edition's long types -- LINT is not
    /// one, LTIME is. The long types differ from their short twins only in
    /// resolution or range, and the literal readers branch on this rather
    /// than on the type.
    pub const fn is_long(self) -> bool {
        matches!(
            self,
            ElementaryType::Ltime
                | ElementaryType::Ldate
                | ElementaryType::LtimeOfDay
                | ElementaryType::LdateAndTime
        )
    }

    /// The width in bits, for the types that have one.
    pub const fn bit_width(self) -> Option<u32> {
        match self {
            ElementaryType::Bool => Some(1),
            ElementaryType::Sint | ElementaryType::Usint | ElementaryType::Byte => Some(8),
            ElementaryType::Int | ElementaryType::Uint | ElementaryType::Word => Some(16),
            ElementaryType::Dint | ElementaryType::Udint | ElementaryType::Dword => Some(32),
            ElementaryType::Lint | ElementaryType::Ulint | ElementaryType::Lword => Some(64),
            ElementaryType::Real => Some(32),
            ElementaryType::Lreal => Some(64),
            _ => None,
        }
    }

    /// The closed range of a signed integer type.
    pub const fn signed_range(self) -> Option<(i64, i64)> {
        match self {
            ElementaryType::Sint => Some((i8::MIN as i64, i8::MAX as i64)),
            ElementaryType::Int => Some((i16::MIN as i64, i16::MAX as i64)),
            ElementaryType::Dint => Some((i32::MIN as i64, i32::MAX as i64)),
            ElementaryType::Lint => Some((i64::MIN, i64::MAX)),
            _ => None,
        }
    }

    /// The largest value an unsigned integer or bit string type can hold.
    pub const fn unsigned_max(self) -> Option<u64> {
        match self {
            ElementaryType::Bool => Some(1),
            ElementaryType::Usint | ElementaryType::Byte => Some(u8::MAX as u64),
            ElementaryType::Uint | ElementaryType::Word => Some(u16::MAX as u64),
            ElementaryType::Udint | ElementaryType::Dword => Some(u32::MAX as u64),
            ElementaryType::Ulint | ElementaryType::Lword => Some(u64::MAX),
            _ => None,
        }
    }

    /// The value the standard initialises a variable of this type to when
    /// the declaration gives no other. Every elementary type has one, and
    /// none of them is "undefined" -- which is why a controller starts in a
    /// known state.
    pub fn default_value(self) -> ElementaryValue {
        match self.class() {
            ElementaryClass::Bit => ElementaryValue::Bool(false),
            ElementaryClass::SignedInteger => ElementaryValue::Signed { ty: self, value: 0 },
            ElementaryClass::UnsignedInteger => ElementaryValue::Unsigned { ty: self, value: 0 },
            ElementaryClass::BitString => ElementaryValue::Bits { ty: self, value: 0 },
            ElementaryClass::Real => ElementaryValue::Real {
                ty: self,
                value: 0.0,
            },
            ElementaryClass::Duration => ElementaryValue::Duration { ty: self, nanos: 0 },
            // The standard's default for the date types is the epoch itself.
            ElementaryClass::Date => ElementaryValue::Date { ty: self, days: 0 },
            ElementaryClass::TimeOfDay => ElementaryValue::TimeOfDay { ty: self, nanos: 0 },
            ElementaryClass::DateAndTime => ElementaryValue::DateAndTime { ty: self, nanos: 0 },
            ElementaryClass::Character => ElementaryValue::Char { ty: self, code: 0 },
            ElementaryClass::CharacterString => ElementaryValue::Text {
                ty: self,
                value: std::string::String::new(),
            },
        }
    }

    /// Read a literal at this type.
    ///
    /// The text is taken exactly as the parser produced it. Surrounding
    /// whitespace is content as far as SYON is concerned, and trimming it
    /// here would be the kind of coercion this layer exists to prevent.
    pub fn read(self, literal: &str) -> Result<ElementaryValue> {
        if literal.is_empty() {
            return Err(self.err(
                ShlitaCode::EmptyLiteral,
                literal,
                "expected a value, found empty text",
            ));
        }
        match self.class() {
            ElementaryClass::Bit => numeric::read_bool(self, literal),
            ElementaryClass::SignedInteger
            | ElementaryClass::UnsignedInteger
            | ElementaryClass::BitString => numeric::read_integer(self, literal),
            ElementaryClass::Real => numeric::read_real(self, literal),
            ElementaryClass::Duration => duration::read(self, literal),
            ElementaryClass::Date => datetime::read_date(self, literal),
            ElementaryClass::TimeOfDay => datetime::read_time_of_day(self, literal),
            ElementaryClass::DateAndTime => datetime::read_date_and_time(self, literal),
            ElementaryClass::Character => text::read_char(self, literal),
            ElementaryClass::CharacterString => text::read_string(self, literal),
        }
    }

    /// Read a parsed SYON node at this type.
    ///
    /// A mapping or a sequence is not a value of an elementary type, and
    /// says so rather than being flattened into one.
    pub fn read_value(self, value: &Value) -> Result<ElementaryValue> {
        match value {
            Value::Scalar(text) | Value::LiteralBlock(text) => self.read(text),
            Value::Mapping(_) => Err(self.err(
                ShlitaCode::NotAScalar,
                "",
                "expected a single value, found a mapping",
            )),
            Value::Sequence(_) => Err(self.err(
                ShlitaCode::NotAScalar,
                "",
                "expected a single value, found a sequence",
            )),
        }
    }

    /// An error at this type, with the failing literal attached.
    pub(crate) fn err(
        self,
        code: ShlitaCode,
        subject: &str,
        message: impl Into<std::string::String>,
    ) -> ShlitaError {
        ShlitaError::new(code, self.name(), subject, message)
    }
}

impl fmt::Display for ElementaryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<ElementaryType> for std::string::String {
    fn from(value: ElementaryType) -> Self {
        value.name().to_string()
    }
}

impl TryFrom<std::string::String> for ElementaryType {
    type Error = ShlitaError;

    fn try_from(value: std::string::String) -> std::result::Result<Self, Self::Error> {
        ElementaryType::from_name(&value).ok_or_else(|| {
            ShlitaError::new(
                ShlitaCode::UnknownTypeName,
                "elementary",
                value,
                "no elementary type by that name",
            )
        })
    }
}

/// A value of an elementary type.
///
/// The type travels with the value, as it does in sheni: an INT and a DINT
/// holding 7 are not equal, because in a controller they are not
/// interchangeable. Bit strings are held apart from unsigned integers of the
/// same width for the same reason -- a WORD may be ANDed and a UINT may not,
/// and a representation that merged them would make that check impossible.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElementaryValue {
    /// BOOL.
    Bool(bool),
    /// SINT, INT, DINT, LINT.
    Signed { ty: ElementaryType, value: i64 },
    /// USINT, UINT, UDINT, ULINT.
    Unsigned { ty: ElementaryType, value: u64 },
    /// BYTE, WORD, DWORD, LWORD, held right-aligned in a `u64`.
    Bits { ty: ElementaryType, value: u64 },
    /// REAL, LREAL. A REAL is held as an `f64` that is exactly
    /// representable as an `f32`, so no rounding is hidden in a widening.
    Real { ty: ElementaryType, value: f64 },
    /// TIME, LTIME. Signed nanoseconds; a TIME is always a whole number of
    /// milliseconds.
    Duration { ty: ElementaryType, nanos: i128 },
    /// DATE, LDATE. Days since 1970-01-01.
    Date { ty: ElementaryType, days: i32 },
    /// TIME_OF_DAY, LTIME_OF_DAY. Nanoseconds since midnight.
    TimeOfDay { ty: ElementaryType, nanos: u64 },
    /// DATE_AND_TIME, LDATE_AND_TIME. Nanoseconds since 1970-01-01T00:00:00.
    DateAndTime { ty: ElementaryType, nanos: i128 },
    /// CHAR, WCHAR, held as the code point.
    Char { ty: ElementaryType, code: u32 },
    /// STRING, WSTRING.
    Text {
        ty: ElementaryType,
        value: std::string::String,
    },
}

impl ElementaryValue {
    /// The type this value was read at.
    pub fn type_of(&self) -> ElementaryType {
        match self {
            ElementaryValue::Bool(_) => ElementaryType::Bool,
            ElementaryValue::Signed { ty, .. }
            | ElementaryValue::Unsigned { ty, .. }
            | ElementaryValue::Bits { ty, .. }
            | ElementaryValue::Real { ty, .. }
            | ElementaryValue::Duration { ty, .. }
            | ElementaryValue::Date { ty, .. }
            | ElementaryValue::TimeOfDay { ty, .. }
            | ElementaryValue::DateAndTime { ty, .. }
            | ElementaryValue::Char { ty, .. }
            | ElementaryValue::Text { ty, .. } => *ty,
        }
    }

    /// The bits of an ANY_BIT value, with BOOL counting as one bit.
    ///
    /// Returns `None` for everything else, which is what keeps `AND` off the
    /// unsigned integers.
    pub fn as_bits(&self) -> Option<u64> {
        match self {
            ElementaryValue::Bool(b) => Some(u64::from(*b)),
            ElementaryValue::Bits { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// The value as an `f64`, for the numeric functions. Exact for every
    /// integer type up to 2^53 and for both reals; wider LINT and ULINT
    /// magnitudes lose precision, which is why the arithmetic functions work
    /// on the integer representation and only the transcendental ones use
    /// this.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ElementaryValue::Signed { value, .. } => Some(*value as f64),
            ElementaryValue::Unsigned { value, .. } => Some(*value as f64),
            ElementaryValue::Real { value, .. } => Some(*value),
            _ => None,
        }
    }

    /// The value as an `i128`, for the integer functions.
    pub fn as_i128(&self) -> Option<i128> {
        match self {
            ElementaryValue::Signed { value, .. } => Some(i128::from(*value)),
            ElementaryValue::Unsigned { value, .. } => Some(i128::from(*value)),
            ElementaryValue::Bits { value, .. } => Some(i128::from(*value)),
            _ => None,
        }
    }

    /// The text of a STRING or WSTRING.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            ElementaryValue::Text { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Build a value of `ty` from an integer, checking the range.
    ///
    /// This is the one place a computed integer becomes a value, so
    /// [`ShlitaCode::ArithmeticOverflow`] is raised here rather than in each
    /// function that can overflow.
    pub fn from_i128(ty: ElementaryType, value: i128, context: &str) -> Result<Self> {
        match ty.class() {
            ElementaryClass::SignedInteger => {
                let (low, high) = ty.signed_range().expect("a signed integer has a range");
                if value < i128::from(low) || value > i128::from(high) {
                    return Err(ShlitaError::new(
                        ShlitaCode::ArithmeticOverflow,
                        context,
                        value.to_string(),
                        format!("the range of {ty} is {low}..={high}"),
                    ));
                }
                Ok(ElementaryValue::Signed {
                    ty,
                    value: value as i64,
                })
            }
            ElementaryClass::UnsignedInteger | ElementaryClass::BitString => {
                let max = ty.unsigned_max().expect("an unsigned type has a maximum");
                if value < 0 || value > i128::from(max) {
                    return Err(ShlitaError::new(
                        ShlitaCode::ArithmeticOverflow,
                        context,
                        value.to_string(),
                        format!("the range of {ty} is 0..={max}"),
                    ));
                }
                let value = value as u64;
                Ok(if ty.class() == ElementaryClass::BitString {
                    ElementaryValue::Bits { ty, value }
                } else {
                    ElementaryValue::Unsigned { ty, value }
                })
            }
            ElementaryClass::Bit => Ok(ElementaryValue::Bool(value != 0)),
            _ => Err(ShlitaError::new(
                ShlitaCode::TypeMismatch,
                context,
                value.to_string(),
                format!("{ty} is not an integer type"),
            )),
        }
    }

    /// Build a REAL or LREAL, checking that a finite result stays finite at
    /// the type's width.
    pub fn from_f64(ty: ElementaryType, value: f64, context: &str) -> Result<Self> {
        if !value.is_finite() {
            return Err(ShlitaError::new(
                ShlitaCode::NonFiniteReal,
                context,
                value.to_string(),
                "the result is not a finite number",
            ));
        }
        if ty == ElementaryType::Real {
            let narrowed = value as f32;
            if !narrowed.is_finite() {
                return Err(ShlitaError::new(
                    ShlitaCode::RealOutOfRange,
                    context,
                    value.to_string(),
                    "the result overflows the range of REAL",
                ));
            }
            return Ok(ElementaryValue::Real {
                ty,
                value: f64::from(narrowed),
            });
        }
        Ok(ElementaryValue::Real { ty, value })
    }
}

/// The canonical text form, which reads back at the same type as an equal
/// value.
///
/// Bit strings print in hexadecimal, zero-padded to their width, because a
/// bit string written in decimal is a bit string a reader has to convert in
/// their head before they can see which bits are set.
impl fmt::Display for ElementaryValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ElementaryValue::Bool(b) => f.write_str(if *b { "TRUE" } else { "FALSE" }),
            ElementaryValue::Signed { value, .. } => write!(f, "{value}"),
            ElementaryValue::Unsigned { value, .. } => write!(f, "{value}"),
            ElementaryValue::Bits { ty, value } => {
                let digits = ty.bit_width().unwrap_or(8) as usize / 4;
                write!(f, "16#{value:0digits$X}")
            }
            ElementaryValue::Real { value, .. } => {
                // A REAL that happens to be integral still has to read back
                // as a REAL, and `1` would read back as an integer. Debug
                // formatting is what always writes the point or an exponent.
                write!(f, "{value:?}")
            }
            ElementaryValue::Duration { ty, nanos } => duration::format(*ty, *nanos, f),
            ElementaryValue::Date { ty, days } => datetime::format_date(*ty, *days, f),
            ElementaryValue::TimeOfDay { ty, nanos } => {
                datetime::format_time_of_day(*ty, *nanos, f)
            }
            ElementaryValue::DateAndTime { ty, nanos } => {
                datetime::format_date_and_time(*ty, *nanos, f)
            }
            ElementaryValue::Char { ty, code } => text::format_char(*ty, *code, f),
            ElementaryValue::Text { ty, value } => text::format_string(*ty, value, f),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_type_has_a_unique_name() {
        let mut names: Vec<&str> = ElementaryType::ALL.iter().map(|t| t.name()).collect();
        names.sort_unstable();
        let count = names.len();
        names.dedup();
        assert_eq!(names.len(), count);
    }

    #[test]
    fn names_and_aliases_both_resolve_and_keywords_are_case_insensitive() {
        for ty in ElementaryType::ALL {
            assert_eq!(ElementaryType::from_name(ty.name()), Some(ty));
            assert_eq!(
                ElementaryType::from_name(&ty.name().to_lowercase()),
                Some(ty)
            );
            for alias in ty.aliases() {
                assert_eq!(ElementaryType::from_name(alias), Some(ty));
            }
        }
        assert_eq!(
            ElementaryType::from_name("tod"),
            Some(ElementaryType::TimeOfDay)
        );
        assert_eq!(ElementaryType::from_name("REALLY"), None);
    }

    /// The collision ADR shlita_01 refused to let happen: a bit string and
    /// the unsigned integer of the same width are different types, and a
    /// value of one is not a value of the other.
    #[test]
    fn a_bit_string_is_not_the_unsigned_integer_of_the_same_width() {
        let word = ElementaryType::Word.read("16#00FF").unwrap();
        let uint = ElementaryType::Uint.read("255").unwrap();
        assert_ne!(word, uint);
        assert_eq!(word.as_bits(), Some(255));
        assert_eq!(uint.as_bits(), None);
    }

    #[test]
    fn the_generics_group_the_types_the_way_the_standard_does() {
        assert!(ElementaryType::Byte.is_any_bit());
        assert!(ElementaryType::Bool.is_any_bit());
        assert!(!ElementaryType::Usint.is_any_bit());
        assert!(ElementaryType::Usint.is_any_int());
        assert!(!ElementaryType::Byte.is_any_int());
        assert!(ElementaryType::Time.is_any_magnitude());
        assert!(!ElementaryType::Time.is_any_num());
        assert!(ElementaryType::DateAndTime.is_any_date());
    }

    #[test]
    fn an_empty_literal_is_an_error_at_every_type_including_string() {
        for ty in ElementaryType::ALL {
            assert_eq!(
                ty.read("").unwrap_err().code(),
                ShlitaCode::EmptyLiteral,
                "{ty} accepted an empty literal"
            );
        }
    }

    #[test]
    fn a_mapping_is_not_a_value() {
        let node = Value::Mapping(Vec::new());
        assert_eq!(
            ElementaryType::Int.read_value(&node).unwrap_err().code(),
            ShlitaCode::NotAScalar
        );
    }

    #[test]
    fn a_scalar_node_reads_like_its_text() {
        let node = Value::Scalar("42".to_string());
        assert_eq!(
            ElementaryType::Int.read_value(&node),
            Ok(ElementaryValue::Signed {
                ty: ElementaryType::Int,
                value: 42
            })
        );
    }

    #[test]
    fn every_type_has_a_default_and_it_is_of_that_type() {
        for ty in ElementaryType::ALL {
            assert_eq!(ty.default_value().type_of(), ty, "{ty}");
        }
        assert_eq!(
            ElementaryType::Bool.default_value(),
            ElementaryValue::Bool(false)
        );
    }

    /// Canonical text is text that reads back. Every value this crate can
    /// print has to survive the round trip, or the printer and the reader
    /// disagree about the language.
    #[test]
    fn canonical_text_reads_back_as_an_equal_value() {
        let samples = [
            (ElementaryType::Bool, "TRUE"),
            (ElementaryType::Sint, "-128"),
            (ElementaryType::Int, "32767"),
            (ElementaryType::Dint, "-1"),
            (ElementaryType::Lint, "9223372036854775807"),
            (ElementaryType::Usint, "255"),
            (ElementaryType::Ulint, "18446744073709551615"),
            (ElementaryType::Byte, "16#FF"),
            (ElementaryType::Word, "2#1010_1010"),
            (ElementaryType::Dword, "16#DEAD_BEEF"),
            (ElementaryType::Real, "3.5"),
            (ElementaryType::Lreal, "-1.25e3"),
            (ElementaryType::Time, "T#1d2h3m4s5ms"),
            (ElementaryType::Ltime, "LTIME#-1s500us"),
            (ElementaryType::Date, "D#2026-08-29"),
            (ElementaryType::TimeOfDay, "TOD#23:59:59.999"),
            (ElementaryType::DateAndTime, "DT#2026-08-29-12:00:00"),
            (ElementaryType::Char, "CHAR#'A'"),
            (ElementaryType::Wchar, "WCHAR#\"A\""),
            (ElementaryType::String, "'a b$$c'"),
            (ElementaryType::WString, "\"wide\""),
        ];
        for (ty, literal) in samples {
            let value = ty
                .read(literal)
                .unwrap_or_else(|e| panic!("{ty} rejected {literal}: {e}"));
            let printed = value.to_string();
            let reread = ty
                .read(&printed)
                .unwrap_or_else(|e| panic!("{ty} rejected its own output {printed}: {e}"));
            assert_eq!(value, reread, "{ty} did not round trip through {printed}");
        }
    }
}
