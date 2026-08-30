//! Numeric codes for [`crate::error::ShlitaError`].
//!
//! The discipline is the one ADR syon_08 established for parse errors and
//! `sheni` and `luach_types` carried upwards: a caller asks "is this
//! specifically an out-of-range integer?" by matching a number, not by
//! matching message text. The code is API; the wording is not.
//!
//! Codes are three digits in the `701-799` band, reserved for the shlita
//! crates by ADR shlita_01. The bands below are spoken for -- `1-499` by
//! sheni's four type groups, `501-599` by `shelishi_schema`, and `601-699`
//! by the hodesh crates -- so a shlita code and a sheni code never collide
//! even where the two are reported side by side.
//!
//! The band is split again inside itself, because two crates share it:
//!
//! - `701-709` general -- not specific to any one type or function
//! - `710-729` elementary literals, raised by [`crate::ElementaryType::read`]
//! - `730-744` the standard functions
//! - `745-749` conversions to and from sheni's primitives
//! - `750-769` the scan runtime and the standard function blocks
//! - `770-799` program organisation units, FBD networks and SFC charts
//!
//! The last two ranges belong to `shlita_runtime` and are declared here
//! rather than there, because a reserved number is only a promise if one
//! list holds all of them. `ktav` is a language rather than a crate in this
//! band and takes `801-899` -- see `ktav::KtavCode`.

use std::fmt;

/// A stable numeric identifier for a shlita failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum ShlitaCode {
    // ---- General (701-709) ----
    /// A type was asked to read a node that is not a scalar.
    NotAScalar = 701,
    /// A type name matches no elementary type.
    UnknownTypeName = 702,
    /// The literal is empty. No elementary type has an empty form, not even
    /// STRING -- an empty string is written `''`.
    EmptyLiteral = 703,

    // ---- Elementary literals (710-729) ----
    /// Not one of the accepted BOOL spellings.
    NotABoolean = 710,
    /// Not an integer in any of the standard's bases.
    NotAnInteger = 711,
    /// A well-formed integer that does not fit the type's width.
    IntegerOutOfRange = 712,
    /// A based literal whose base is not 2, 8 or 16, or whose digits fall
    /// outside the base it declares.
    MalformedBase = 713,
    /// An underscore where the standard does not allow one -- leading,
    /// trailing, doubled, or adjacent to the `#` of a based literal.
    MisplacedUnderscore = 714,
    /// Not a REAL literal. The standard requires a decimal point with a
    /// digit on each side, or an exponent.
    NotAReal = 715,
    /// `NaN` or an infinity, which the standard does not give a literal form.
    NonFiniteReal = 716,
    /// A finite literal that overflows to an infinity at the type's width.
    RealOutOfRange = 717,
    /// A typed prefix naming a different type than the one being read --
    /// `INT#7` handed to DINT.
    WrongTypePrefix = 718,
    /// Not a TIME or LTIME literal.
    MalformedDuration = 719,
    /// A duration whose magnitude exceeds what the type can hold.
    DurationOutOfRange = 720,
    /// Not a DATE literal.
    MalformedDate = 721,
    /// Not a TIME_OF_DAY literal.
    MalformedTimeOfDay = 722,
    /// Not a DATE_AND_TIME literal.
    MalformedDateAndTime = 723,
    /// A well-formed date that does not exist, or falls outside the range
    /// the type's arithmetic is defined over.
    DateOutOfRange = 724,
    /// Not a character-string literal -- unquoted, or unterminated.
    MalformedString = 725,
    /// The wrong quote for the type: STRING is single-quoted and WSTRING is
    /// double-quoted, and the standard does not let either borrow the other.
    WrongStringQuote = 726,
    /// A `$` escape the standard does not define.
    MalformedEscape = 727,
    /// A CHAR or WCHAR literal that is not exactly one character.
    NotASingleCharacter = 728,
    /// A character whose code point does not fit the type -- CHAR holds one
    /// byte, WCHAR one UTF-16 code unit.
    CharacterOutOfRange = 729,

    // ---- Standard functions (730-744) ----
    /// No standard function goes by that name.
    UnknownFunction = 730,
    /// The right function, the wrong number of arguments.
    WrongArgumentCount = 731,
    /// An argument of a type the function is not defined over, or a mix of
    /// types where the standard requires one.
    TypeMismatch = 732,
    /// Arithmetic on something that is not a number.
    NotANumber = 733,
    /// A bitwise operation on something that is not a bit string. BYTE and
    /// WORD may be ANDed; USINT and UINT may not, and the standard keeps
    /// them distinct types for exactly this reason.
    NotABitString = 734,
    /// A division or a modulus by zero.
    DivisionByZero = 735,
    /// A result that does not fit the width of the type it is computed at.
    /// Reported rather than wrapped, on the reasoning sheni used for an
    /// out-of-range literal.
    ArithmeticOverflow = 736,
    /// An argument outside a function's domain -- SQRT of a negative, LN of
    /// a non-positive.
    DomainError = 737,
    /// A string index outside `1..=LEN`, or a length running past the end.
    IndexOutOfRange = 738,
    /// A string result longer than the implementation's maximum.
    StringTooLong = 739,
    /// A MUX selector with no corresponding input.
    SelectorOutOfRange = 740,

    // ---- Conversions (745-749) ----
    /// No conversion is defined between the two types.
    NotConvertible = 745,
    /// A conversion that is defined, on a value the target cannot hold.
    ConversionOutOfRange = 746,

    // ---- The scan runtime (750-769) ----
    /// A function block was called outside a scan, or with a scan context
    /// whose cycle time is not positive. TON accumulates elapsed time across
    /// scans, so asking for its value outside one is not a question with an
    /// answer.
    NoScanContext = 750,
    /// A function block input that is required was not supplied.
    MissingInput = 751,
    /// An input supplied under a name the block does not have.
    UnknownParameter = 752,
    /// A preset time or count that the block cannot work with -- a negative
    /// PT, or a PV outside the counter's width.
    InvalidPreset = 753,
    /// No standard function block goes by that name.
    UnknownFunctionBlock = 754,

    // ---- Documents: POU, FBD, SFC (770-799) ----
    /// The document is not a program organisation unit at all -- the top
    /// level key is missing or misspelled.
    NotAPou = 770,
    /// A required key of a POU, network or chart is missing.
    MissingKey = 771,
    /// A key that the PLCopen vocabulary does not define. Rejected rather
    /// than ignored, because a misspelled qualifier that is silently dropped
    /// is a chart that runs and is wrong.
    UnknownKey = 772,
    /// A key whose value is of the wrong shape -- a scalar where a sequence
    /// was required, or the reverse.
    WrongShape = 773,
    /// Two blocks, steps or variables sharing a name or a local id.
    DuplicateName = 774,
    /// A connection naming a local id that no block or variable carries.
    DanglingConnection = 775,
    /// A connection running backwards through the execution order without
    /// being marked as feedback. A network with feedback has no order the
    /// graph alone determines, so the break is declared rather than
    /// inferred -- see ADR shlita_02.
    UndeclaredFeedback = 776,
    /// Two blocks sharing an execution order id, which leaves the order of
    /// the two undetermined.
    DuplicateExecutionOrder = 777,
    /// A block naming a type that is neither a standard function nor a
    /// standard function block.
    UnknownBlockType = 778,
    /// A chart with no initial step, or with more than one.
    InitialStepNotUnique = 779,
    /// A transition naming a step that the chart does not contain.
    UnknownStep = 780,
    /// An action qualifier outside the nine the standard defines.
    UnknownQualifier = 781,
    /// A qualifier that takes a duration, written without one, or one that
    /// takes none, written with one.
    QualifierTimeMismatch = 782,
    /// A divergence whose branches do not reconverge, or a convergence with
    /// no matching divergence.
    UnbalancedDivergence = 783,
    /// A variable read by a body that the interface does not declare.
    UndeclaredVariable = 784,
    /// A POU that calls itself, directly or through a cycle. IEC 61131-3
    /// forbids recursion, and a controller scan has to terminate.
    RecursivePou = 785,
}

impl ShlitaCode {
    /// The code as a number, for a caller across an FFI or a wire format.
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// The first and last code of the range this code was allocated from,
    /// and the name of that range.
    pub const fn band(self) -> (u16, u16, &'static str) {
        match self.as_u16() {
            701..=709 => (701, 709, "general"),
            710..=729 => (710, 729, "elementary literals"),
            730..=744 => (730, 744, "standard functions"),
            745..=749 => (745, 749, "conversions"),
            750..=769 => (750, 769, "scan runtime"),
            _ => (770, 799, "documents"),
        }
    }
}

impl fmt::Display for ShlitaCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SHLITA-{}", self.as_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every code the two crates use, so a renumbering cannot land silently.
    const ALL: [ShlitaCode; 57] = [
        ShlitaCode::NotAScalar,
        ShlitaCode::UnknownTypeName,
        ShlitaCode::EmptyLiteral,
        ShlitaCode::NotABoolean,
        ShlitaCode::NotAnInteger,
        ShlitaCode::IntegerOutOfRange,
        ShlitaCode::MalformedBase,
        ShlitaCode::MisplacedUnderscore,
        ShlitaCode::NotAReal,
        ShlitaCode::NonFiniteReal,
        ShlitaCode::RealOutOfRange,
        ShlitaCode::WrongTypePrefix,
        ShlitaCode::MalformedDuration,
        ShlitaCode::DurationOutOfRange,
        ShlitaCode::MalformedDate,
        ShlitaCode::MalformedTimeOfDay,
        ShlitaCode::MalformedDateAndTime,
        ShlitaCode::DateOutOfRange,
        ShlitaCode::MalformedString,
        ShlitaCode::WrongStringQuote,
        ShlitaCode::MalformedEscape,
        ShlitaCode::NotASingleCharacter,
        ShlitaCode::CharacterOutOfRange,
        ShlitaCode::UnknownFunction,
        ShlitaCode::WrongArgumentCount,
        ShlitaCode::TypeMismatch,
        ShlitaCode::NotANumber,
        ShlitaCode::NotABitString,
        ShlitaCode::DivisionByZero,
        ShlitaCode::ArithmeticOverflow,
        ShlitaCode::DomainError,
        ShlitaCode::IndexOutOfRange,
        ShlitaCode::StringTooLong,
        ShlitaCode::SelectorOutOfRange,
        ShlitaCode::NotConvertible,
        ShlitaCode::ConversionOutOfRange,
        ShlitaCode::NoScanContext,
        ShlitaCode::MissingInput,
        ShlitaCode::UnknownParameter,
        ShlitaCode::InvalidPreset,
        ShlitaCode::UnknownFunctionBlock,
        ShlitaCode::NotAPou,
        ShlitaCode::MissingKey,
        ShlitaCode::UnknownKey,
        ShlitaCode::WrongShape,
        ShlitaCode::DuplicateName,
        ShlitaCode::DanglingConnection,
        ShlitaCode::UndeclaredFeedback,
        ShlitaCode::DuplicateExecutionOrder,
        ShlitaCode::UnknownBlockType,
        ShlitaCode::InitialStepNotUnique,
        ShlitaCode::UnknownStep,
        ShlitaCode::UnknownQualifier,
        ShlitaCode::QualifierTimeMismatch,
        ShlitaCode::UnbalancedDivergence,
        ShlitaCode::UndeclaredVariable,
        ShlitaCode::RecursivePou,
    ];

    #[test]
    fn codes_are_pinned() {
        assert_eq!(ShlitaCode::NotAScalar.as_u16(), 701);
        assert_eq!(ShlitaCode::NotABoolean.as_u16(), 710);
        assert_eq!(ShlitaCode::CharacterOutOfRange.as_u16(), 729);
        assert_eq!(ShlitaCode::UnknownFunction.as_u16(), 730);
        assert_eq!(ShlitaCode::SelectorOutOfRange.as_u16(), 740);
        assert_eq!(ShlitaCode::NotConvertible.as_u16(), 745);
        assert_eq!(ShlitaCode::NoScanContext.as_u16(), 750);
        assert_eq!(ShlitaCode::NotAPou.as_u16(), 770);
        assert_eq!(ShlitaCode::RecursivePou.as_u16(), 785);
    }

    /// ADR shlita_01 reserved 701-799 and nothing enforces it but this.
    #[test]
    fn every_code_stays_inside_the_reserved_band() {
        for code in ALL {
            let n = code.as_u16();
            assert!((701..=799).contains(&n), "{code} is outside 701-799");
            let (low, high, _) = code.band();
            assert!((low..=high).contains(&n), "{code} is outside its own band");
        }
    }

    #[test]
    fn display_carries_the_prefix() {
        assert_eq!(ShlitaCode::NotAScalar.to_string(), "SHLITA-701");
        assert_eq!(ShlitaCode::DivisionByZero.to_string(), "SHLITA-735");
    }

    #[test]
    fn bands_are_named() {
        assert_eq!(
            ShlitaCode::IntegerOutOfRange.band().2,
            "elementary literals"
        );
        assert_eq!(ShlitaCode::NoScanContext.band().2, "scan runtime");
        assert_eq!(ShlitaCode::UnknownQualifier.band().2, "documents");
    }
}
