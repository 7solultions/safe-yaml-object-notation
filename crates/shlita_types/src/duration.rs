//! TIME and LTIME: the IEC duration literal.
//!
//! `T#5s500ms`, and everything ADR shlita_01 said sheni's `duration` refuses:
//! a sign, a fraction on the least significant unit, an underscore between
//! groups, overflow in the leading unit, and the `TIME#` long form. That list
//! is why this is a fourth duration type and not a widening of the third.
//!
//! The rules the standard fixes, and this module enforces:
//!
//! - The units run `d h m s ms` for TIME, and LTIME adds `us` and `ns`.
//! - They appear at most once each, in descending order.
//! - Only the most significant unit present may exceed its natural range, so
//!   `T#25h` is a duration and `T#1d25h` is not.
//! - Only the least significant unit present may carry a fraction.
//! - TIME resolves to a whole millisecond; LTIME to a whole nanosecond.

use std::fmt;

use crate::elementary::{ElementaryType, ElementaryValue};
use crate::error::{Result, ShlitaError};
use crate::error_code::ShlitaCode;
use crate::numeric::{strip_type_prefix, strip_underscores};

/// A unit, its length in nanoseconds, and the value it carries before the
/// next unit up takes over.
struct Unit {
    name: &'static str,
    nanos: i128,
    modulus: i128,
}

const UNITS: [Unit; 7] = [
    Unit {
        name: "d",
        nanos: 86_400_000_000_000,
        modulus: i128::MAX,
    },
    Unit {
        name: "h",
        nanos: 3_600_000_000_000,
        modulus: 24,
    },
    Unit {
        name: "m",
        nanos: 60_000_000_000,
        modulus: 60,
    },
    Unit {
        name: "s",
        nanos: 1_000_000_000,
        modulus: 60,
    },
    Unit {
        name: "ms",
        nanos: 1_000_000,
        modulus: 1_000,
    },
    Unit {
        name: "us",
        nanos: 1_000,
        modulus: 1_000,
    },
    Unit {
        name: "ns",
        nanos: 1,
        modulus: 1_000,
    },
];

/// How many units the type admits: TIME stops at milliseconds.
const fn unit_count(ty: ElementaryType) -> usize {
    if ty.is_long() {
        7
    } else {
        5
    }
}

/// The finest whole tick the type can hold, in nanoseconds.
const fn resolution(ty: ElementaryType) -> i128 {
    if ty.is_long() {
        1
    } else {
        1_000_000
    }
}

/// The largest magnitude the type can hold, in nanoseconds. TIME counts
/// milliseconds and LTIME nanoseconds, and both count them in 64 bits.
const fn limit(ty: ElementaryType) -> i128 {
    if ty.is_long() {
        i64::MAX as i128
    } else {
        i64::MAX as i128 * 1_000_000
    }
}

/// Read TIME or LTIME.
pub(crate) fn read(ty: ElementaryType, literal: &str) -> Result<ElementaryValue> {
    if !literal.contains('#') {
        return Err(ty.err(
            ShlitaCode::MalformedDuration,
            literal,
            format!(
                "a duration literal opens with its type, as in `{}#5s`",
                ty.aliases().first().copied().unwrap_or(ty.name())
            ),
        ));
    }
    let body = strip_type_prefix(ty, literal)?;
    let (negative, body) = match body.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, body.strip_prefix('+').unwrap_or(body)),
    };
    if body.is_empty() {
        return Err(ty.err(
            ShlitaCode::MalformedDuration,
            literal,
            "the literal names no units",
        ));
    }

    let bytes = body.as_bytes();
    let mut at = 0usize;
    let mut previous: Option<usize> = None;
    let mut total: i128 = 0;
    let mut fraction_seen = false;

    while at < bytes.len() {
        if fraction_seen {
            return Err(ty.err(
                ShlitaCode::MalformedDuration,
                literal,
                "only the least significant unit may carry a fraction",
            ));
        }
        let start = at;
        while at < bytes.len() && (bytes[at].is_ascii_digit() || bytes[at] == b'_') {
            at += 1;
        }
        let whole = &body[start..at];
        let mut fraction = "";
        if at < bytes.len() && bytes[at] == b'.' {
            at += 1;
            let from = at;
            while at < bytes.len() && (bytes[at].is_ascii_digit() || bytes[at] == b'_') {
                at += 1;
            }
            fraction = &body[from..at];
            fraction_seen = true;
        }
        let from = at;
        while at < bytes.len() && bytes[at].is_ascii_alphabetic() {
            at += 1;
        }
        let unit_name = &body[from..at];
        if unit_name.is_empty() {
            return Err(ty.err(
                ShlitaCode::MalformedDuration,
                literal,
                "a number here is not followed by a unit",
            ));
        }

        let index = UNITS
            .iter()
            .take(unit_count(ty))
            .position(|u| unit_name.eq_ignore_ascii_case(u.name))
            .ok_or_else(|| {
                let known: Vec<&str> = UNITS.iter().take(unit_count(ty)).map(|u| u.name).collect();
                ty.err(
                    ShlitaCode::MalformedDuration,
                    literal,
                    format!(
                        "`{unit_name}` is not a unit of {ty}; the units are {}",
                        known.join(", ")
                    ),
                )
            })?;
        if let Some(previous) = previous {
            if index <= previous {
                return Err(ty.err(
                    ShlitaCode::MalformedDuration,
                    literal,
                    "the units run from largest to smallest, each at most once",
                ));
            }
        }
        let unit = &UNITS[index];

        let whole = strip_underscores(ty, literal, whole)?;
        if whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
            return Err(ty.err(
                ShlitaCode::MalformedDuration,
                literal,
                format!("`{unit_name}` has no number in front of it"),
            ));
        }
        let whole: i128 = whole.parse().map_err(|_| out_of_range(ty, literal))?;
        if previous.is_some() && whole >= unit.modulus {
            return Err(ty.err(
                ShlitaCode::MalformedDuration,
                literal,
                format!(
                    "only the leading unit may overflow, and {whole}{} is {} or more",
                    unit.name, unit.modulus
                ),
            ));
        }

        let mut group = whole
            .checked_mul(unit.nanos)
            .ok_or_else(|| out_of_range(ty, literal))?;
        if !fraction.is_empty() {
            let fraction = strip_underscores(ty, literal, fraction)?;
            if fraction.is_empty() || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return Err(ty.err(
                    ShlitaCode::MalformedDuration,
                    literal,
                    "the fraction has no digits",
                ));
            }
            let numerator: i128 = fraction.parse().map_err(|_| out_of_range(ty, literal))?;
            let denominator = 10i128
                .checked_pow(fraction.len() as u32)
                .ok_or_else(|| out_of_range(ty, literal))?;
            let scaled = numerator
                .checked_mul(unit.nanos)
                .ok_or_else(|| out_of_range(ty, literal))?;
            if scaled % denominator != 0 {
                return Err(ty.err(
                    ShlitaCode::MalformedDuration,
                    literal,
                    format!("the fraction is finer than {ty} can hold"),
                ));
            }
            group = group
                .checked_add(scaled / denominator)
                .ok_or_else(|| out_of_range(ty, literal))?;
        }
        total = total
            .checked_add(group)
            .ok_or_else(|| out_of_range(ty, literal))?;
        previous = Some(index);
    }

    if previous.is_none() {
        return Err(ty.err(
            ShlitaCode::MalformedDuration,
            literal,
            "the literal names no units",
        ));
    }
    if total % resolution(ty) != 0 {
        return Err(ty.err(
            ShlitaCode::MalformedDuration,
            literal,
            format!(
                "{ty} resolves to {}",
                if ty.is_long() {
                    "nanoseconds"
                } else {
                    "milliseconds"
                }
            ),
        ));
    }
    if total > limit(ty) {
        return Err(out_of_range(ty, literal));
    }
    Ok(ElementaryValue::Duration {
        ty,
        nanos: if negative { -total } else { total },
    })
}

fn out_of_range(ty: ElementaryType, literal: &str) -> crate::error::ShlitaError {
    ty.err(
        ShlitaCode::DurationOutOfRange,
        literal,
        format!("the duration is larger than {ty} can hold"),
    )
}

/// Write the canonical form: the short prefix, then the sign, then every
/// non-zero unit from largest to smallest. A zero duration is `T#0s`, which
/// is the shortest text that still says which type it is.
pub(crate) fn format(ty: ElementaryType, nanos: i128, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let short = ty.aliases().first().copied().unwrap_or(ty.name());
    write!(f, "{short}#")?;
    if nanos < 0 {
        f.write_str("-")?;
    }
    let mut left = nanos.unsigned_abs() as i128;
    if left == 0 {
        return f.write_str("0s");
    }
    for unit in UNITS.iter().take(unit_count(ty)) {
        let count = left / unit.nanos;
        if count > 0 {
            write!(f, "{count}{}", unit.name)?;
            left -= count * unit.nanos;
        }
    }
    Ok(())
}

/// Build a duration from a computed nanosecond count.
///
/// This is where the arithmetic functions land, so the two rules a computed
/// duration has to obey live in one place: it is truncated to the type's
/// resolution, the way a controller's clock truncates rather than rounds,
/// and it is refused outright when it will not fit.
pub(crate) fn checked(ty: ElementaryType, nanos: i128, context: &str) -> Result<ElementaryValue> {
    let nanos = nanos - nanos % resolution(ty);
    if nanos > limit(ty) || nanos < -limit(ty) {
        return Err(ShlitaError::new(
            ShlitaCode::ArithmeticOverflow,
            context,
            nanos.to_string(),
            format!("the result is larger than {ty} can hold"),
        ));
    }
    Ok(ElementaryValue::Duration { ty, nanos })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nanos(ty: ElementaryType, literal: &str) -> i128 {
        match ty.read(literal) {
            Ok(ElementaryValue::Duration { nanos, .. }) => nanos,
            Ok(other) => panic!("{literal} read as {other:?}"),
            Err(e) => panic!("{literal}: {e}"),
        }
    }

    fn code(ty: ElementaryType, literal: &str) -> ShlitaCode {
        ty.read(literal).unwrap_err().code()
    }

    #[test]
    fn the_long_and_short_prefixes_name_the_same_type() {
        assert_eq!(nanos(ElementaryType::Time, "T#5s"), 5_000_000_000);
        assert_eq!(nanos(ElementaryType::Time, "TIME#5s"), 5_000_000_000);
        assert_eq!(nanos(ElementaryType::Time, "t#5s"), 5_000_000_000);
        assert_eq!(nanos(ElementaryType::Ltime, "LT#5s"), 5_000_000_000);
        assert_eq!(nanos(ElementaryType::Ltime, "LTIME#5s"), 5_000_000_000);
    }

    /// The four things sheni's `duration` refuses, one test each way.
    #[test]
    fn the_iec_literal_takes_what_shenis_duration_will_not() {
        assert_eq!(nanos(ElementaryType::Time, "T#-1s"), -1_000_000_000);
        assert_eq!(nanos(ElementaryType::Time, "T#1.5s"), 1_500_000_000);
        assert_eq!(nanos(ElementaryType::Time, "T#1_000ms"), 1_000_000_000);
        assert_eq!(nanos(ElementaryType::Time, "T#25h"), 90_000_000_000_000);
    }

    #[test]
    fn units_descend_and_repeat_never() {
        assert_eq!(
            nanos(ElementaryType::Time, "T#1d2h3m4s5ms"),
            86_400_000_000_000 + 7_200_000_000_000 + 180_000_000_000 + 4_000_000_000 + 5_000_000
        );
        assert_eq!(
            code(ElementaryType::Time, "T#5ms1s"),
            ShlitaCode::MalformedDuration
        );
        assert_eq!(
            code(ElementaryType::Time, "T#1s1s"),
            ShlitaCode::MalformedDuration
        );
    }

    #[test]
    fn only_the_leading_unit_may_overflow() {
        assert!(ElementaryType::Time.read("T#90m").is_ok());
        assert_eq!(
            code(ElementaryType::Time, "T#1h90m"),
            ShlitaCode::MalformedDuration
        );
    }

    #[test]
    fn only_the_last_unit_may_carry_a_fraction() {
        assert!(ElementaryType::Time.read("T#1h30.5m").is_ok());
        assert_eq!(
            code(ElementaryType::Time, "T#1.5h30m"),
            ShlitaCode::MalformedDuration
        );
    }

    /// TIME stops at the millisecond; the finer units are LTIME's, and this
    /// is the difference between the two types.
    #[test]
    fn time_resolves_to_milliseconds_and_ltime_to_nanoseconds() {
        assert_eq!(
            code(ElementaryType::Time, "T#5us"),
            ShlitaCode::MalformedDuration
        );
        assert_eq!(
            code(ElementaryType::Time, "T#0.0005s"),
            ShlitaCode::MalformedDuration
        );
        assert_eq!(nanos(ElementaryType::Ltime, "LT#5us"), 5_000);
        assert_eq!(nanos(ElementaryType::Ltime, "LT#0.0005s"), 500_000);
        assert_eq!(
            code(ElementaryType::Ltime, "LT#0.5ns"),
            ShlitaCode::MalformedDuration
        );
    }

    #[test]
    fn a_duration_needs_its_prefix_and_a_unit() {
        assert_eq!(
            code(ElementaryType::Time, "5s"),
            ShlitaCode::MalformedDuration
        );
        assert_eq!(
            code(ElementaryType::Time, "T#5"),
            ShlitaCode::MalformedDuration
        );
        assert_eq!(
            code(ElementaryType::Time, "T#"),
            ShlitaCode::MalformedDuration
        );
        assert_eq!(
            code(ElementaryType::Time, "T#s"),
            ShlitaCode::MalformedDuration
        );
        assert_eq!(
            code(ElementaryType::Ltime, "T#5s"),
            ShlitaCode::WrongTypePrefix
        );
    }

    #[test]
    fn a_duration_beyond_the_types_range_is_refused() {
        assert_eq!(
            code(ElementaryType::Ltime, "LT#9223372037s"),
            ShlitaCode::DurationOutOfRange
        );
        assert!(ElementaryType::Time.read("T#9223372036854775s").is_ok());
    }

    #[test]
    fn the_canonical_form_drops_the_zero_units() {
        let show = |ty: ElementaryType, literal: &str| ty.read(literal).unwrap().to_string();
        assert_eq!(show(ElementaryType::Time, "T#1d2h3m4s5ms"), "T#1d2h3m4s5ms");
        assert_eq!(show(ElementaryType::Time, "T#90m"), "T#1h30m");
        assert_eq!(show(ElementaryType::Time, "T#0s"), "T#0s");
        assert_eq!(show(ElementaryType::Time, "T#-1.5s"), "T#-1s500ms");
        assert_eq!(show(ElementaryType::Ltime, "LT#1s500us"), "LT#1s500us");
    }
}
