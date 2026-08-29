//! Numeric codes for [`crate::error::TypeError`].
//!
//! The same discipline ADR 0008 established for parse errors, applied to the
//! layer above: a caller asks "is this specifically an out-of-range integer?"
//! by matching a number, not by matching message text. The code is API; the
//! wording is not.
//!
//! Codes are three digits, banded by the group the failing type belongs to
//! (see [`crate::TypeGroup::code_band`]):
//!
//! - `1-99` general -- not specific to any one group
//! - `101-199` primitives
//! - `201-299` simple types
//! - `301-399` complex types
//! - `401-499` collections
//! - `501-599` reserved for `shelishi_schema`, the layer above, which
//!   supplies types from a runtime schema
//!
//! Only the general, primitive, and simple bands are populated so far; the
//! rest are reserved so a number is never reused for a different meaning.
//!
//! Where a failure in one band mirrors one in another, it takes the mirrored
//! low two digits and carries its band in its name -- [`SheniCode::LeadingPlus`]
//! at 106 and [`SheniCode::SimpleLeadingPlus`] at 206 are the same problem in
//! two groups.

use std::fmt;

use crate::group::TypeGroup;

/// A stable numeric identifier for a typing failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u16)]
pub enum SheniCode {
    // ---- General (1-99) ----
    /// A type was asked to read a node that is not a scalar -- a mapping or a
    /// sequence where a single value was declared.
    NotAScalar = 1,
    /// A type name in a schema matches no known type.
    UnknownTypeName = 2,

    // ---- Primitives (101-199) ----
    /// The literal is empty. Empty text is a valid `string` and nothing else.
    EmptyLiteral = 101,
    /// The literal is not one of the accepted boolean spellings.
    NotABoolean = 102,
    /// `on` or `off`, reserved by ADR sheni-0001 and not accepted yet.
    ReservedBooleanSpelling = 103,
    /// The literal is not a decimal integer.
    NotAnInteger = 104,
    /// A leading zero on a multi-digit integer, which reads as octal in
    /// neighbouring languages and is rejected rather than guessed at.
    LeadingZero = 105,
    /// A leading `+`. Only `-`, and only on a signed type, is accepted.
    LeadingPlus = 106,
    /// A digit separator such as `1_000`.
    DigitSeparator = 107,
    /// A negative literal read at an unsigned type.
    NegativeInUnsigned = 108,
    /// A well-formed integer that does not fit the type's width.
    IntegerOutOfRange = 109,
    /// The literal is not a decimal float.
    NotAFloat = 110,
    /// `NaN` or an infinity, which have no agreed text form across formats
    /// and do not survive a round trip.
    NonFiniteFloat = 111,
    /// A finite literal that overflows to an infinity at the type's width.
    FloatOutOfRange = 112,
    /// A `char` literal that is not exactly one Unicode scalar value.
    NotASingleCharacter = 113,
    /// A `byte` literal whose code point does not fit in `0..=255`.
    ByteOutOfRange = 114,
    /// `unknown` written where a strict primitive was declared. The author is
    /// reaching for the soft twin, and naming it is more use than reporting a
    /// malformed literal. Never raised at `string`, where `unknown` is text.
    UnknownAtStrictType = 115,

    // ---- Simple types (201-299) ----
    /// The literal is empty. Mirrors [`Self::EmptyLiteral`]; no simple type
    /// has an empty form.
    SimpleEmptyValue = 201,
    /// A leading `+` on a date, which the `time` crate would read as an
    /// expanded year. Mirrors [`Self::LeadingPlus`].
    SimpleLeadingPlus = 206,

    /// Not an ISO 8601 calendar date.
    MalformedDate = 220,
    /// Not an ISO 8601 wall-clock time.
    MalformedTime = 221,
    /// Not an RFC 3339 timestamp.
    MalformedTimestamp = 222,
    /// Not a well-formed duration in the type's convention. Shared by
    /// `duration`, `duration_iso`, and `duration_human`, which differ in the
    /// text they accept but not in what going wrong means.
    MalformedDuration = 223,
    /// Not a UUID in any spelling.
    MalformedUuid = 224,
    /// A UUID in a spelling the `uuid` crate accepts but ADR sheni-0002 does
    /// not -- braced, URN-prefixed, or unhyphenated. Recognisable and wrong,
    /// as distinct from unreadable.
    NonCanonicalUuid = 225,
    /// Not a valid RFC 5322 address.
    MalformedEmail = 226,
    /// Not a valid URL.
    MalformedUrl = 227,
    /// A relative URL, which has no meaning without a base to resolve it
    /// against. Distinct from [`Self::MalformedUrl`]: the text is well
    /// formed, it is just not absolute.
    RelativeUrl = 228,
    /// Not an IPv4 or IPv6 address.
    MalformedIpAddress = 229,
    /// Three uppercase letters that are not an ISO 4217 code, or not three
    /// uppercase letters at all.
    UnknownCurrencyCode = 230,
    /// Not a language tag in the form `en` or `en.EN`.
    MalformedLanguage = 231,
    /// Not usable as a single file name -- empty, or carrying a path
    /// separator, a control character, or a `.` / `..` meaning.
    MalformedFileName = 232,
    /// Not usable as a path -- empty, or carrying a control character.
    MalformedPath = 233,
    /// Not an EDTF expression at all. Shared by `soft_date` and
    /// `soft_date_range`, which delegate to one parser and fail the same way.
    MalformedEdtf = 234,
    /// An EDTF set, `[one of]` or `{all of}`. Both brackets are forbidden
    /// constructs in SYON, so the value is unwritable rather than merely
    /// unsupported -- see ADR sheni_06.
    EdtfSetNotWritable = 235,
    /// Well-formed EDTF of the wrong shape for the declared type: an interval
    /// at `soft_date`, a date at `soft_date_range`, a date with a time of day
    /// at either. Right family, wrong member; the message names the type the
    /// author wanted.
    EdtfShapeMismatch = 236,
}

impl SheniCode {
    /// The code as a number, for callers across an FFI or a wire format.
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// The group this code is banded under, or `None` for a general code
    /// below 100.
    pub fn group(self) -> Option<TypeGroup> {
        let band = (self.as_u16() / 100) * 100;
        TypeGroup::ALL.into_iter().find(|g| g.code_band() == band)
    }
}

impl fmt::Display for SheniCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SHENI-{:03}", self.as_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Codes are a compatibility surface: once published a number cannot move.
    /// This pins the ones that exist so a renumbering cannot land silently.
    #[test]
    fn codes_are_pinned() {
        assert_eq!(SheniCode::NotAScalar.as_u16(), 1);
        assert_eq!(SheniCode::UnknownTypeName.as_u16(), 2);
        assert_eq!(SheniCode::EmptyLiteral.as_u16(), 101);
        assert_eq!(SheniCode::NotABoolean.as_u16(), 102);
        assert_eq!(SheniCode::ReservedBooleanSpelling.as_u16(), 103);
        assert_eq!(SheniCode::NotAnInteger.as_u16(), 104);
        assert_eq!(SheniCode::LeadingZero.as_u16(), 105);
        assert_eq!(SheniCode::LeadingPlus.as_u16(), 106);
        assert_eq!(SheniCode::DigitSeparator.as_u16(), 107);
        assert_eq!(SheniCode::NegativeInUnsigned.as_u16(), 108);
        assert_eq!(SheniCode::IntegerOutOfRange.as_u16(), 109);
        assert_eq!(SheniCode::NotAFloat.as_u16(), 110);
        assert_eq!(SheniCode::NonFiniteFloat.as_u16(), 111);
        assert_eq!(SheniCode::FloatOutOfRange.as_u16(), 112);
        assert_eq!(SheniCode::NotASingleCharacter.as_u16(), 113);
        assert_eq!(SheniCode::ByteOutOfRange.as_u16(), 114);
        assert_eq!(SheniCode::UnknownAtStrictType.as_u16(), 115);
        assert_eq!(SheniCode::SimpleEmptyValue.as_u16(), 201);
        assert_eq!(SheniCode::SimpleLeadingPlus.as_u16(), 206);
        assert_eq!(SheniCode::MalformedDate.as_u16(), 220);
        assert_eq!(SheniCode::MalformedTime.as_u16(), 221);
        assert_eq!(SheniCode::MalformedTimestamp.as_u16(), 222);
        assert_eq!(SheniCode::MalformedDuration.as_u16(), 223);
        assert_eq!(SheniCode::MalformedUuid.as_u16(), 224);
        assert_eq!(SheniCode::NonCanonicalUuid.as_u16(), 225);
        assert_eq!(SheniCode::MalformedEmail.as_u16(), 226);
        assert_eq!(SheniCode::MalformedUrl.as_u16(), 227);
        assert_eq!(SheniCode::RelativeUrl.as_u16(), 228);
        assert_eq!(SheniCode::MalformedIpAddress.as_u16(), 229);
        assert_eq!(SheniCode::UnknownCurrencyCode.as_u16(), 230);
        assert_eq!(SheniCode::MalformedLanguage.as_u16(), 231);
        assert_eq!(SheniCode::MalformedFileName.as_u16(), 232);
        assert_eq!(SheniCode::MalformedPath.as_u16(), 233);
        assert_eq!(SheniCode::MalformedEdtf.as_u16(), 234);
        assert_eq!(SheniCode::EdtfSetNotWritable.as_u16(), 235);
        assert_eq!(SheniCode::EdtfShapeMismatch.as_u16(), 236);
    }

    #[test]
    fn display_is_zero_padded() {
        assert_eq!(SheniCode::NotAScalar.to_string(), "SHENI-001");
        assert_eq!(SheniCode::ByteOutOfRange.to_string(), "SHENI-114");
    }

    #[test]
    fn codes_report_their_band() {
        assert_eq!(SheniCode::NotAScalar.group(), None);
        assert_eq!(SheniCode::LeadingZero.group(), Some(TypeGroup::Primitive));
        assert_eq!(SheniCode::MalformedDate.group(), Some(TypeGroup::Simple));
    }

    /// A code that mirrors one in another band keeps the low two digits.
    #[test]
    fn mirrored_codes_share_their_low_digits() {
        for (a, b) in [
            (SheniCode::EmptyLiteral, SheniCode::SimpleEmptyValue),
            (SheniCode::LeadingPlus, SheniCode::SimpleLeadingPlus),
        ] {
            assert_eq!(a.as_u16() % 100, b.as_u16() % 100, "{a} vs {b}");
            assert_ne!(a.group(), b.group());
        }
    }
}
