//! The standard functions: the half of IEC 61131-3 that has no state.
//!
//! Every function here is a function of its arguments alone. Call it twice
//! with the same inputs and it answers the same thing, because there is
//! nothing between the calls for it to remember. That is the line ADR
//! shlita_01 drew: the timers, the counters, the bistables and the edge
//! detectors retain state between scans and belong to `shlita_runtime`, and
//! the split is made here rather than discovered later.
//!
//! Two rules of the standard shape the whole module:
//!
//! - **Arguments agree in type.** `ADD(INT#1, DINT#2)` is not a call the
//!   standard defines. The conversions are explicit and live in
//!   [`crate::convert`], so nothing here widens an argument quietly.
//! - **The bit strings are not the unsigned integers.** `AND` takes ANY_BIT,
//!   which is BOOL, BYTE, WORD, DWORD and LWORD. Handing it a UINT is an
//!   error, and it is the error the type vocabulary exists to make possible.
//!
//! Results that do not fit are reported rather than wrapped, on the same
//! reasoning that makes an out-of-range literal an error.

use std::cmp::Ordering;

use crate::datetime;
use crate::duration;
use crate::elementary::{ElementaryClass, ElementaryType, ElementaryValue};
use crate::error::{Result, ShlitaError};
use crate::error_code::ShlitaCode;

/// The longest string a function will build.
///
/// The standard leaves the maximum length to the implementation and every
/// vendor picks one; 254 is the most common. Reading a literal is not
/// limited -- a document says what it says -- but a computed string that
/// runs past this is [`ShlitaCode::StringTooLong`] rather than a value no
/// controller could hold.
pub const MAX_STRING_LENGTH: usize = 254;

/// A standard function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StandardFunction {
    // Bitwise, over ANY_BIT.
    And,
    Or,
    Xor,
    Not,
    // Bit shifts.
    Shl,
    Shr,
    Rol,
    Ror,
    // Selection.
    Sel,
    Max,
    Min,
    Limit,
    Mux,
    // Comparison.
    Gt,
    Ge,
    Eq,
    Le,
    Lt,
    Ne,
    // Arithmetic.
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Expt,
    Move,
    // Numeric.
    Abs,
    Sqrt,
    Ln,
    Log,
    Exp,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Trunc,
    // Character strings.
    Len,
    Left,
    Right,
    Mid,
    Concat,
    Insert,
    Delete,
    Replace,
    Find,
}

impl StandardFunction {
    /// Every standard function this crate implements.
    pub const ALL: [StandardFunction; 47] = [
        StandardFunction::And,
        StandardFunction::Or,
        StandardFunction::Xor,
        StandardFunction::Not,
        StandardFunction::Shl,
        StandardFunction::Shr,
        StandardFunction::Rol,
        StandardFunction::Ror,
        StandardFunction::Sel,
        StandardFunction::Max,
        StandardFunction::Min,
        StandardFunction::Limit,
        StandardFunction::Mux,
        StandardFunction::Gt,
        StandardFunction::Ge,
        StandardFunction::Eq,
        StandardFunction::Le,
        StandardFunction::Lt,
        StandardFunction::Ne,
        StandardFunction::Add,
        StandardFunction::Sub,
        StandardFunction::Mul,
        StandardFunction::Div,
        StandardFunction::Mod,
        StandardFunction::Expt,
        StandardFunction::Move,
        StandardFunction::Abs,
        StandardFunction::Sqrt,
        StandardFunction::Ln,
        StandardFunction::Log,
        StandardFunction::Exp,
        StandardFunction::Sin,
        StandardFunction::Cos,
        StandardFunction::Tan,
        StandardFunction::Asin,
        StandardFunction::Acos,
        StandardFunction::Atan,
        StandardFunction::Trunc,
        StandardFunction::Len,
        StandardFunction::Left,
        StandardFunction::Right,
        StandardFunction::Mid,
        StandardFunction::Concat,
        StandardFunction::Insert,
        StandardFunction::Delete,
        StandardFunction::Replace,
        StandardFunction::Find,
    ];

    /// The function's name, as a document writes it.
    pub const fn name(self) -> &'static str {
        match self {
            StandardFunction::And => "AND",
            StandardFunction::Or => "OR",
            StandardFunction::Xor => "XOR",
            StandardFunction::Not => "NOT",
            StandardFunction::Shl => "SHL",
            StandardFunction::Shr => "SHR",
            StandardFunction::Rol => "ROL",
            StandardFunction::Ror => "ROR",
            StandardFunction::Sel => "SEL",
            StandardFunction::Max => "MAX",
            StandardFunction::Min => "MIN",
            StandardFunction::Limit => "LIMIT",
            StandardFunction::Mux => "MUX",
            StandardFunction::Gt => "GT",
            StandardFunction::Ge => "GE",
            StandardFunction::Eq => "EQ",
            StandardFunction::Le => "LE",
            StandardFunction::Lt => "LT",
            StandardFunction::Ne => "NE",
            StandardFunction::Add => "ADD",
            StandardFunction::Sub => "SUB",
            StandardFunction::Mul => "MUL",
            StandardFunction::Div => "DIV",
            StandardFunction::Mod => "MOD",
            StandardFunction::Expt => "EXPT",
            StandardFunction::Move => "MOVE",
            StandardFunction::Abs => "ABS",
            StandardFunction::Sqrt => "SQRT",
            StandardFunction::Ln => "LN",
            StandardFunction::Log => "LOG",
            StandardFunction::Exp => "EXP",
            StandardFunction::Sin => "SIN",
            StandardFunction::Cos => "COS",
            StandardFunction::Tan => "TAN",
            StandardFunction::Asin => "ASIN",
            StandardFunction::Acos => "ACOS",
            StandardFunction::Atan => "ATAN",
            StandardFunction::Trunc => "TRUNC",
            StandardFunction::Len => "LEN",
            StandardFunction::Left => "LEFT",
            StandardFunction::Right => "RIGHT",
            StandardFunction::Mid => "MID",
            StandardFunction::Concat => "CONCAT",
            StandardFunction::Insert => "INSERT",
            StandardFunction::Delete => "DELETE",
            StandardFunction::Replace => "REPLACE",
            StandardFunction::Find => "FIND",
        }
    }

    /// The function of that name. Names are case-insensitive, as IEC
    /// keywords are.
    pub fn from_name(name: &str) -> Option<Self> {
        StandardFunction::ALL
            .into_iter()
            .find(|f| name.eq_ignore_ascii_case(f.name()))
    }

    /// How many arguments the function takes: an exact count, or a minimum
    /// for the extensible ones.
    pub const fn arity(self) -> Arity {
        match self {
            StandardFunction::And
            | StandardFunction::Or
            | StandardFunction::Xor
            | StandardFunction::Add
            | StandardFunction::Mul
            | StandardFunction::Max
            | StandardFunction::Min
            | StandardFunction::Concat
            | StandardFunction::Gt
            | StandardFunction::Ge
            | StandardFunction::Eq
            | StandardFunction::Le
            | StandardFunction::Lt
            | StandardFunction::Ne
            | StandardFunction::Mux => Arity::AtLeast(2),
            StandardFunction::Not
            | StandardFunction::Move
            | StandardFunction::Abs
            | StandardFunction::Sqrt
            | StandardFunction::Ln
            | StandardFunction::Log
            | StandardFunction::Exp
            | StandardFunction::Sin
            | StandardFunction::Cos
            | StandardFunction::Tan
            | StandardFunction::Asin
            | StandardFunction::Acos
            | StandardFunction::Atan
            | StandardFunction::Trunc
            | StandardFunction::Len => Arity::Exactly(1),
            StandardFunction::Shl
            | StandardFunction::Shr
            | StandardFunction::Rol
            | StandardFunction::Ror
            | StandardFunction::Sub
            | StandardFunction::Div
            | StandardFunction::Mod
            | StandardFunction::Expt
            | StandardFunction::Left
            | StandardFunction::Right
            | StandardFunction::Find => Arity::Exactly(2),
            StandardFunction::Sel
            | StandardFunction::Limit
            | StandardFunction::Mid
            | StandardFunction::Insert
            | StandardFunction::Delete => Arity::Exactly(3),
            StandardFunction::Replace => Arity::Exactly(4),
        }
    }

    /// Apply the function.
    pub fn call(self, args: &[ElementaryValue]) -> Result<ElementaryValue> {
        let name = self.name();
        match self.arity() {
            Arity::Exactly(n) if args.len() != n => {
                return Err(wrong_count(name, format!("{n}"), args.len()))
            }
            Arity::AtLeast(n) if args.len() < n => {
                return Err(wrong_count(name, format!("at least {n}"), args.len()))
            }
            _ => {}
        }
        match self {
            StandardFunction::And | StandardFunction::Or | StandardFunction::Xor => {
                bitwise(self, args)
            }
            StandardFunction::Not => not(args),
            StandardFunction::Shl
            | StandardFunction::Shr
            | StandardFunction::Rol
            | StandardFunction::Ror => shift(self, args),
            StandardFunction::Sel => sel(args),
            StandardFunction::Mux => mux(args),
            StandardFunction::Max | StandardFunction::Min => extreme(self, args),
            StandardFunction::Limit => limit(args),
            StandardFunction::Gt
            | StandardFunction::Ge
            | StandardFunction::Eq
            | StandardFunction::Le
            | StandardFunction::Lt
            | StandardFunction::Ne => comparison(self, args),
            StandardFunction::Add => add(args),
            StandardFunction::Sub => sub(args),
            StandardFunction::Mul => mul(args),
            StandardFunction::Div => div(args),
            StandardFunction::Mod => modulus(args),
            StandardFunction::Expt => expt(args),
            StandardFunction::Move => Ok(args[0].clone()),
            StandardFunction::Abs => abs(args),
            StandardFunction::Trunc => trunc(args),
            StandardFunction::Sqrt
            | StandardFunction::Ln
            | StandardFunction::Log
            | StandardFunction::Exp
            | StandardFunction::Sin
            | StandardFunction::Cos
            | StandardFunction::Tan
            | StandardFunction::Asin
            | StandardFunction::Acos
            | StandardFunction::Atan => transcendental(self, args),
            StandardFunction::Len
            | StandardFunction::Left
            | StandardFunction::Right
            | StandardFunction::Mid
            | StandardFunction::Concat
            | StandardFunction::Insert
            | StandardFunction::Delete
            | StandardFunction::Replace
            | StandardFunction::Find => string(self, args),
        }
    }
}

/// How many arguments a function takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arity {
    Exactly(usize),
    /// The extensible functions: ADD and AND take any number of arguments
    /// from two upwards.
    AtLeast(usize),
}

/// Call a standard function by name.
pub fn call(name: &str, args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let function = StandardFunction::from_name(name).ok_or_else(|| {
        ShlitaError::new(
            ShlitaCode::UnknownFunction,
            name,
            "",
            "no standard function goes by that name",
        )
    })?;
    function.call(args)
}

// ---- shared checks ----------------------------------------------------

fn err(
    name: &str,
    code: ShlitaCode,
    subject: impl Into<String>,
    message: impl Into<String>,
) -> ShlitaError {
    ShlitaError::new(code, name, subject, message)
}

fn wrong_count(name: &str, expected: String, given: usize) -> ShlitaError {
    err(
        name,
        ShlitaCode::WrongArgumentCount,
        "",
        format!("{name} takes {expected} arguments, and was given {given}"),
    )
}

/// The type every argument shares, or the mismatch that says they do not.
fn one_type(name: &str, args: &[ElementaryValue]) -> Result<ElementaryType> {
    let first = args[0].type_of();
    for other in &args[1..] {
        if other.type_of() != first {
            return Err(err(
                name,
                ShlitaCode::TypeMismatch,
                other.to_string(),
                format!(
                    "{name} takes arguments of one type, and was given {first} and {}",
                    other.type_of()
                ),
            ));
        }
    }
    Ok(first)
}

fn require(name: &str, value: &ElementaryValue, code: ShlitaCode, what: &str) -> Result<()> {
    let ok = match code {
        ShlitaCode::NotABitString => value.type_of().is_any_bit(),
        ShlitaCode::NotANumber => value.type_of().is_any_num(),
        _ => true,
    };
    if ok {
        Ok(())
    } else {
        Err(err(
            name,
            code,
            value.to_string(),
            format!("{name} takes {what}, and {} is not one", value.type_of()),
        ))
    }
}

/// The mask that keeps a bit string inside its width.
fn mask(ty: ElementaryType) -> u64 {
    match ty.bit_width() {
        Some(64) => u64::MAX,
        Some(width) => (1u64 << width) - 1,
        None => 0,
    }
}

/// Order two values of the same type.
fn compare(name: &str, left: &ElementaryValue, right: &ElementaryValue) -> Result<Ordering> {
    one_type(name, std::slice::from_ref(left))?;
    if left.type_of() != right.type_of() {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            right.to_string(),
            format!(
                "{name} compares values of one type, and was given {} and {}",
                left.type_of(),
                right.type_of()
            ),
        ));
    }
    let ordering = match (left, right) {
        (ElementaryValue::Bool(a), ElementaryValue::Bool(b)) => a.cmp(b),
        (ElementaryValue::Signed { value: a, .. }, ElementaryValue::Signed { value: b, .. }) => {
            a.cmp(b)
        }
        (
            ElementaryValue::Unsigned { value: a, .. },
            ElementaryValue::Unsigned { value: b, .. },
        )
        | (ElementaryValue::Bits { value: a, .. }, ElementaryValue::Bits { value: b, .. }) => {
            a.cmp(b)
        }
        (ElementaryValue::Real { value: a, .. }, ElementaryValue::Real { value: b, .. }) => a
            .partial_cmp(b)
            .expect("a real value is finite, so it orders"),
        (
            ElementaryValue::Duration { nanos: a, .. },
            ElementaryValue::Duration { nanos: b, .. },
        )
        | (
            ElementaryValue::DateAndTime { nanos: a, .. },
            ElementaryValue::DateAndTime { nanos: b, .. },
        ) => a.cmp(b),
        (ElementaryValue::Date { days: a, .. }, ElementaryValue::Date { days: b, .. }) => a.cmp(b),
        (
            ElementaryValue::TimeOfDay { nanos: a, .. },
            ElementaryValue::TimeOfDay { nanos: b, .. },
        ) => a.cmp(b),
        (ElementaryValue::Char { code: a, .. }, ElementaryValue::Char { code: b, .. }) => a.cmp(b),
        (ElementaryValue::Text { value: a, .. }, ElementaryValue::Text { value: b, .. }) => {
            a.cmp(b)
        }
        _ => unreachable!("the two values share a type"),
    };
    Ok(ordering)
}

// ---- bitwise ----------------------------------------------------------

fn bitwise(function: StandardFunction, args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = function.name();
    let ty = one_type(name, args)?;
    for arg in args {
        require(name, arg, ShlitaCode::NotABitString, "bit strings")?;
    }
    let mut bits = args[0].as_bits().expect("checked to be a bit string");
    for arg in &args[1..] {
        let next = arg.as_bits().expect("checked to be a bit string");
        bits = match function {
            StandardFunction::And => bits & next,
            StandardFunction::Or => bits | next,
            _ => bits ^ next,
        };
    }
    Ok(bits_value(ty, bits))
}

fn not(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Not.name();
    require(name, &args[0], ShlitaCode::NotABitString, "a bit string")?;
    let ty = args[0].type_of();
    let bits = args[0].as_bits().expect("checked to be a bit string");
    Ok(bits_value(ty, !bits & mask(ty)))
}

fn bits_value(ty: ElementaryType, bits: u64) -> ElementaryValue {
    if ty == ElementaryType::Bool {
        ElementaryValue::Bool(bits & 1 == 1)
    } else {
        ElementaryValue::Bits {
            ty,
            value: bits & mask(ty),
        }
    }
}

fn shift(function: StandardFunction, args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = function.name();
    require(name, &args[0], ShlitaCode::NotABitString, "a bit string")?;
    let ty = args[0].type_of();
    let bits = args[0].as_bits().expect("checked to be a bit string");
    if !args[1].type_of().is_any_int() {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            args[1].to_string(),
            format!("{name} shifts by an integer, not by {}", args[1].type_of()),
        ));
    }
    let places = args[1].as_i128().expect("an integer converts");
    if places < 0 {
        return Err(err(
            name,
            ShlitaCode::DomainError,
            places.to_string(),
            format!("{name} shifts by a count that is not negative"),
        ));
    }
    let width = u128::from(ty.bit_width().unwrap_or(1));
    let places = places as u128;
    let shifted = match function {
        StandardFunction::Shl | StandardFunction::Shr if places >= width => 0,
        StandardFunction::Shl => bits << places,
        StandardFunction::Shr => (bits & mask(ty)) >> places,
        _ => {
            // A rotation by a full width is the identity, so only the
            // remainder does any work.
            let places = (places % width) as u32;
            let bits = bits & mask(ty);
            if places == 0 {
                bits
            } else {
                let width = width as u32;
                match function {
                    StandardFunction::Rol => (bits << places) | (bits >> (width - places)),
                    _ => (bits >> places) | (bits << (width - places)),
                }
            }
        }
    };
    Ok(bits_value(ty, shifted))
}

// ---- selection --------------------------------------------------------

fn sel(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Sel.name();
    let ElementaryValue::Bool(gate) = args[0] else {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            args[0].to_string(),
            "SEL selects on a BOOL",
        ));
    };
    one_type(name, &args[1..])?;
    Ok(args[if gate { 2 } else { 1 }].clone())
}

fn mux(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Mux.name();
    if !args[0].type_of().is_any_int() {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            args[0].to_string(),
            "MUX selects on an integer",
        ));
    }
    one_type(name, &args[1..])?;
    let selector = args[0].as_i128().expect("an integer converts");
    let inputs = &args[1..];
    if selector < 0 || selector as usize >= inputs.len() {
        return Err(err(
            name,
            ShlitaCode::SelectorOutOfRange,
            selector.to_string(),
            format!("MUX was given {} inputs to select from", inputs.len()),
        ));
    }
    Ok(inputs[selector as usize].clone())
}

fn extreme(function: StandardFunction, args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = function.name();
    one_type(name, args)?;
    let mut best = &args[0];
    for candidate in &args[1..] {
        let ordering = compare(name, candidate, best)?;
        let takes = match function {
            StandardFunction::Max => ordering == Ordering::Greater,
            _ => ordering == Ordering::Less,
        };
        if takes {
            best = candidate;
        }
    }
    Ok(best.clone())
}

fn limit(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Limit.name();
    one_type(name, args)?;
    let (low, value, high) = (&args[0], &args[1], &args[2]);
    if compare(name, value, low)? == Ordering::Less {
        return Ok(low.clone());
    }
    if compare(name, value, high)? == Ordering::Greater {
        return Ok(high.clone());
    }
    Ok(value.clone())
}

fn comparison(function: StandardFunction, args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = function.name();
    // The comparisons are extensible, and `GT(a, b, c)` means `a > b > c`
    // rather than `(a > b) > c` -- the standard chains them.
    for pair in args.windows(2) {
        let ordering = compare(name, &pair[0], &pair[1])?;
        let holds = match function {
            StandardFunction::Gt => ordering == Ordering::Greater,
            StandardFunction::Ge => ordering != Ordering::Less,
            StandardFunction::Eq => ordering == Ordering::Equal,
            StandardFunction::Le => ordering != Ordering::Greater,
            StandardFunction::Lt => ordering == Ordering::Less,
            _ => ordering != Ordering::Equal,
        };
        if !holds {
            return Ok(ElementaryValue::Bool(false));
        }
    }
    Ok(ElementaryValue::Bool(true))
}

// ---- arithmetic -------------------------------------------------------

fn add(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Add.name();
    // The mixed forms the standard names separately -- ADD_TOD_TIME and
    // ADD_DT_TIME -- reached through the one overloaded ADD.
    if args.len() == 2 {
        if let ElementaryValue::Duration { nanos: span, .. } = args[1] {
            match args[0] {
                ElementaryValue::TimeOfDay { ty, nanos } => {
                    return Ok(datetime::wrapped_time_of_day(ty, i128::from(nanos) + span))
                }
                ElementaryValue::DateAndTime { ty, nanos } => {
                    return datetime::checked_date_and_time(ty, nanos + span, name)
                }
                _ => {}
            }
        }
    }
    let ty = one_type(name, args)?;
    match ty.class() {
        ElementaryClass::Duration => {
            let mut total: i128 = 0;
            for arg in args {
                let ElementaryValue::Duration { nanos, .. } = arg else {
                    unreachable!("one type, and it is a duration")
                };
                total = total
                    .checked_add(*nanos)
                    .ok_or_else(|| overflow(name, ty))?;
            }
            duration::checked(ty, total, name)
        }
        ElementaryClass::Real => {
            let mut total = 0.0;
            for arg in args {
                total += arg.as_f64().expect("one type, and it is a real");
            }
            ElementaryValue::from_f64(ty, total, name)
        }
        _ => {
            require(name, &args[0], ShlitaCode::NotANumber, "numbers")?;
            let mut total: i128 = 0;
            for arg in args {
                total = total
                    .checked_add(arg.as_i128().expect("one type, and it is an integer"))
                    .ok_or_else(|| overflow(name, ty))?;
            }
            ElementaryValue::from_i128(ty, total, name)
        }
    }
}

fn sub(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Sub.name();
    // The date differences, which give a span rather than a point.
    match (&args[0], &args[1]) {
        (
            ElementaryValue::Date { ty, days: left },
            ElementaryValue::Date {
                ty: right_ty,
                days: right,
            },
        ) if ty == right_ty => {
            let nanos = (i128::from(*left) - i128::from(*right)) * datetime::nanos_per_day();
            return duration::checked(span_type(*ty), nanos, name);
        }
        (
            ElementaryValue::DateAndTime { ty, nanos: left },
            ElementaryValue::DateAndTime {
                ty: right_ty,
                nanos: right,
            },
        ) if ty == right_ty => return duration::checked(span_type(*ty), left - right, name),
        (
            ElementaryValue::TimeOfDay { ty, nanos: left },
            ElementaryValue::TimeOfDay {
                ty: right_ty,
                nanos: right,
            },
        ) if ty == right_ty => {
            return duration::checked(span_type(*ty), i128::from(*left) - i128::from(*right), name)
        }
        (
            ElementaryValue::TimeOfDay { ty, nanos },
            ElementaryValue::Duration { nanos: span, .. },
        ) => {
            return Ok(datetime::wrapped_time_of_day(
                *ty,
                i128::from(*nanos) - span,
            ))
        }
        (
            ElementaryValue::DateAndTime { ty, nanos },
            ElementaryValue::Duration { nanos: span, .. },
        ) => return datetime::checked_date_and_time(*ty, nanos - span, name),
        _ => {}
    }
    let ty = one_type(name, args)?;
    match ty.class() {
        ElementaryClass::Duration => {
            let (
                ElementaryValue::Duration { nanos: left, .. },
                ElementaryValue::Duration { nanos: right, .. },
            ) = (&args[0], &args[1])
            else {
                unreachable!("one type, and it is a duration")
            };
            duration::checked(ty, left - right, name)
        }
        ElementaryClass::Real => ElementaryValue::from_f64(
            ty,
            args[0].as_f64().expect("a real") - args[1].as_f64().expect("a real"),
            name,
        ),
        _ => {
            require(name, &args[0], ShlitaCode::NotANumber, "numbers")?;
            let left = args[0].as_i128().expect("an integer");
            let right = args[1].as_i128().expect("an integer");
            ElementaryValue::from_i128(ty, left - right, name)
        }
    }
}

/// The span type that goes with a point type: DATE and LDATE differ by a
/// TIME and an LTIME respectively.
fn span_type(ty: ElementaryType) -> ElementaryType {
    if ty.is_long() {
        ElementaryType::Ltime
    } else {
        ElementaryType::Time
    }
}

fn mul(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Mul.name();
    // MULTIME: a span times a number is a span, and the standard defines it
    // in that order only.
    if args.len() == 2 {
        if let ElementaryValue::Duration { ty, nanos } = args[0] {
            if args[1].type_of().is_any_num() {
                let factor = args[1].as_f64().expect("a number converts");
                let scaled = nanos as f64 * factor;
                if !scaled.is_finite() {
                    return Err(overflow(name, ty));
                }
                return duration::checked(ty, scaled as i128, name);
            }
        }
    }
    let ty = one_type(name, args)?;
    if ty.is_any_real() {
        let mut total = 1.0;
        for arg in args {
            total *= arg.as_f64().expect("a real");
        }
        return ElementaryValue::from_f64(ty, total, name);
    }
    require(name, &args[0], ShlitaCode::NotANumber, "numbers")?;
    let mut total: i128 = 1;
    for arg in args {
        total = total
            .checked_mul(arg.as_i128().expect("an integer"))
            .ok_or_else(|| overflow(name, ty))?;
    }
    ElementaryValue::from_i128(ty, total, name)
}

fn div(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Div.name();
    if let ElementaryValue::Duration { ty, nanos } = args[0] {
        if args[1].type_of().is_any_num() {
            let divisor = args[1].as_f64().expect("a number converts");
            if divisor == 0.0 {
                return Err(division_by_zero(name));
            }
            return duration::checked(ty, (nanos as f64 / divisor) as i128, name);
        }
    }
    let ty = one_type(name, args)?;
    if ty.is_any_real() {
        let divisor = args[1].as_f64().expect("a real");
        if divisor == 0.0 {
            return Err(division_by_zero(name));
        }
        return ElementaryValue::from_f64(ty, args[0].as_f64().expect("a real") / divisor, name);
    }
    require(name, &args[0], ShlitaCode::NotANumber, "numbers")?;
    let divisor = args[1].as_i128().expect("an integer");
    if divisor == 0 {
        return Err(division_by_zero(name));
    }
    // Integer division truncates toward zero, which is what the standard
    // says and what the hardware does.
    ElementaryValue::from_i128(ty, args[0].as_i128().expect("an integer") / divisor, name)
}

fn modulus(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Mod.name();
    let ty = one_type(name, args)?;
    if !ty.is_any_int() {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            args[0].to_string(),
            "MOD is defined over the integers",
        ));
    }
    let divisor = args[1].as_i128().expect("an integer");
    if divisor == 0 {
        return Err(division_by_zero(name));
    }
    // The remainder takes the sign of the dividend, as truncating division
    // requires.
    ElementaryValue::from_i128(ty, args[0].as_i128().expect("an integer") % divisor, name)
}

fn expt(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Expt.name();
    let ty = args[0].type_of();
    if !ty.is_any_real() {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            args[0].to_string(),
            "EXPT raises a real to a power",
        ));
    }
    if !args[1].type_of().is_any_num() {
        return Err(err(
            name,
            ShlitaCode::NotANumber,
            args[1].to_string(),
            "the exponent is a number",
        ));
    }
    let base = args[0].as_f64().expect("a real");
    let exponent = args[1].as_f64().expect("a number");
    let value = base.powf(exponent);
    if value.is_nan() {
        return Err(err(
            name,
            ShlitaCode::DomainError,
            format!("{base}^{exponent}"),
            "a negative base raised to a fractional power is not a real number",
        ));
    }
    ElementaryValue::from_f64(ty, value, name)
}

fn abs(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Abs.name();
    let ty = args[0].type_of();
    match &args[0] {
        ElementaryValue::Real { value, .. } => ElementaryValue::from_f64(ty, value.abs(), name),
        ElementaryValue::Signed { value, .. } => {
            ElementaryValue::from_i128(ty, i128::from(*value).abs(), name)
        }
        ElementaryValue::Unsigned { .. } => Ok(args[0].clone()),
        ElementaryValue::Duration { ty, nanos } => duration::checked(*ty, nanos.abs(), name),
        other => Err(err(
            name,
            ShlitaCode::NotANumber,
            other.to_string(),
            format!("ABS takes a number, and {} is not one", other.type_of()),
        )),
    }
}

fn trunc(args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = StandardFunction::Trunc.name();
    let ElementaryValue::Real { value, .. } = args[0] else {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            args[0].to_string(),
            "TRUNC takes a real",
        ));
    };
    ElementaryValue::from_i128(ElementaryType::Dint, value.trunc() as i128, name)
}

fn transcendental(function: StandardFunction, args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = function.name();
    let ty = args[0].type_of();
    if !ty.is_any_real() {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            args[0].to_string(),
            format!("{name} is defined over REAL and LREAL"),
        ));
    }
    let value = args[0].as_f64().expect("a real");
    let domain = |ok: bool| {
        if ok {
            Ok(())
        } else {
            Err(err(
                name,
                ShlitaCode::DomainError,
                value.to_string(),
                format!("{value} is outside the domain of {name}"),
            ))
        }
    };
    let result = match function {
        StandardFunction::Sqrt => {
            domain(value >= 0.0)?;
            value.sqrt()
        }
        StandardFunction::Ln => {
            domain(value > 0.0)?;
            value.ln()
        }
        StandardFunction::Log => {
            domain(value > 0.0)?;
            value.log10()
        }
        StandardFunction::Exp => value.exp(),
        StandardFunction::Sin => value.sin(),
        StandardFunction::Cos => value.cos(),
        StandardFunction::Tan => value.tan(),
        StandardFunction::Asin => {
            domain((-1.0..=1.0).contains(&value))?;
            value.asin()
        }
        StandardFunction::Acos => {
            domain((-1.0..=1.0).contains(&value))?;
            value.acos()
        }
        _ => value.atan(),
    };
    ElementaryValue::from_f64(ty, result, name)
}

fn overflow(name: &str, ty: ElementaryType) -> ShlitaError {
    err(
        name,
        ShlitaCode::ArithmeticOverflow,
        "",
        format!("the result does not fit {ty}"),
    )
}

fn division_by_zero(name: &str) -> ShlitaError {
    err(name, ShlitaCode::DivisionByZero, "0", "the divisor is zero")
}

// ---- character strings ------------------------------------------------

/// A string argument as a vector of characters.
///
/// The index a string function takes counts characters, and for WSTRING
/// every character is one UTF-16 code unit because the reader refuses any
/// that is not.
fn characters(name: &str, value: &ElementaryValue) -> Result<Vec<char>> {
    match value {
        ElementaryValue::Text { value, .. } => Ok(value.chars().collect()),
        other => Err(err(
            name,
            ShlitaCode::TypeMismatch,
            other.to_string(),
            format!(
                "{name} takes a character string, and {} is not one",
                other.type_of()
            ),
        )),
    }
}

fn count(name: &str, value: &ElementaryValue) -> Result<i128> {
    if !value.type_of().is_any_int() {
        return Err(err(
            name,
            ShlitaCode::TypeMismatch,
            value.to_string(),
            format!("{name} takes a count as an integer"),
        ));
    }
    Ok(value.as_i128().expect("an integer converts"))
}

fn built(name: &str, ty: ElementaryType, characters: Vec<char>) -> Result<ElementaryValue> {
    if characters.len() > MAX_STRING_LENGTH {
        return Err(err(
            name,
            ShlitaCode::StringTooLong,
            characters.len().to_string(),
            format!("a computed string stops at {MAX_STRING_LENGTH} characters"),
        ));
    }
    Ok(ElementaryValue::Text {
        ty,
        value: characters.into_iter().collect(),
    })
}

fn index_error(name: &str, subject: impl Into<String>, message: impl Into<String>) -> ShlitaError {
    err(name, ShlitaCode::IndexOutOfRange, subject, message)
}

fn string(function: StandardFunction, args: &[ElementaryValue]) -> Result<ElementaryValue> {
    let name = function.name();
    let ty = args[0].type_of();
    let subject = characters(name, &args[0])?;
    let length = subject.len() as i128;
    match function {
        StandardFunction::Len => ElementaryValue::from_i128(ElementaryType::Dint, length, name),
        StandardFunction::Left | StandardFunction::Right => {
            let take = count(name, &args[1])?;
            if take < 0 || take > length {
                return Err(index_error(
                    name,
                    take.to_string(),
                    format!("the string is {length} characters long"),
                ));
            }
            let take = take as usize;
            let taken = if function == StandardFunction::Left {
                subject[..take].to_vec()
            } else {
                subject[subject.len() - take..].to_vec()
            };
            built(name, ty, taken)
        }
        StandardFunction::Mid => {
            let take = count(name, &args[1])?;
            let from = count(name, &args[2])?;
            if take < 0 || from < 1 || from - 1 + take > length {
                return Err(index_error(
                    name,
                    format!("{take} from {from}"),
                    format!("the string is {length} characters long, and positions count from 1"),
                ));
            }
            let from = (from - 1) as usize;
            built(name, ty, subject[from..from + take as usize].to_vec())
        }
        StandardFunction::Concat => {
            one_type(name, args)?;
            let mut joined = Vec::new();
            for arg in args {
                joined.extend(characters(name, arg)?);
            }
            built(name, ty, joined)
        }
        StandardFunction::Insert => {
            let inserted = characters(name, &args[1])?;
            if args[1].type_of() != ty {
                return Err(err(
                    name,
                    ShlitaCode::TypeMismatch,
                    args[1].to_string(),
                    format!("{name} joins two strings of one type"),
                ));
            }
            let after = count(name, &args[2])?;
            if after < 0 || after > length {
                return Err(index_error(
                    name,
                    after.to_string(),
                    format!("the string is {length} characters long"),
                ));
            }
            let after = after as usize;
            let mut built_up = subject[..after].to_vec();
            built_up.extend(inserted);
            built_up.extend_from_slice(&subject[after..]);
            built(name, ty, built_up)
        }
        StandardFunction::Delete => {
            let take = count(name, &args[1])?;
            let from = count(name, &args[2])?;
            if take < 0 || from < 1 || from - 1 + take > length {
                return Err(index_error(
                    name,
                    format!("{take} from {from}"),
                    format!("the string is {length} characters long, and positions count from 1"),
                ));
            }
            let from = (from - 1) as usize;
            let mut kept = subject[..from].to_vec();
            kept.extend_from_slice(&subject[from + take as usize..]);
            built(name, ty, kept)
        }
        StandardFunction::Replace => {
            if args[1].type_of() != ty {
                return Err(err(
                    name,
                    ShlitaCode::TypeMismatch,
                    args[1].to_string(),
                    format!("{name} joins two strings of one type"),
                ));
            }
            let replacement = characters(name, &args[1])?;
            let take = count(name, &args[2])?;
            let from = count(name, &args[3])?;
            if take < 0 || from < 1 || from - 1 + take > length {
                return Err(index_error(
                    name,
                    format!("{take} from {from}"),
                    format!("the string is {length} characters long, and positions count from 1"),
                ));
            }
            let from = (from - 1) as usize;
            let mut built_up = subject[..from].to_vec();
            built_up.extend(replacement);
            built_up.extend_from_slice(&subject[from + take as usize..]);
            built(name, ty, built_up)
        }
        _ => {
            // FIND, which answers with a position and not with a string.
            if args[1].type_of() != ty {
                return Err(err(
                    name,
                    ShlitaCode::TypeMismatch,
                    args[1].to_string(),
                    format!("{name} searches a string for a string of the same type"),
                ));
            }
            let needle = characters(name, &args[1])?;
            let found = if needle.is_empty() {
                // The standard's FIND answers 0 when there is nothing to
                // find, and an empty needle is nothing to find.
                0
            } else {
                subject
                    .windows(needle.len())
                    .position(|window| window == needle)
                    .map(|at| at as i128 + 1)
                    .unwrap_or(0)
            };
            ElementaryValue::from_i128(ElementaryType::Dint, found, name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(ty: ElementaryType, literal: &str) -> ElementaryValue {
        ty.read(literal)
            .unwrap_or_else(|e| panic!("{literal}: {e}"))
    }

    fn apply(name: &str, args: &[ElementaryValue]) -> Result<ElementaryValue> {
        call(name, args)
    }

    fn text_of(name: &str, args: &[ElementaryValue]) -> String {
        apply(name, args).unwrap().to_string()
    }

    fn code(name: &str, args: &[ElementaryValue]) -> ShlitaCode {
        apply(name, args).unwrap_err().code()
    }

    #[test]
    fn every_function_has_a_unique_name_that_resolves() {
        for function in StandardFunction::ALL {
            assert_eq!(StandardFunction::from_name(function.name()), Some(function));
            assert_eq!(
                StandardFunction::from_name(&function.name().to_lowercase()),
                Some(function)
            );
        }
        assert_eq!(StandardFunction::from_name("NOPE"), None);
        assert_eq!(
            call("NOPE", &[]).unwrap_err().code(),
            ShlitaCode::UnknownFunction
        );
    }

    /// The reason ADR shlita_01 refused to map the bit strings onto sheni's
    /// unsigned integers: this check would not exist.
    #[test]
    fn the_bitwise_functions_take_bit_strings_and_refuse_integers() {
        let a = read(ElementaryType::Byte, "2#1100");
        let b = read(ElementaryType::Byte, "2#1010");
        assert_eq!(text_of("AND", &[a.clone(), b.clone()]), "16#08");
        assert_eq!(text_of("OR", &[a.clone(), b.clone()]), "16#0E");
        assert_eq!(text_of("XOR", &[a.clone(), b]), "16#06");
        assert_eq!(text_of("NOT", &[a]), "16#F3");

        let unsigned = read(ElementaryType::Usint, "12");
        assert_eq!(
            code("AND", &[unsigned.clone(), unsigned]),
            ShlitaCode::NotABitString
        );
    }

    #[test]
    fn bool_is_the_narrowest_bit_string() {
        let t = ElementaryValue::Bool(true);
        let f = ElementaryValue::Bool(false);
        assert_eq!(apply("AND", &[t.clone(), f.clone()]), Ok(f.clone()));
        assert_eq!(apply("OR", &[t.clone(), f]), Ok(t.clone()));
        assert_eq!(apply("NOT", &[t]), Ok(ElementaryValue::Bool(false)));
    }

    #[test]
    fn shifts_drop_bits_and_rotations_keep_them() {
        let byte = read(ElementaryType::Byte, "2#1000_0001");
        let one = read(ElementaryType::Usint, "1");
        assert_eq!(text_of("SHL", &[byte.clone(), one.clone()]), "16#02");
        assert_eq!(text_of("SHR", &[byte.clone(), one.clone()]), "16#40");
        assert_eq!(text_of("ROL", &[byte.clone(), one.clone()]), "16#03");
        assert_eq!(text_of("ROR", &[byte.clone(), one]), "16#C0");
        let nine = read(ElementaryType::Usint, "9");
        assert_eq!(text_of("SHL", &[byte.clone(), nine.clone()]), "16#00");
        assert_eq!(text_of("ROL", &[byte, nine]), "16#03");
    }

    #[test]
    fn arguments_have_to_agree_in_type() {
        let int = read(ElementaryType::Int, "1");
        let dint = read(ElementaryType::Dint, "1");
        assert_eq!(code("ADD", &[int.clone(), dint]), ShlitaCode::TypeMismatch);
        assert_eq!(code("ADD", &[int]), ShlitaCode::WrongArgumentCount);
    }

    #[test]
    fn a_result_that_does_not_fit_is_reported_rather_than_wrapped() {
        let big = read(ElementaryType::Sint, "127");
        let one = read(ElementaryType::Sint, "1");
        assert_eq!(
            code("ADD", &[big.clone(), one.clone()]),
            ShlitaCode::ArithmeticOverflow
        );
        assert_eq!(apply("SUB", &[big, one]).unwrap().to_string(), "126");
    }

    #[test]
    fn integer_division_truncates_and_zero_is_an_error() {
        let seven = read(ElementaryType::Int, "7");
        let two = read(ElementaryType::Int, "2");
        let minus_seven = read(ElementaryType::Int, "-7");
        let zero = read(ElementaryType::Int, "0");
        assert_eq!(text_of("DIV", &[seven.clone(), two.clone()]), "3");
        assert_eq!(text_of("DIV", &[minus_seven.clone(), two.clone()]), "-3");
        assert_eq!(text_of("MOD", &[minus_seven, two]), "-1");
        assert_eq!(
            code("DIV", &[seven.clone(), zero.clone()]),
            ShlitaCode::DivisionByZero
        );
        assert_eq!(code("MOD", &[seven, zero]), ShlitaCode::DivisionByZero);
    }

    /// TIME is in ANY_MAGNITUDE, so it adds to itself and scales by a
    /// number, and it is not in ANY_NUM, so it does neither with a date.
    #[test]
    fn durations_add_to_each_other_and_scale_by_a_number() {
        let span = read(ElementaryType::Time, "T#1s");
        let three = read(ElementaryType::Int, "3");
        assert_eq!(text_of("ADD", &[span.clone(), span.clone()]), "T#2s");
        assert_eq!(text_of("MUL", &[span.clone(), three.clone()]), "T#3s");
        assert_eq!(text_of("DIV", &[span.clone(), three]), "T#333ms");
        assert_eq!(
            text_of("ABS", &[read(ElementaryType::Time, "T#-1s")]),
            "T#1s"
        );
    }

    #[test]
    fn a_span_added_to_a_point_gives_a_point_and_two_points_give_a_span() {
        let noon = read(ElementaryType::TimeOfDay, "TOD#12:00:00");
        let hour = read(ElementaryType::Time, "T#1h");
        assert_eq!(
            text_of("ADD", &[noon.clone(), hour.clone()]),
            "TOD#13:00:00"
        );
        assert_eq!(text_of("SUB", &[noon.clone(), hour]), "TOD#11:00:00");
        assert_eq!(
            text_of(
                "SUB",
                &[noon, read(ElementaryType::TimeOfDay, "TOD#11:30:00")]
            ),
            "T#30m"
        );
        let today = read(ElementaryType::Date, "D#2026-08-29");
        let yesterday = read(ElementaryType::Date, "D#2026-08-28");
        assert_eq!(text_of("SUB", &[today, yesterday]), "T#1d");
    }

    /// A clock wraps at midnight; a date and time does not, because the day
    /// is part of it.
    #[test]
    fn a_time_of_day_wraps_and_a_date_and_time_carries() {
        let late = read(ElementaryType::TimeOfDay, "TOD#23:30:00");
        let hour = read(ElementaryType::Time, "T#1h");
        assert_eq!(text_of("ADD", &[late, hour.clone()]), "TOD#00:30:00");
        let stamp = read(ElementaryType::DateAndTime, "DT#2026-08-29-23:30:00");
        assert_eq!(text_of("ADD", &[stamp, hour]), "DT#2026-08-30-00:30:00");
    }

    #[test]
    fn the_selection_functions_choose_without_computing() {
        let a = read(ElementaryType::Int, "10");
        let b = read(ElementaryType::Int, "20");
        assert_eq!(
            apply("SEL", &[ElementaryValue::Bool(false), a.clone(), b.clone()]),
            Ok(a.clone())
        );
        assert_eq!(
            apply("SEL", &[ElementaryValue::Bool(true), a.clone(), b.clone()]),
            Ok(b.clone())
        );
        assert_eq!(apply("MAX", &[a.clone(), b.clone()]), Ok(b.clone()));
        assert_eq!(apply("MIN", &[a.clone(), b.clone()]), Ok(a.clone()));
        assert_eq!(
            apply(
                "LIMIT",
                &[a.clone(), read(ElementaryType::Int, "99"), b.clone()]
            ),
            Ok(b.clone())
        );
        assert_eq!(
            apply("MUX", &[read(ElementaryType::Int, "1"), a, b.clone()]),
            Ok(b)
        );
        assert_eq!(
            code(
                "MUX",
                &[
                    read(ElementaryType::Int, "5"),
                    read(ElementaryType::Int, "1"),
                    read(ElementaryType::Int, "2")
                ]
            ),
            ShlitaCode::SelectorOutOfRange
        );
    }

    /// The comparisons are extensible and chain, which is the standard's
    /// reading and not the C one.
    #[test]
    fn comparisons_chain_and_answer_a_bool() {
        let one = read(ElementaryType::Int, "1");
        let two = read(ElementaryType::Int, "2");
        let three = read(ElementaryType::Int, "3");
        assert_eq!(
            apply("LT", &[one.clone(), two.clone(), three.clone()]),
            Ok(ElementaryValue::Bool(true))
        );
        assert_eq!(
            apply("LT", &[one.clone(), three.clone(), two.clone()]),
            Ok(ElementaryValue::Bool(false))
        );
        assert_eq!(
            apply("EQ", &[one.clone(), one]),
            Ok(ElementaryValue::Bool(true))
        );
        assert_eq!(apply("NE", &[two, three]), Ok(ElementaryValue::Bool(true)));
    }

    #[test]
    fn the_numeric_functions_report_their_domain_rather_than_answering_nan() {
        let minus_one = read(ElementaryType::Lreal, "-1.0");
        let four = read(ElementaryType::Lreal, "4.0");
        assert_eq!(text_of("SQRT", std::slice::from_ref(&four)), "2.0");
        assert_eq!(
            code("SQRT", std::slice::from_ref(&minus_one)),
            ShlitaCode::DomainError
        );
        assert_eq!(
            code("LN", &[read(ElementaryType::Lreal, "0.0")]),
            ShlitaCode::DomainError
        );
        assert_eq!(code("ASIN", &[four]), ShlitaCode::DomainError);
        assert_eq!(text_of("ABS", &[minus_one]), "1.0");
        assert_eq!(
            text_of("TRUNC", &[read(ElementaryType::Lreal, "-1.9")]),
            "-1"
        );
    }

    #[test]
    fn the_string_functions_count_characters_from_one() {
        let hello = read(ElementaryType::String, "'hello'");
        let world = read(ElementaryType::String, "'world'");
        let two = read(ElementaryType::Int, "2");
        let three = read(ElementaryType::Int, "3");
        assert_eq!(text_of("LEN", std::slice::from_ref(&hello)), "5");
        assert_eq!(text_of("LEFT", &[hello.clone(), two.clone()]), "'he'");
        assert_eq!(text_of("RIGHT", &[hello.clone(), two.clone()]), "'lo'");
        assert_eq!(
            text_of("MID", &[hello.clone(), three.clone(), two.clone()]),
            "'ell'"
        );
        assert_eq!(
            text_of("CONCAT", &[hello.clone(), world.clone()]),
            "'helloworld'"
        );
        assert_eq!(
            text_of("INSERT", &[hello.clone(), world.clone(), two.clone()]),
            "'heworldllo'"
        );
        assert_eq!(
            text_of("DELETE", &[hello.clone(), two.clone(), three.clone()]),
            "'heo'"
        );
        assert_eq!(
            text_of(
                "REPLACE",
                &[hello.clone(), world.clone(), two.clone(), three.clone()]
            ),
            "'heworldo'"
        );
        assert_eq!(
            text_of(
                "FIND",
                &[hello.clone(), read(ElementaryType::String, "'llo'")]
            ),
            "3"
        );
        assert_eq!(text_of("FIND", &[hello.clone(), world]), "0");
        assert_eq!(
            code("MID", &[hello.clone(), read(ElementaryType::Int, "9"), two]),
            ShlitaCode::IndexOutOfRange
        );
        assert_eq!(
            code(
                "CONCAT",
                &[hello, read(ElementaryType::WString, "\"wide\"")]
            ),
            ShlitaCode::TypeMismatch
        );
    }

    #[test]
    fn a_computed_string_stops_at_the_implementations_maximum() {
        let long = ElementaryValue::Text {
            ty: ElementaryType::String,
            value: "x".repeat(200),
        };
        assert_eq!(
            code("CONCAT", &[long.clone(), long]),
            ShlitaCode::StringTooLong
        );
    }
}
