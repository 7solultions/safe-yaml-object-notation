//! The primitive group: values that carry no interpretation beyond their own
//! shape.
//!
//! Seven kinds -- boolean, unsigned integer, signed integer, float, byte
//! character, unicode character, string -- spelled the way Rust spells them,
//! with the width of a number part of its type rather than an attribute
//! alongside it.
//!
//! Reading a literal is checked, not guessed. The accepted forms are a closed
//! set fixed by `design/architecture/ADR_sheni_01__primitives.syon`; anything
//! outside it is a [`TypeError`] carrying a [`SheniCode`], never a silent
//! coercion.

use std::fmt;

use serde::{Deserialize, Serialize};
use syon_parser::Value;

use crate::error::TypeError;
use crate::error_code::SheniCode;
use crate::group::TypeGroup;

/// The width of an integer type, in bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum IntWidth {
    W8 = 8,
    W16 = 16,
    W32 = 32,
    W64 = 64,
    W128 = 128,
}

impl IntWidth {
    /// Every width, narrowest first.
    pub const ALL: [IntWidth; 5] = [
        IntWidth::W8,
        IntWidth::W16,
        IntWidth::W32,
        IntWidth::W64,
        IntWidth::W128,
    ];

    pub const fn bits(self) -> u32 {
        self as u32
    }

    /// The largest value an unsigned integer of this width can hold.
    pub const fn unsigned_max(self) -> u128 {
        match self {
            IntWidth::W128 => u128::MAX,
            _ => (1u128 << self.bits()) - 1,
        }
    }

    /// The most negative value a signed integer of this width can hold.
    pub const fn signed_min(self) -> i128 {
        match self {
            IntWidth::W128 => i128::MIN,
            _ => -(1i128 << (self.bits() - 1)),
        }
    }

    /// The largest value a signed integer of this width can hold.
    pub const fn signed_max(self) -> i128 {
        match self {
            IntWidth::W128 => i128::MAX,
            _ => (1i128 << (self.bits() - 1)) - 1,
        }
    }
}

/// The width of a float type, in bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum FloatWidth {
    W32 = 32,
    W64 = 64,
}

impl FloatWidth {
    /// Every width, narrowest first.
    pub const ALL: [FloatWidth; 2] = [FloatWidth::W32, FloatWidth::W64];

    pub const fn bits(self) -> u32 {
        self as u32
    }
}

/// A primitive type.
///
/// Serialises as its name -- `"u8"`, `"bool"` -- rather than as a tagged
/// enum, so a schema written in SYON names the type the same way the Rust
/// code does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum PrimitiveType {
    /// `bool`. Accepts `true`, `false`, `yes`, `no`, case-insensitively.
    Boolean,
    /// `u8` .. `u128`. Non-negative decimal integers.
    Unsigned(IntWidth),
    /// `i8` .. `i128`. Decimal integers, optionally negated.
    Signed(IntWidth),
    /// `f32` / `f64`. Finite decimal floats.
    Float(FloatWidth),
    /// `byte`. One character whose code point fits in `0..=255`.
    Byte,
    /// `char`. Exactly one Unicode scalar value.
    Char,
    /// `string`. Any well-formed UTF-8 text, taken verbatim.
    String,
}

impl PrimitiveType {
    /// Every primitive type, in the order the ADR lists them.
    pub const ALL: [PrimitiveType; 16] = [
        PrimitiveType::Boolean,
        PrimitiveType::Unsigned(IntWidth::W8),
        PrimitiveType::Unsigned(IntWidth::W16),
        PrimitiveType::Unsigned(IntWidth::W32),
        PrimitiveType::Unsigned(IntWidth::W64),
        PrimitiveType::Unsigned(IntWidth::W128),
        PrimitiveType::Signed(IntWidth::W8),
        PrimitiveType::Signed(IntWidth::W16),
        PrimitiveType::Signed(IntWidth::W32),
        PrimitiveType::Signed(IntWidth::W64),
        PrimitiveType::Signed(IntWidth::W128),
        PrimitiveType::Float(FloatWidth::W32),
        PrimitiveType::Float(FloatWidth::W64),
        PrimitiveType::Byte,
        PrimitiveType::Char,
        PrimitiveType::String,
    ];

    /// Always [`TypeGroup::Primitive`]. Present so a caller holding some type
    /// descriptor can ask its group without knowing which group it is.
    pub const fn group(self) -> TypeGroup {
        TypeGroup::Primitive
    }

    /// The type's name as it is written in a schema.
    pub const fn name(self) -> &'static str {
        match self {
            PrimitiveType::Boolean => "bool",
            PrimitiveType::Unsigned(IntWidth::W8) => "u8",
            PrimitiveType::Unsigned(IntWidth::W16) => "u16",
            PrimitiveType::Unsigned(IntWidth::W32) => "u32",
            PrimitiveType::Unsigned(IntWidth::W64) => "u64",
            PrimitiveType::Unsigned(IntWidth::W128) => "u128",
            PrimitiveType::Signed(IntWidth::W8) => "i8",
            PrimitiveType::Signed(IntWidth::W16) => "i16",
            PrimitiveType::Signed(IntWidth::W32) => "i32",
            PrimitiveType::Signed(IntWidth::W64) => "i64",
            PrimitiveType::Signed(IntWidth::W128) => "i128",
            PrimitiveType::Float(FloatWidth::W32) => "f32",
            PrimitiveType::Float(FloatWidth::W64) => "f64",
            PrimitiveType::Byte => "byte",
            PrimitiveType::Char => "char",
            PrimitiveType::String => "string",
        }
    }

    /// The reverse of [`Self::name`]. Case-sensitive, for the reason
    /// [`TypeGroup::from_name`] is.
    pub fn from_name(name: &str) -> Option<Self> {
        PrimitiveType::ALL.into_iter().find(|t| t.name() == name)
    }

    /// Read a literal at this type.
    ///
    /// The text is taken exactly as the parser produced it -- no trimming.
    /// Surrounding whitespace is content as far as SYON is concerned, and
    /// quietly discarding it here would be the kind of coercion this layer
    /// exists to avoid.
    pub fn read(self, literal: &str) -> Result<PrimitiveValue, TypeError> {
        if literal.is_empty() && self != PrimitiveType::String {
            return Err(self.err(
                SheniCode::EmptyLiteral,
                literal,
                "expected a value, found empty text",
            ));
        }
        if self != PrimitiveType::String && literal.eq_ignore_ascii_case("unknown") {
            return Err(self.err(
                SheniCode::UnknownAtStrictType,
                literal,
                format!(
                    "`unknown` is a value of `soft_{name}`, not of `{name}`",
                    name = self.name()
                ),
            ));
        }
        match self {
            PrimitiveType::Boolean => self.read_boolean(literal),
            PrimitiveType::Unsigned(width) => self.read_unsigned(literal, width),
            PrimitiveType::Signed(width) => self.read_signed(literal, width),
            PrimitiveType::Float(width) => self.read_float(literal, width),
            PrimitiveType::Byte => self.read_byte(literal),
            PrimitiveType::Char => self.read_char(literal),
            PrimitiveType::String => Ok(PrimitiveValue::String(literal.to_string())),
        }
    }

    /// Read a parsed SYON node at this type.
    ///
    /// A `|` block scalar is text like any other scalar; a mapping or a
    /// sequence is not a single value and fails with
    /// [`SheniCode::NotAScalar`].
    pub fn read_value(self, value: &Value) -> Result<PrimitiveValue, TypeError> {
        match value {
            Value::Scalar(text) | Value::LiteralBlock(text) => self.read(text),
            Value::Mapping(_) => Err(self.err(
                SheniCode::NotAScalar,
                "",
                "expected a single value, found a mapping",
            )),
            Value::Sequence(_) => Err(self.err(
                SheniCode::NotAScalar,
                "",
                "expected a single value, found a sequence",
            )),
        }
    }

    fn read_boolean(self, literal: &str) -> Result<PrimitiveValue, TypeError> {
        if literal.eq_ignore_ascii_case("true") || literal.eq_ignore_ascii_case("yes") {
            return Ok(PrimitiveValue::Boolean(true));
        }
        if literal.eq_ignore_ascii_case("false") || literal.eq_ignore_ascii_case("no") {
            return Ok(PrimitiveValue::Boolean(false));
        }
        if literal.eq_ignore_ascii_case("on") || literal.eq_ignore_ascii_case("off") {
            return Err(self.err(
                SheniCode::ReservedBooleanSpelling,
                literal,
                "`on` and `off` are reserved and not accepted as booleans; write `yes` or `no`",
            ));
        }
        Err(self.err(
            SheniCode::NotABoolean,
            literal,
            "expected one of `true`, `false`, `yes`, `no`",
        ))
    }

    /// The checks shared by both integer signednesses, run before either one
    /// parses: separators, a leading `+`, digits only, and leading zeros.
    /// Returns the digits with any sign stripped.
    fn check_integer_shape(self, literal: &str) -> Result<&str, TypeError> {
        if literal.contains('_') {
            return Err(self.err(
                SheniCode::DigitSeparator,
                literal,
                "digit separators are not accepted; write the digits alone",
            ));
        }
        if literal.starts_with('+') {
            return Err(self.err(
                SheniCode::LeadingPlus,
                literal,
                "a leading `+` is not accepted; a bare number is already positive",
            ));
        }
        let digits = literal.strip_prefix('-').unwrap_or(literal);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(self.err(SheniCode::NotAnInteger, literal, "expected decimal digits"));
        }
        if digits.len() > 1 && digits.starts_with('0') {
            return Err(self.err(
                SheniCode::LeadingZero,
                literal,
                "leading zeros are not accepted; they read as octal elsewhere",
            ));
        }
        Ok(digits)
    }

    fn read_unsigned(self, literal: &str, width: IntWidth) -> Result<PrimitiveValue, TypeError> {
        let digits = self.check_integer_shape(literal)?;
        if literal.starts_with('-') {
            return Err(self.err(
                SheniCode::NegativeInUnsigned,
                literal,
                format!("{} holds no negative values", self.name()),
            ));
        }
        // Every byte is a digit by now, so a parse failure is an overflow of
        // u128 itself -- which is out of range for every unsigned width.
        let value: u128 = digits
            .parse()
            .map_err(|_| self.out_of_range(literal, width.unsigned_max()))?;
        if value > width.unsigned_max() {
            return Err(self.out_of_range(literal, width.unsigned_max()));
        }
        Ok(PrimitiveValue::Unsigned { width, value })
    }

    fn read_signed(self, literal: &str, width: IntWidth) -> Result<PrimitiveValue, TypeError> {
        self.check_integer_shape(literal)?;
        let value: i128 = literal
            .parse()
            .map_err(|_| self.signed_out_of_range(literal, width))?;
        if value < width.signed_min() || value > width.signed_max() {
            return Err(self.signed_out_of_range(literal, width));
        }
        Ok(PrimitiveValue::Signed { width, value })
    }

    /// Floats are checked for the forms that do not round-trip -- separators,
    /// a leading `+`, `NaN`, the infinities -- and otherwise read with Rust's
    /// own float grammar. The leading-zero rule is an integer rule: `0.5`
    /// needs its zero.
    fn read_float(self, literal: &str, width: FloatWidth) -> Result<PrimitiveValue, TypeError> {
        if literal.contains('_') {
            return Err(self.err(
                SheniCode::DigitSeparator,
                literal,
                "digit separators are not accepted; write the digits alone",
            ));
        }
        if literal.starts_with('+') {
            return Err(self.err(
                SheniCode::LeadingPlus,
                literal,
                "a leading `+` is not accepted; a bare number is already positive",
            ));
        }
        let unsigned = literal.strip_prefix('-').unwrap_or(literal);
        if unsigned.eq_ignore_ascii_case("nan")
            || unsigned.eq_ignore_ascii_case("inf")
            || unsigned.eq_ignore_ascii_case("infinity")
        {
            return Err(self.err(
                SheniCode::NonFiniteFloat,
                literal,
                "NaN and the infinities have no portable text form and are not accepted",
            ));
        }
        let value = match width {
            FloatWidth::W32 => literal.parse::<f32>().map(f64::from).map_err(|_| ()),
            FloatWidth::W64 => literal.parse::<f64>().map_err(|_| ()),
        }
        .map_err(|_| {
            self.err(
                SheniCode::NotAFloat,
                literal,
                "expected a decimal float, optionally with an exponent",
            )
        })?;
        if !value.is_finite() {
            return Err(self.err(
                SheniCode::FloatOutOfRange,
                literal,
                format!("magnitude too large for {}", self.name()),
            ));
        }
        Ok(PrimitiveValue::Float { width, value })
    }

    fn read_byte(self, literal: &str) -> Result<PrimitiveValue, TypeError> {
        let c = self.single_char(literal)?;
        u8::try_from(u32::from(c))
            .map(PrimitiveValue::Byte)
            .map_err(|_| {
                self.err(
                    SheniCode::ByteOutOfRange,
                    literal,
                    "code point does not fit in 0..=255; use `char` for text beyond Latin-1",
                )
            })
    }

    fn read_char(self, literal: &str) -> Result<PrimitiveValue, TypeError> {
        self.single_char(literal).map(PrimitiveValue::Char)
    }

    fn single_char(self, literal: &str) -> Result<char, TypeError> {
        let mut chars = literal.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(self.err(
                SheniCode::NotASingleCharacter,
                literal,
                "expected exactly one character",
            )),
        }
    }

    fn out_of_range(self, literal: &str, max: u128) -> TypeError {
        self.err(
            SheniCode::IntegerOutOfRange,
            literal,
            format!("outside 0..={max}"),
        )
    }

    fn signed_out_of_range(self, literal: &str, width: IntWidth) -> TypeError {
        self.err(
            SheniCode::IntegerOutOfRange,
            literal,
            format!("outside {}..={}", width.signed_min(), width.signed_max()),
        )
    }

    fn err(self, code: SheniCode, literal: &str, message: impl Into<String>) -> TypeError {
        TypeError::new(code, self.name(), literal, message)
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<PrimitiveType> for String {
    fn from(value: PrimitiveType) -> Self {
        value.name().to_string()
    }
}

impl TryFrom<String> for PrimitiveType {
    type Error = TypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        PrimitiveType::from_name(&value).ok_or_else(|| {
            TypeError::new(
                SheniCode::UnknownTypeName,
                "primitive",
                value,
                "no primitive type by that name",
            )
        })
    }
}

/// A value of a primitive type.
///
/// Integers keep the width they were read at alongside the value, so a `u8`
/// and a `u64` holding 7 are not equal -- the type is part of the value, not
/// something recoverable from its magnitude.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveValue {
    Boolean(bool),
    Unsigned { width: IntWidth, value: u128 },
    Signed { width: IntWidth, value: i128 },
    Float { width: FloatWidth, value: f64 },
    Byte(u8),
    Char(char),
    String(String),
}

impl PrimitiveValue {
    /// The type this value was read at.
    pub fn type_of(&self) -> PrimitiveType {
        match self {
            PrimitiveValue::Boolean(_) => PrimitiveType::Boolean,
            PrimitiveValue::Unsigned { width, .. } => PrimitiveType::Unsigned(*width),
            PrimitiveValue::Signed { width, .. } => PrimitiveType::Signed(*width),
            PrimitiveValue::Float { width, .. } => PrimitiveType::Float(*width),
            PrimitiveValue::Byte(_) => PrimitiveType::Byte,
            PrimitiveValue::Char(_) => PrimitiveType::Char,
            PrimitiveValue::String(_) => PrimitiveType::String,
        }
    }
}

/// The canonical text form, which reads back at the same type as an equal
/// value. Booleans canonicalise to `true` / `false`, not `yes` / `no`.
impl fmt::Display for PrimitiveValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrimitiveValue::Boolean(b) => write!(f, "{b}"),
            PrimitiveValue::Unsigned { value, .. } => write!(f, "{value}"),
            PrimitiveValue::Signed { value, .. } => write!(f, "{value}"),
            PrimitiveValue::Float { value, .. } => write!(f, "{value}"),
            PrimitiveValue::Byte(b) => write!(f, "{}", char::from(*b)),
            PrimitiveValue::Char(c) => write!(f, "{c}"),
            PrimitiveValue::String(s) => f.write_str(s),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code(t: PrimitiveType, literal: &str) -> SheniCode {
        t.read(literal)
            .expect_err("expected this literal to be rejected")
            .code()
    }

    #[test]
    fn every_type_names_itself_and_reads_back() {
        for t in PrimitiveType::ALL {
            assert_eq!(PrimitiveType::from_name(t.name()), Some(t), "{t}");
            assert_eq!(t.group(), TypeGroup::Primitive);
        }
    }

    #[test]
    fn all_covers_every_width() {
        assert_eq!(PrimitiveType::ALL.len(), 16);
        for w in IntWidth::ALL {
            assert!(PrimitiveType::ALL.contains(&PrimitiveType::Unsigned(w)));
            assert!(PrimitiveType::ALL.contains(&PrimitiveType::Signed(w)));
        }
        for w in FloatWidth::ALL {
            assert!(PrimitiveType::ALL.contains(&PrimitiveType::Float(w)));
        }
    }

    #[test]
    fn unknown_type_names_are_rejected() {
        assert_eq!(PrimitiveType::from_name("int"), None);
        assert_eq!(PrimitiveType::from_name("Bool"), None);
        assert_eq!(PrimitiveType::from_name("u24"), None);
        assert_eq!(PrimitiveType::from_name("str"), None);
    }

    // ---- booleans ----

    #[test]
    fn booleans_accept_the_closed_set_case_insensitively() {
        for yes in ["true", "TRUE", "True", "yes", "YES", "Yes"] {
            assert_eq!(
                PrimitiveType::Boolean.read(yes),
                Ok(PrimitiveValue::Boolean(true)),
                "{yes}"
            );
        }
        for no in ["false", "FALSE", "False", "no", "NO", "No"] {
            assert_eq!(
                PrimitiveType::Boolean.read(no),
                Ok(PrimitiveValue::Boolean(false)),
                "{no}"
            );
        }
    }

    #[test]
    fn on_and_off_are_reserved_not_merely_unknown() {
        for reserved in ["on", "off", "ON", "Off"] {
            assert_eq!(
                code(PrimitiveType::Boolean, reserved),
                SheniCode::ReservedBooleanSpelling,
                "{reserved}"
            );
        }
    }

    #[test]
    fn yaml_1_1_boolean_spellings_outside_the_set_are_rejected() {
        for other in ["y", "n", "t", "f", "1", "0", "sí", " yes"] {
            assert_eq!(code(PrimitiveType::Boolean, other), SheniCode::NotABoolean);
        }
    }

    /// The Norway problem: `no` is a boolean only where a boolean is declared.
    #[test]
    fn a_boolean_spelling_read_at_string_stays_text() {
        for text in ["no", "yes", "true", "off"] {
            assert_eq!(
                PrimitiveType::String.read(text),
                Ok(PrimitiveValue::String(text.to_string()))
            );
        }
    }

    // ---- integers ----

    #[test]
    fn unsigned_reads_decimal_digits() {
        assert_eq!(
            PrimitiveType::Unsigned(IntWidth::W8).read("0"),
            Ok(PrimitiveValue::Unsigned {
                width: IntWidth::W8,
                value: 0
            })
        );
        assert_eq!(
            PrimitiveType::Unsigned(IntWidth::W8).read("255"),
            Ok(PrimitiveValue::Unsigned {
                width: IntWidth::W8,
                value: 255
            })
        );
    }

    #[test]
    fn each_unsigned_width_accepts_its_max_and_rejects_one_past_it() {
        for w in IntWidth::ALL {
            let t = PrimitiveType::Unsigned(w);
            let max = w.unsigned_max();
            assert!(t.read(&max.to_string()).is_ok(), "{t} max");
            // u128's max has no successor to test; every narrower width does.
            if w != IntWidth::W128 {
                assert_eq!(
                    code(t, &(max + 1).to_string()),
                    SheniCode::IntegerOutOfRange,
                    "{t} max + 1"
                );
            }
        }
    }

    #[test]
    fn an_integer_too_large_for_u128_is_out_of_range_not_malformed() {
        assert_eq!(
            code(PrimitiveType::Unsigned(IntWidth::W128), &"9".repeat(40)),
            SheniCode::IntegerOutOfRange
        );
    }

    #[test]
    fn signed_reads_a_leading_minus() {
        assert_eq!(
            PrimitiveType::Signed(IntWidth::W8).read("-128"),
            Ok(PrimitiveValue::Signed {
                width: IntWidth::W8,
                value: -128
            })
        );
    }

    #[test]
    fn each_signed_width_accepts_its_bounds_and_rejects_one_past_them() {
        for w in IntWidth::ALL {
            let t = PrimitiveType::Signed(w);
            assert!(t.read(&w.signed_min().to_string()).is_ok(), "{t} min");
            assert!(t.read(&w.signed_max().to_string()).is_ok(), "{t} max");
            if w != IntWidth::W128 {
                assert_eq!(
                    code(t, &(w.signed_max() + 1).to_string()),
                    SheniCode::IntegerOutOfRange,
                    "{t} max + 1"
                );
                assert_eq!(
                    code(t, &(w.signed_min() - 1).to_string()),
                    SheniCode::IntegerOutOfRange,
                    "{t} min - 1"
                );
            }
        }
    }

    #[test]
    fn a_negative_literal_at_an_unsigned_type_says_so_specifically() {
        assert_eq!(
            code(PrimitiveType::Unsigned(IntWidth::W8), "-1"),
            SheniCode::NegativeInUnsigned
        );
    }

    #[test]
    fn integer_shape_rules_each_have_their_own_code() {
        let u32_t = PrimitiveType::Unsigned(IntWidth::W32);
        assert_eq!(code(u32_t, "007"), SheniCode::LeadingZero);
        assert_eq!(code(u32_t, "+7"), SheniCode::LeadingPlus);
        assert_eq!(code(u32_t, "1_000"), SheniCode::DigitSeparator);
        assert_eq!(code(u32_t, "0x1f"), SheniCode::NotAnInteger);
        assert_eq!(code(u32_t, "1.0"), SheniCode::NotAnInteger);
        assert_eq!(code(u32_t, " 7"), SheniCode::NotAnInteger);
        assert_eq!(code(u32_t, "7 "), SheniCode::NotAnInteger);
        assert_eq!(code(u32_t, "seven"), SheniCode::NotAnInteger);
        assert_eq!(code(u32_t, "-"), SheniCode::NotAnInteger);
        assert_eq!(code(u32_t, ""), SheniCode::EmptyLiteral);
    }

    #[test]
    fn a_bare_zero_is_not_a_leading_zero() {
        assert!(PrimitiveType::Unsigned(IntWidth::W8).read("0").is_ok());
        assert!(PrimitiveType::Signed(IntWidth::W8).read("-0").is_ok());
        assert_eq!(
            code(PrimitiveType::Signed(IntWidth::W8), "-00"),
            SheniCode::LeadingZero
        );
    }

    // ---- floats ----

    #[test]
    fn floats_read_decimals_and_exponents() {
        for literal in ["0.0", "-0.5", "1e3", "1.5e-3", "3"] {
            assert!(
                PrimitiveType::Float(FloatWidth::W64).read(literal).is_ok(),
                "{literal}"
            );
        }
    }

    #[test]
    fn floats_keep_their_value() {
        assert_eq!(
            PrimitiveType::Float(FloatWidth::W64).read("0.1"),
            Ok(PrimitiveValue::Float {
                width: FloatWidth::W64,
                value: 0.1
            })
        );
        assert_eq!(
            PrimitiveType::Float(FloatWidth::W32).read("0.1"),
            Ok(PrimitiveValue::Float {
                width: FloatWidth::W32,
                value: f64::from(0.1f32)
            })
        );
    }

    #[test]
    fn non_finite_literals_are_rejected_by_name() {
        for literal in ["NaN", "nan", "inf", "-inf", "Infinity", "-Infinity"] {
            assert_eq!(
                code(PrimitiveType::Float(FloatWidth::W64), literal),
                SheniCode::NonFiniteFloat,
                "{literal}"
            );
        }
    }

    #[test]
    fn a_finite_literal_that_overflows_the_width_is_out_of_range() {
        assert_eq!(
            code(PrimitiveType::Float(FloatWidth::W64), "1e400"),
            SheniCode::FloatOutOfRange
        );
        // Fits an f64 comfortably, overflows an f32.
        assert_eq!(
            code(PrimitiveType::Float(FloatWidth::W32), "1e39"),
            SheniCode::FloatOutOfRange
        );
        assert!(PrimitiveType::Float(FloatWidth::W64).read("1e39").is_ok());
    }

    #[test]
    fn float_shape_rules_each_have_their_own_code() {
        let f64_t = PrimitiveType::Float(FloatWidth::W64);
        assert_eq!(code(f64_t, "+1.0"), SheniCode::LeadingPlus);
        assert_eq!(code(f64_t, "1_0.0"), SheniCode::DigitSeparator);
        assert_eq!(code(f64_t, "1.0.0"), SheniCode::NotAFloat);
        assert_eq!(code(f64_t, "one"), SheniCode::NotAFloat);
        assert_eq!(code(f64_t, ""), SheniCode::EmptyLiteral);
    }

    /// The leading-zero rule is an integer rule; `0.5` needs its zero.
    #[test]
    fn floats_keep_a_leading_zero() {
        assert!(PrimitiveType::Float(FloatWidth::W64).read("0.5").is_ok());
    }

    // ---- characters and text ----

    #[test]
    fn char_takes_exactly_one_scalar_value() {
        assert_eq!(PrimitiveType::Char.read("a"), Ok(PrimitiveValue::Char('a')));
        assert_eq!(PrimitiveType::Char.read("ß"), Ok(PrimitiveValue::Char('ß')));
        assert_eq!(
            PrimitiveType::Char.read("字"),
            Ok(PrimitiveValue::Char('字'))
        );
        assert_eq!(
            code(PrimitiveType::Char, "ab"),
            SheniCode::NotASingleCharacter
        );
        assert_eq!(code(PrimitiveType::Char, ""), SheniCode::EmptyLiteral);
    }

    #[test]
    fn byte_takes_one_character_within_latin_1() {
        assert_eq!(PrimitiveType::Byte.read("a"), Ok(PrimitiveValue::Byte(97)));
        assert_eq!(PrimitiveType::Byte.read("ÿ"), Ok(PrimitiveValue::Byte(255)));
        assert_eq!(code(PrimitiveType::Byte, "字"), SheniCode::ByteOutOfRange);
        assert_eq!(
            code(PrimitiveType::Byte, "ab"),
            SheniCode::NotASingleCharacter
        );
    }

    #[test]
    fn string_takes_any_text_verbatim_including_empty() {
        for text in ["", "  padded  ", "42", "line\nbreak", "字"] {
            assert_eq!(
                PrimitiveType::String.read(text),
                Ok(PrimitiveValue::String(text.to_string())),
                "{text:?}"
            );
        }
    }

    // ---- reading parsed SYON ----

    #[test]
    fn a_scalar_node_reads_at_its_declared_type() {
        let node = Value::Scalar("42".to_string());
        assert_eq!(
            PrimitiveType::Unsigned(IntWidth::W8).read_value(&node),
            Ok(PrimitiveValue::Unsigned {
                width: IntWidth::W8,
                value: 42
            })
        );
    }

    #[test]
    fn a_block_scalar_is_text_like_any_other_scalar() {
        let node = Value::LiteralBlock("a paragraph\n".to_string());
        assert_eq!(
            PrimitiveType::String.read_value(&node),
            Ok(PrimitiveValue::String("a paragraph\n".to_string()))
        );
    }

    #[test]
    fn a_mapping_or_sequence_is_not_a_single_value() {
        assert_eq!(
            PrimitiveType::String
                .read_value(&Value::Mapping(vec![]))
                .unwrap_err()
                .code(),
            SheniCode::NotAScalar
        );
        assert_eq!(
            PrimitiveType::String
                .read_value(&Value::Sequence(vec![]))
                .unwrap_err()
                .code(),
            SheniCode::NotAScalar
        );
    }

    // ---- values ----

    #[test]
    fn a_value_reports_the_type_it_was_read_at() {
        for t in PrimitiveType::ALL {
            let literal = match t {
                PrimitiveType::Boolean => "true",
                PrimitiveType::Float(_) => "1.5",
                PrimitiveType::Byte | PrimitiveType::Char => "a",
                PrimitiveType::String => "text",
                _ => "7",
            };
            let value = t.read(literal).unwrap_or_else(|e| panic!("{t}: {e}"));
            assert_eq!(value.type_of(), t);
        }
    }

    #[test]
    fn the_same_number_at_two_widths_is_two_values() {
        let narrow = PrimitiveType::Unsigned(IntWidth::W8).read("7").unwrap();
        let wide = PrimitiveType::Unsigned(IntWidth::W64).read("7").unwrap();
        assert_ne!(narrow, wide);
    }

    #[test]
    fn the_canonical_form_reads_back_as_an_equal_value() {
        let literals = [
            (PrimitiveType::Boolean, "yes"),
            (PrimitiveType::Unsigned(IntWidth::W64), "1234"),
            (PrimitiveType::Signed(IntWidth::W32), "-9"),
            (PrimitiveType::Float(FloatWidth::W64), "0.1"),
            (PrimitiveType::Float(FloatWidth::W32), "0.1"),
            (PrimitiveType::Byte, "a"),
            (PrimitiveType::Char, "字"),
            (PrimitiveType::String, "  padded  "),
        ];
        for (t, literal) in literals {
            let value = t.read(literal).unwrap();
            assert_eq!(t.read(&value.to_string()), Ok(value), "{t} {literal:?}");
        }
    }

    #[test]
    fn booleans_canonicalise_to_true_and_false() {
        assert_eq!(
            PrimitiveType::Boolean.read("yes").unwrap().to_string(),
            "true"
        );
        assert_eq!(
            PrimitiveType::Boolean.read("No").unwrap().to_string(),
            "false"
        );
    }

    #[test]
    fn a_type_serialises_as_its_name() {
        let json = serde_json::to_string(&PrimitiveType::Unsigned(IntWidth::W16)).unwrap();
        assert_eq!(json, "\"u16\"");
        let back: PrimitiveType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, PrimitiveType::Unsigned(IntWidth::W16));
    }

    #[test]
    fn an_unknown_type_name_fails_to_deserialise_with_its_code() {
        let err = PrimitiveType::try_from("int".to_string()).unwrap_err();
        assert_eq!(err.code(), SheniCode::UnknownTypeName);
        assert!(serde_json::from_str::<PrimitiveType>("\"int\"").is_err());
    }
}
