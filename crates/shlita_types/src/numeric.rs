//! Reading the numeric literals: BOOL, the integers, the bit strings and the
//! reals.
//!
//! Four things make an IEC numeric literal different from a Rust one, and
//! each is a rule here:
//!
//! - A literal may carry its type: `INT#7`, `BYTE#16#FF`. The prefix has to
//!   agree with the type being read, or the document says one thing and means
//!   another.
//! - A literal may be based: `2#1010_1010`, `8#777`, `16#DEAD_BEEF`, and the
//!   bases are exactly 2, 8 and 16.
//! - Underscores separate digit groups, and are allowed only between digits.
//! - A real literal needs a decimal point with digits on both sides, or an
//!   exponent. `1.` and `.5` are not real literals.
//!
//! Anything outside the accepted forms is an error rather than a coercion.

use crate::elementary::{ElementaryClass, ElementaryType, ElementaryValue};
use crate::error::Result;
use crate::error_code::ShlitaCode;

/// Strip an optional `TYPE#` prefix, checking that it names the type being
/// read.
///
/// A base is not a prefix: in `16#FF` the text before the `#` is digits, and
/// the whole literal comes back untouched for the based reader to handle.
pub(crate) fn strip_type_prefix(ty: ElementaryType, literal: &str) -> Result<&str> {
    let Some(hash) = literal.find('#') else {
        return Ok(literal);
    };
    let head = &literal[..hash];
    let unsigned_head = head.strip_prefix(['+', '-']).unwrap_or(head);
    if !unsigned_head.is_empty() && unsigned_head.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(literal);
    }
    match ElementaryType::from_name(head) {
        Some(named) if named == ty => Ok(&literal[hash + 1..]),
        Some(named) => Err(ty.err(
            ShlitaCode::WrongTypePrefix,
            literal,
            format!("the literal is prefixed {named}, and is being read at {ty}"),
        )),
        None => Err(ty.err(
            ShlitaCode::UnknownTypeName,
            literal,
            format!("`{head}` is not the name of an elementary type"),
        )),
    }
}

/// Split a leading sign from the rest.
fn split_sign(text: &str) -> (bool, &str) {
    match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    }
}

/// Remove the group separators from one run of digits, rejecting an
/// underscore that is not between two digits.
pub(crate) fn strip_underscores(ty: ElementaryType, literal: &str, digits: &str) -> Result<String> {
    if digits.starts_with('_') || digits.ends_with('_') || digits.contains("__") {
        return Err(ty.err(
            ShlitaCode::MisplacedUnderscore,
            literal,
            "an underscore separates two digits; it cannot lead, trail or double",
        ));
    }
    Ok(digits.replace('_', ""))
}

/// Read BOOL.
///
/// The four spellings are the standard's: the two words and the two bits.
/// `yes` and `no` are sheni's, and are not accepted here -- a controller
/// program written by a PLC engineer says `TRUE`.
pub(crate) fn read_bool(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = strip_type_prefix(ty, literal)?;
    if body.eq_ignore_ascii_case("TRUE") || body == "1" {
        return Ok(ElementaryValue::Bool(true));
    }
    if body.eq_ignore_ascii_case("FALSE") || body == "0" {
        return Ok(ElementaryValue::Bool(false));
    }
    Err(ty.err(
        ShlitaCode::NotABoolean,
        literal,
        "expected TRUE, FALSE, 1 or 0",
    ))
}

/// Read a signed integer, an unsigned integer or a bit string.
pub(crate) fn read_integer(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = strip_type_prefix(ty, literal)?;
    let (negative, rest) = split_sign(body);
    let (base, digits) = match rest.find('#') {
        Some(at) => {
            if negative {
                return Err(ty.err(
                    ShlitaCode::MalformedBase,
                    literal,
                    "a based literal carries no sign; the standard signs decimal literals only",
                ));
            }
            let base = match &rest[..at] {
                "2" => 2u32,
                "8" => 8,
                "16" => 16,
                other => {
                    return Err(ty.err(
                        ShlitaCode::MalformedBase,
                        literal,
                        format!("the standard's bases are 2, 8 and 16, not {other}"),
                    ))
                }
            };
            (base, &rest[at + 1..])
        }
        None => (10, rest),
    };
    let digits = strip_underscores(ty, literal, digits)?;
    if digits.is_empty() {
        return Err(ty.err(ShlitaCode::NotAnInteger, literal, "expected digits"));
    }
    if !digits.chars().all(|c| c.is_digit(base)) {
        return Err(if base == 10 {
            ty.err(
                ShlitaCode::NotAnInteger,
                literal,
                "expected decimal digits, an optional sign and nothing else",
            )
        } else {
            ty.err(
                ShlitaCode::MalformedBase,
                literal,
                format!("a digit falls outside base {base}"),
            )
        });
    }
    let magnitude = u128::from_str_radix(&digits, base)
        .map_err(|_| range_error(ty, literal, "the literal is too large for any integer type"))?;

    match ty.class() {
        ElementaryClass::SignedInteger => {
            let (low, high) = ty.signed_range().expect("a signed integer has a range");
            let signed = if negative {
                i128::try_from(magnitude).map(|m| -m)
            } else {
                i128::try_from(magnitude)
            }
            .map_err(|_| {
                range_error(ty, literal, format!("the range of {ty} is {low}..={high}"))
            })?;
            if signed < i128::from(low) || signed > i128::from(high) {
                return Err(range_error(
                    ty,
                    literal,
                    format!("the range of {ty} is {low}..={high}"),
                ));
            }
            Ok(ElementaryValue::Signed {
                ty,
                value: signed as i64,
            })
        }
        ElementaryClass::UnsignedInteger | ElementaryClass::BitString => {
            let max = ty.unsigned_max().expect("an unsigned type has a maximum");
            if negative && magnitude != 0 {
                return Err(range_error(
                    ty,
                    literal,
                    format!("{ty} has no negative values"),
                ));
            }
            if magnitude > u128::from(max) {
                return Err(range_error(
                    ty,
                    literal,
                    format!("the range of {ty} is 0..={max}"),
                ));
            }
            let value = magnitude as u64;
            Ok(if ty.class() == ElementaryClass::BitString {
                ElementaryValue::Bits { ty, value }
            } else {
                ElementaryValue::Unsigned { ty, value }
            })
        }
        _ => unreachable!("read_integer is only reached for integer and bit string types"),
    }
}

fn range_error(
    ty: ElementaryType,
    literal: &str,
    message: impl Into<String>,
) -> crate::error::ShlitaError {
    ty.err(ShlitaCode::IntegerOutOfRange, literal, message)
}

/// Read REAL or LREAL.
pub(crate) fn read_real(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    let body = strip_type_prefix(ty, literal)?;
    let (negative, rest) = split_sign(body);
    if rest.eq_ignore_ascii_case("inf")
        || rest.eq_ignore_ascii_case("infinity")
        || rest.eq_ignore_ascii_case("nan")
    {
        return Err(ty.err(
            ShlitaCode::NonFiniteReal,
            literal,
            "the standard gives no literal form for an infinity or a NaN",
        ));
    }

    let (mantissa, exponent) = match rest.find(['e', 'E']) {
        Some(at) => (&rest[..at], Some(&rest[at + 1..])),
        None => (rest, None),
    };
    let (whole, fraction) = match mantissa.find('.') {
        Some(at) => (&mantissa[..at], Some(&mantissa[at + 1..])),
        None => (mantissa, None),
    };
    if fraction.is_none() && exponent.is_none() {
        return Err(ty.err(
            ShlitaCode::NotAReal,
            literal,
            "a real literal has a decimal point or an exponent",
        ));
    }

    let whole = digits_only(
        ty,
        literal,
        whole,
        "a digit is missing before the decimal point",
    )?;
    let fraction = match fraction {
        Some(text) => Some(digits_only(
            ty,
            literal,
            text,
            "a digit is missing after the decimal point",
        )?),
        None => None,
    };
    let exponent = match exponent {
        Some(text) => {
            let (exp_negative, exp_digits) = split_sign(text);
            let exp_digits = digits_only(ty, literal, exp_digits, "the exponent has no digits")?;
            Some(format!(
                "{}{}",
                if exp_negative { "-" } else { "" },
                exp_digits
            ))
        }
        None => None,
    };

    let mut clean = String::new();
    if negative {
        clean.push('-');
    }
    clean.push_str(&whole);
    if let Some(fraction) = &fraction {
        clean.push('.');
        clean.push_str(fraction);
    }
    if let Some(exponent) = &exponent {
        clean.push('e');
        clean.push_str(exponent);
    }

    let value: f64 = clean
        .parse()
        .map_err(|_| ty.err(ShlitaCode::NotAReal, literal, "not a real literal"))?;
    if !value.is_finite() {
        return Err(ty.err(
            ShlitaCode::RealOutOfRange,
            literal,
            format!("the literal overflows the range of {ty}"),
        ));
    }
    ElementaryValue::from_f64(ty, value, ty.name()).map_err(|e| {
        // `from_f64` reports against a computation; here the subject is the
        // literal the document actually contains.
        ty.err(e.code(), literal, e.message())
    })
}

/// One run of digits, with the separators removed.
fn digits_only(ty: ElementaryType, literal: &str, digits: &str, missing: &str) -> Result<String> {
    let digits = strip_underscores(ty, literal, digits)?;
    if digits.is_empty() {
        return Err(ty.err(ShlitaCode::NotAReal, literal, missing));
    }
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ty.err(ShlitaCode::NotAReal, literal, "not a real literal"));
    }
    Ok(digits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
        ty.read(literal)
    }

    fn code(ty: ElementaryType, literal: &str) -> ShlitaCode {
        read(ty, literal).unwrap_err().code()
    }

    #[test]
    fn bool_takes_the_two_words_and_the_two_bits() {
        for literal in ["TRUE", "true", "1", "BOOL#TRUE", "BOOL#1"] {
            assert_eq!(
                read(ElementaryType::Bool, literal),
                Ok(ElementaryValue::Bool(true))
            );
        }
        for literal in ["FALSE", "false", "0", "BOOL#0"] {
            assert_eq!(
                read(ElementaryType::Bool, literal),
                Ok(ElementaryValue::Bool(false))
            );
        }
    }

    /// Sheni's booleans admit `yes` and `no`; the standard's do not, and a
    /// crate that accepted both would be inventing a third language.
    #[test]
    fn bool_does_not_take_shenis_spellings() {
        assert_eq!(code(ElementaryType::Bool, "yes"), ShlitaCode::NotABoolean);
        assert_eq!(code(ElementaryType::Bool, "no"), ShlitaCode::NotABoolean);
    }

    #[test]
    fn integers_read_in_every_base_the_standard_defines() {
        let expect = |literal: &str, value: i64| {
            assert_eq!(
                read(ElementaryType::Int, literal),
                Ok(ElementaryValue::Signed {
                    ty: ElementaryType::Int,
                    value
                }),
                "{literal}"
            );
        };
        expect("0", 0);
        expect("-7", -7);
        expect("+7", 7);
        expect("2#1010_1010", 170);
        expect("8#777", 511);
        expect("16#7FFF", 32767);
        expect("16#7fff", 32767);
        expect("INT#42", 42);
        expect("INT#16#2A", 42);
    }

    #[test]
    fn a_literal_that_does_not_fit_says_so_rather_than_wrapping() {
        assert_eq!(
            code(ElementaryType::Sint, "128"),
            ShlitaCode::IntegerOutOfRange
        );
        assert_eq!(
            read(ElementaryType::Sint, "127").unwrap().to_string(),
            "127"
        );
        assert_eq!(
            code(ElementaryType::Sint, "-129"),
            ShlitaCode::IntegerOutOfRange
        );
        assert_eq!(
            code(ElementaryType::Usint, "-1"),
            ShlitaCode::IntegerOutOfRange
        );
        assert_eq!(
            code(ElementaryType::Byte, "256"),
            ShlitaCode::IntegerOutOfRange
        );
        assert_eq!(
            code(ElementaryType::Ulint, "18446744073709551616"),
            ShlitaCode::IntegerOutOfRange
        );
    }

    #[test]
    fn a_prefix_naming_another_type_is_refused() {
        assert_eq!(
            code(ElementaryType::Dint, "INT#7"),
            ShlitaCode::WrongTypePrefix
        );
        assert_eq!(
            code(ElementaryType::Dint, "FOO#7"),
            ShlitaCode::UnknownTypeName
        );
    }

    #[test]
    fn the_bases_are_two_eight_and_sixteen_and_the_digits_have_to_fit_them() {
        assert_eq!(code(ElementaryType::Int, "3#12"), ShlitaCode::MalformedBase);
        assert_eq!(
            code(ElementaryType::Int, "2#1012"),
            ShlitaCode::MalformedBase
        );
        assert_eq!(
            code(ElementaryType::Int, "-16#FF"),
            ShlitaCode::MalformedBase
        );
    }

    #[test]
    fn underscores_separate_digits_and_do_nothing_else() {
        assert!(read(ElementaryType::Dint, "1_000_000").is_ok());
        assert_eq!(
            code(ElementaryType::Dint, "_1000"),
            ShlitaCode::MisplacedUnderscore
        );
        assert_eq!(
            code(ElementaryType::Dint, "1000_"),
            ShlitaCode::MisplacedUnderscore
        );
        assert_eq!(
            code(ElementaryType::Dint, "1__000"),
            ShlitaCode::MisplacedUnderscore
        );
        assert_eq!(
            code(ElementaryType::Dint, "16#_FF"),
            ShlitaCode::MisplacedUnderscore
        );
    }

    #[test]
    fn reals_need_a_point_between_digits_or_an_exponent() {
        assert!(read(ElementaryType::Lreal, "1.0").is_ok());
        assert!(read(ElementaryType::Lreal, "-1.34e-12").is_ok());
        assert!(read(ElementaryType::Lreal, "1e3").is_ok());
        assert!(read(ElementaryType::Lreal, "REAL#1.0").is_err());
        assert_eq!(code(ElementaryType::Lreal, "1"), ShlitaCode::NotAReal);
        assert_eq!(code(ElementaryType::Lreal, "1."), ShlitaCode::NotAReal);
        assert_eq!(code(ElementaryType::Lreal, ".5"), ShlitaCode::NotAReal);
        assert_eq!(code(ElementaryType::Lreal, "1.0e"), ShlitaCode::NotAReal);
    }

    #[test]
    fn an_infinity_is_named_rather_than_dismissed_as_malformed() {
        assert_eq!(code(ElementaryType::Real, "INF"), ShlitaCode::NonFiniteReal);
        assert_eq!(
            code(ElementaryType::Lreal, "nan"),
            ShlitaCode::NonFiniteReal
        );
        assert_eq!(
            code(ElementaryType::Real, "1e39"),
            ShlitaCode::RealOutOfRange
        );
        assert!(read(ElementaryType::Lreal, "1e39").is_ok());
    }

    /// A REAL is single precision, and a value read at REAL is the value a
    /// controller would hold -- not the double the text happens to name.
    #[test]
    fn a_real_is_narrowed_to_single_precision_when_it_is_read() {
        let value = read(ElementaryType::Real, "0.1").unwrap();
        assert_eq!(
            value,
            ElementaryValue::Real {
                ty: ElementaryType::Real,
                value: f64::from(0.1f32)
            }
        );
        assert_ne!(value.as_f64(), Some(0.1f64));
    }
}
