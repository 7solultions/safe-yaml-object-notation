//! Conversions: inside the IEC vocabulary, and across to sheni's primitives.
//!
//! Nothing in this crate converts on its own. A function refuses arguments
//! of different types rather than widening one of them, and this module is
//! where a caller says explicitly what it wants -- which is the standard's
//! own arrangement, where `INT_TO_DINT` is a function you call and not a
//! thing that happens.
//!
//! The bridge to sheni is the surface ADR shlita_01 accepted as the cost of
//! an own vocabulary, and it is deliberately not total. BYTE converts to a
//! `u8` and not to sheni's `byte`, because sheni's `byte` is a character.
//! TIME converts to no primitive at all, because a duration is not one.

use sheni_types::{FloatWidth, IntWidth, PrimitiveType, PrimitiveValue};

use crate::datetime;
use crate::duration;
use crate::elementary::{ElementaryClass, ElementaryType, ElementaryValue};
use crate::error::{Result, ShlitaError};
use crate::error_code::ShlitaCode;

impl ElementaryValue {
    /// Convert to another elementary type.
    ///
    /// The conversions are the standard's `*_TO_*` functions: between the
    /// numbers, between the bit strings and the integers, from a duration to
    /// its own count of ticks, and from a DATE_AND_TIME down to either half.
    /// A conversion the standard does not define is
    /// [`ShlitaCode::NotConvertible`] rather than a best effort.
    ///
    /// A real converted to an integer rounds to the nearest, ties to even,
    /// as IEC 61131-3 requires -- truncation is [`crate::functions`]' TRUNC
    /// and is a different function on purpose.
    pub fn convert_to(&self, target: ElementaryType) -> Result<ElementaryValue> {
        let source = self.type_of();
        if source == target {
            return Ok(self.clone());
        }
        let context = format!("{source}_TO_{target}");
        match (source.class(), target.class()) {
            // Everything integral converts to everything integral, and the
            // range check is what makes it safe.
            (
                ElementaryClass::Bit
                | ElementaryClass::SignedInteger
                | ElementaryClass::UnsignedInteger
                | ElementaryClass::BitString,
                ElementaryClass::Bit
                | ElementaryClass::SignedInteger
                | ElementaryClass::UnsignedInteger
                | ElementaryClass::BitString,
            ) => {
                let value = self
                    .as_bits()
                    .map(i128::from)
                    .unwrap_or_else(|| self.as_i128().expect("an integral value converts"));
                convert_integer(target, value, &context)
            }
            (
                ElementaryClass::Bit
                | ElementaryClass::SignedInteger
                | ElementaryClass::UnsignedInteger
                | ElementaryClass::BitString,
                ElementaryClass::Real,
            ) => {
                let value = self
                    .as_i128()
                    .or_else(|| self.as_bits().map(i128::from))
                    .expect("an integral value converts");
                ElementaryValue::from_f64(target, value as f64, &context)
            }
            (ElementaryClass::Real, ElementaryClass::Real) => {
                ElementaryValue::from_f64(target, self.as_f64().expect("a real"), &context)
            }
            (ElementaryClass::Real, _) if target.is_any_int() || target.is_any_bit() => {
                let rounded = round_ties_even(self.as_f64().expect("a real"));
                if !rounded.is_finite() || rounded.abs() >= 1.8e19 {
                    return Err(out_of_range(&context, target));
                }
                convert_integer(target, rounded as i128, &context)
            }
            // A duration converts to its own count of ticks, and back.
            (ElementaryClass::Duration, _) if target.is_any_int() => {
                let ElementaryValue::Duration { ty, nanos } = self else {
                    unreachable!("the class is Duration")
                };
                let ticks = nanos / tick(*ty);
                convert_integer(target, ticks, &context)
            }
            (_, ElementaryClass::Duration) if source.is_any_int() => {
                let ticks = self.as_i128().expect("an integer");
                duration::checked(target, ticks * tick(target), &context)
            }
            // A DATE_AND_TIME is a date and a time of day, and says so.
            (ElementaryClass::DateAndTime, ElementaryClass::Date) => {
                let ElementaryValue::DateAndTime { nanos, .. } = self else {
                    unreachable!("the class is DateAndTime")
                };
                datetime::checked_date(
                    target,
                    nanos.div_euclid(datetime::nanos_per_day()),
                    &context,
                )
            }
            (ElementaryClass::DateAndTime, ElementaryClass::TimeOfDay) => {
                let ElementaryValue::DateAndTime { nanos, .. } = self else {
                    unreachable!("the class is DateAndTime")
                };
                Ok(datetime::wrapped_time_of_day(target, *nanos))
            }
            (ElementaryClass::Date, ElementaryClass::DateAndTime) => {
                let ElementaryValue::Date { days, .. } = self else {
                    unreachable!("the class is Date")
                };
                datetime::checked_date_and_time(
                    target,
                    i128::from(*days) * datetime::nanos_per_day(),
                    &context,
                )
            }
            // The widths of one date class convert into each other.
            (ElementaryClass::Date, ElementaryClass::Date) => {
                let ElementaryValue::Date { days, .. } = self else {
                    unreachable!("the class is Date")
                };
                datetime::checked_date(target, i128::from(*days), &context)
            }
            (ElementaryClass::TimeOfDay, ElementaryClass::TimeOfDay) => {
                let ElementaryValue::TimeOfDay { nanos, .. } = self else {
                    unreachable!("the class is TimeOfDay")
                };
                if !target.is_long() && nanos % 1_000_000 != 0 {
                    return Err(ShlitaError::new(
                        ShlitaCode::ConversionOutOfRange,
                        context,
                        nanos.to_string(),
                        format!("{target} resolves to milliseconds"),
                    ));
                }
                Ok(datetime::wrapped_time_of_day(target, i128::from(*nanos)))
            }
            (ElementaryClass::DateAndTime, ElementaryClass::DateAndTime) => {
                let ElementaryValue::DateAndTime { nanos, .. } = self else {
                    unreachable!("the class is DateAndTime")
                };
                datetime::checked_date_and_time(target, *nanos, &context)
            }
            (ElementaryClass::Duration, ElementaryClass::Duration) => {
                let ElementaryValue::Duration { nanos, .. } = self else {
                    unreachable!("the class is Duration")
                };
                duration::checked(target, *nanos, &context)
            }
            // A character is its code point, and a string of one character.
            (ElementaryClass::Character, ElementaryClass::Character) => {
                let ElementaryValue::Char { code, .. } = self else {
                    unreachable!("the class is Character")
                };
                let limit = if target == ElementaryType::Char {
                    0xFF
                } else {
                    0xFFFF
                };
                if *code > limit {
                    return Err(out_of_range(&context, target));
                }
                Ok(ElementaryValue::Char {
                    ty: target,
                    code: *code,
                })
            }
            (ElementaryClass::Character, ElementaryClass::CharacterString) => {
                let ElementaryValue::Char { code, .. } = self else {
                    unreachable!("the class is Character")
                };
                let character =
                    char::from_u32(*code).ok_or_else(|| out_of_range(&context, target))?;
                if target == ElementaryType::String && *code > 0xFF {
                    return Err(out_of_range(&context, target));
                }
                Ok(ElementaryValue::Text {
                    ty: target,
                    value: character.to_string(),
                })
            }
            (ElementaryClass::CharacterString, ElementaryClass::CharacterString) => {
                let text = self.as_text().expect("the class is CharacterString");
                if target == ElementaryType::String && text.chars().any(|c| c as u32 > 0xFF) {
                    return Err(out_of_range(&context, target));
                }
                Ok(ElementaryValue::Text {
                    ty: target,
                    value: text.to_string(),
                })
            }
            _ => Err(ShlitaError::new(
                ShlitaCode::NotConvertible,
                context,
                self.to_string(),
                format!("no conversion is defined from {source} to {target}"),
            )),
        }
    }

    /// The same value as one of sheni's primitives.
    ///
    /// The mapping is by shape and not by name: BYTE becomes a `u8`, because
    /// sheni's `byte` is a character and would be a lie. The types with no
    /// primitive counterpart -- the durations and the three date types --
    /// are [`ShlitaCode::NotConvertible`], which is the honest answer rather
    /// than a millisecond count wearing the wrong name.
    pub fn to_sheni(&self) -> Result<PrimitiveValue> {
        let ty = self.type_of();
        let context = format!("{ty}_TO_SHENI");
        let width = |bits: u32| match bits {
            8 => IntWidth::W8,
            16 => IntWidth::W16,
            32 => IntWidth::W32,
            _ => IntWidth::W64,
        };
        match self {
            ElementaryValue::Bool(value) => Ok(PrimitiveValue::Boolean(*value)),
            ElementaryValue::Signed { ty, value } => Ok(PrimitiveValue::Signed {
                width: width(ty.bit_width().expect("an integer has a width")),
                value: i128::from(*value),
            }),
            ElementaryValue::Unsigned { ty, value } | ElementaryValue::Bits { ty, value } => {
                Ok(PrimitiveValue::Unsigned {
                    width: width(ty.bit_width().expect("an integer has a width")),
                    value: u128::from(*value),
                })
            }
            ElementaryValue::Real { ty, value } => Ok(PrimitiveValue::Float {
                width: if *ty == ElementaryType::Real {
                    FloatWidth::W32
                } else {
                    FloatWidth::W64
                },
                value: *value,
            }),
            ElementaryValue::Char { ty, code } => {
                let character = char::from_u32(*code).ok_or_else(|| out_of_range(&context, *ty))?;
                if *ty == ElementaryType::Char {
                    Ok(PrimitiveValue::Byte(*code as u8))
                } else {
                    Ok(PrimitiveValue::Char(character))
                }
            }
            ElementaryValue::Text { value, .. } => Ok(PrimitiveValue::String(value.clone())),
            _ => Err(ShlitaError::new(
                ShlitaCode::NotConvertible,
                context,
                self.to_string(),
                format!("sheni has no primitive that means {ty}"),
            )),
        }
    }

    /// Read one of sheni's primitives at an elementary type.
    ///
    /// The direction that can fail on range: a sheni `u32` holding 70000 is
    /// not a UINT, and says so rather than truncating.
    pub fn from_sheni(value: &PrimitiveValue, ty: ElementaryType) -> Result<ElementaryValue> {
        let context = format!("SHENI_TO_{ty}");
        match value {
            PrimitiveValue::Boolean(value) => {
                if ty == ElementaryType::Bool {
                    Ok(ElementaryValue::Bool(*value))
                } else {
                    convert_integer(ty, i128::from(*value), &context)
                }
            }
            PrimitiveValue::Unsigned { value, .. } => {
                let value = i128::try_from(*value).map_err(|_| out_of_range(&context, ty))?;
                convert_integer(ty, value, &context)
            }
            PrimitiveValue::Signed { value, .. } => convert_integer(ty, *value, &context),
            PrimitiveValue::Float { value, .. } => {
                if ty.is_any_real() {
                    ElementaryValue::from_f64(ty, *value, &context)
                } else {
                    Err(not_convertible(
                        &context,
                        PrimitiveType::Float(FloatWidth::W64),
                        ty,
                    ))
                }
            }
            PrimitiveValue::Byte(byte) => character(ty, u32::from(*byte), &context),
            PrimitiveValue::Char(character_value) => {
                character(ty, *character_value as u32, &context)
            }
            PrimitiveValue::String(text) => {
                if !ty.is_any_string() {
                    return Err(not_convertible(&context, PrimitiveType::String, ty));
                }
                if ty == ElementaryType::String && text.chars().any(|c| c as u32 > 0xFF) {
                    return Err(out_of_range(&context, ty));
                }
                Ok(ElementaryValue::Text {
                    ty,
                    value: text.clone(),
                })
            }
        }
    }
}

/// The nanoseconds in one tick of a duration type: a millisecond for TIME,
/// a nanosecond for LTIME.
const fn tick(ty: ElementaryType) -> i128 {
    if ty.is_long() {
        1
    } else {
        1_000_000
    }
}

/// Land an integer in a target type, or say it does not fit.
fn convert_integer(target: ElementaryType, value: i128, context: &str) -> Result<ElementaryValue> {
    ElementaryValue::from_i128(target, value, context).map_err(|e| {
        if e.code() == ShlitaCode::ArithmeticOverflow {
            ShlitaError::new(
                ShlitaCode::ConversionOutOfRange,
                context,
                value.to_string(),
                e.message(),
            )
        } else {
            e
        }
    })
}

fn character(ty: ElementaryType, code: u32, context: &str) -> Result<ElementaryValue> {
    if ty.is_any_char() {
        let limit = if ty == ElementaryType::Char {
            0xFF
        } else {
            0xFFFF
        };
        if code > limit {
            return Err(out_of_range(context, ty));
        }
        return Ok(ElementaryValue::Char { ty, code });
    }
    if ty.is_any_string() {
        let character = char::from_u32(code).ok_or_else(|| out_of_range(context, ty))?;
        if ty == ElementaryType::String && code > 0xFF {
            return Err(out_of_range(context, ty));
        }
        return Ok(ElementaryValue::Text {
            ty,
            value: character.to_string(),
        });
    }
    convert_integer(ty, i128::from(code), context)
}

fn out_of_range(context: &str, ty: ElementaryType) -> ShlitaError {
    ShlitaError::at(
        ShlitaCode::ConversionOutOfRange,
        context,
        format!("the value does not fit {ty}"),
    )
}

fn not_convertible(context: &str, from: PrimitiveType, to: ElementaryType) -> ShlitaError {
    ShlitaError::at(
        ShlitaCode::NotConvertible,
        context,
        format!(
            "no conversion is defined from sheni's {} to {to}",
            from.name()
        ),
    )
}

/// Round half to even, which is what IEC 61131-3 requires of REAL_TO_INT.
fn round_ties_even(value: f64) -> f64 {
    let rounded = value.round();
    if (value - value.trunc()).abs() == 0.5 && rounded % 2.0 != 0.0 {
        rounded - value.signum()
    } else {
        rounded
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(ty: ElementaryType, literal: &str) -> ElementaryValue {
        ty.read(literal)
            .unwrap_or_else(|e| panic!("{literal}: {e}"))
    }

    #[test]
    fn the_integral_types_convert_into_each_other_with_a_range_check() {
        let byte = read(ElementaryType::Byte, "16#FF");
        assert_eq!(
            byte.convert_to(ElementaryType::Uint).unwrap().to_string(),
            "255"
        );
        assert_eq!(
            byte.convert_to(ElementaryType::Sint).unwrap_err().code(),
            ShlitaCode::ConversionOutOfRange
        );
        assert_eq!(
            read(ElementaryType::Int, "-1")
                .convert_to(ElementaryType::Uint)
                .unwrap_err()
                .code(),
            ShlitaCode::ConversionOutOfRange
        );
    }

    /// The standard rounds to the nearest and breaks a tie to the even
    /// number; TRUNC is the other function, and it is not this one.
    #[test]
    fn a_real_converted_to_an_integer_rounds_ties_to_even() {
        let show = |literal: &str| {
            read(ElementaryType::Lreal, literal)
                .convert_to(ElementaryType::Dint)
                .unwrap()
                .to_string()
        };
        assert_eq!(show("0.5"), "0");
        assert_eq!(show("1.5"), "2");
        assert_eq!(show("2.5"), "2");
        assert_eq!(show("-1.5"), "-2");
        assert_eq!(show("1.4"), "1");
    }

    #[test]
    fn a_duration_converts_to_its_own_ticks_and_back() {
        let span = read(ElementaryType::Time, "T#1s500ms");
        assert_eq!(
            span.convert_to(ElementaryType::Dint).unwrap().to_string(),
            "1500"
        );
        assert_eq!(
            read(ElementaryType::Dint, "1500")
                .convert_to(ElementaryType::Time)
                .unwrap()
                .to_string(),
            "T#1s500ms"
        );
        // LTIME counts nanoseconds, so three seconds of it is already more
        // than a DINT can hold -- the tick is part of the conversion.
        assert_eq!(
            read(ElementaryType::Ltime, "LT#3s")
                .convert_to(ElementaryType::Dint)
                .unwrap_err()
                .code(),
            ShlitaCode::ConversionOutOfRange
        );
        assert_eq!(
            read(ElementaryType::Ltime, "LT#1s")
                .convert_to(ElementaryType::Dint)
                .unwrap()
                .to_string(),
            "1000000000"
        );
    }

    #[test]
    fn a_date_and_time_splits_into_its_two_halves() {
        let stamp = read(ElementaryType::DateAndTime, "DT#2026-08-29-12:30:00");
        assert_eq!(
            stamp.convert_to(ElementaryType::Date).unwrap().to_string(),
            "D#2026-08-29"
        );
        assert_eq!(
            stamp
                .convert_to(ElementaryType::TimeOfDay)
                .unwrap()
                .to_string(),
            "TOD#12:30:00"
        );
    }

    #[test]
    fn a_conversion_the_standard_does_not_define_is_refused() {
        assert_eq!(
            read(ElementaryType::Time, "T#1s")
                .convert_to(ElementaryType::Date)
                .unwrap_err()
                .code(),
            ShlitaCode::NotConvertible
        );
        assert_eq!(
            read(ElementaryType::String, "'1'")
                .convert_to(ElementaryType::Int)
                .unwrap_err()
                .code(),
            ShlitaCode::NotConvertible
        );
    }

    /// The bridge ADR shlita_01 accepted as the cost of an own vocabulary.
    #[test]
    fn the_bridge_to_sheni_maps_by_shape_and_not_by_name() {
        let byte = read(ElementaryType::Byte, "16#41");
        assert_eq!(
            byte.to_sheni().unwrap(),
            PrimitiveValue::Unsigned {
                width: IntWidth::W8,
                value: 65
            }
        );
        // Not sheni's `byte`, which is the character `A` -- the collision
        // the ADR named.
        assert_ne!(byte.to_sheni().unwrap(), PrimitiveValue::Byte(65));

        assert_eq!(
            read(ElementaryType::Char, "CHAR#'A'").to_sheni().unwrap(),
            PrimitiveValue::Byte(65)
        );
        assert_eq!(
            read(ElementaryType::Bool, "TRUE").to_sheni().unwrap(),
            PrimitiveValue::Boolean(true)
        );
    }

    #[test]
    fn a_duration_has_no_sheni_primitive_to_be() {
        assert_eq!(
            read(ElementaryType::Time, "T#1s")
                .to_sheni()
                .unwrap_err()
                .code(),
            ShlitaCode::NotConvertible
        );
        assert_eq!(
            read(ElementaryType::Date, "D#2026-08-29")
                .to_sheni()
                .unwrap_err()
                .code(),
            ShlitaCode::NotConvertible
        );
    }

    #[test]
    fn a_sheni_primitive_reads_at_an_elementary_type_or_says_why_not() {
        let big = PrimitiveValue::Unsigned {
            width: IntWidth::W32,
            value: 70_000,
        };
        assert_eq!(
            ElementaryValue::from_sheni(&big, ElementaryType::Udint)
                .unwrap()
                .to_string(),
            "70000"
        );
        assert_eq!(
            ElementaryValue::from_sheni(&big, ElementaryType::Uint)
                .unwrap_err()
                .code(),
            ShlitaCode::ConversionOutOfRange
        );
        assert_eq!(
            ElementaryValue::from_sheni(&PrimitiveValue::String("hi".into()), ElementaryType::Int)
                .unwrap_err()
                .code(),
            ShlitaCode::NotConvertible
        );
    }
}
