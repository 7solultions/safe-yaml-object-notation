//! Soft primitives: a primitive whose value space contains an unknown.
//!
//! ADR sheni_03 tied optionality to the type -- a field may be optional only
//! where its type has a fallback -- and gave the numerics a fallback of zero.
//! Zero is an answer, though. A count of zero is a fact somebody established,
//! not a sign that nobody looked, and a field falling back to it conflates the
//! two.
//!
//! A soft primitive keeps them apart. It accepts exactly what its strict twin
//! accepts, plus the word `unknown`, and its fallback is that unknown rather
//! than a legal value borrowed to stand in for one:
//!
//! ```
//! use sheni_types::{PrimitiveType, SoftPrimitiveType, SoftPrimitiveValue};
//!
//! let soft = SoftPrimitiveType::from_name("soft_u8").unwrap();
//! assert_eq!(soft.read("unknown"), Ok(SoftPrimitiveValue::unknown(PrimitiveType::from_name("u8").unwrap())));
//! assert_eq!(soft.read("0").unwrap().to_string(), "0");
//!
//! // Softness is not leniency: every check the strict twin makes is made here.
//! assert_eq!(soft.read("300").unwrap_err().code(), sheni_types::SheniCode::IntegerOutOfRange);
//! ```
//!
//! Fifteen of the sixteen primitives have a twin. `string` does not, because
//! it accepts any well-formed UTF-8 verbatim, so no text lies outside its
//! value space and no word can mean "not known" without also being a string
//! somebody meant literally. See
//! `design/architecture/ADR_sheni_09__soft_primitives.syon`.

use std::fmt;

use serde::{Deserialize, Serialize};
use syon_parser::Value;

use crate::error::TypeError;
use crate::error_code::SheniCode;
use crate::group::TypeGroup;
use crate::primitives::{FloatWidth, IntWidth, PrimitiveType, PrimitiveValue};

/// The one word that means "not known", accepted case-insensitively and
/// written back lowercase.
pub const UNKNOWN_LITERAL: &str = "unknown";

/// A primitive type widened by one value: the unknown.
///
/// Constructed only for a primitive that can spell its own unknown, which is
/// every primitive except [`PrimitiveType::String`] -- see
/// [`SoftPrimitiveType::new`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct SoftPrimitiveType(PrimitiveType);

impl SoftPrimitiveType {
    /// Every soft primitive, in the order [`PrimitiveType::ALL`] lists their
    /// strict twins. Fifteen, not sixteen: there is no `soft_string`.
    pub const ALL: [SoftPrimitiveType; 15] = [
        SoftPrimitiveType(PrimitiveType::Boolean),
        SoftPrimitiveType(PrimitiveType::Unsigned(IntWidth::W8)),
        SoftPrimitiveType(PrimitiveType::Unsigned(IntWidth::W16)),
        SoftPrimitiveType(PrimitiveType::Unsigned(IntWidth::W32)),
        SoftPrimitiveType(PrimitiveType::Unsigned(IntWidth::W64)),
        SoftPrimitiveType(PrimitiveType::Unsigned(IntWidth::W128)),
        SoftPrimitiveType(PrimitiveType::Signed(IntWidth::W8)),
        SoftPrimitiveType(PrimitiveType::Signed(IntWidth::W16)),
        SoftPrimitiveType(PrimitiveType::Signed(IntWidth::W32)),
        SoftPrimitiveType(PrimitiveType::Signed(IntWidth::W64)),
        SoftPrimitiveType(PrimitiveType::Signed(IntWidth::W128)),
        SoftPrimitiveType(PrimitiveType::Float(FloatWidth::W32)),
        SoftPrimitiveType(PrimitiveType::Float(FloatWidth::W64)),
        SoftPrimitiveType(PrimitiveType::Byte),
        SoftPrimitiveType(PrimitiveType::Char),
    ];

    /// The soft twin of `inner`, or `None` where the type cannot spell its own
    /// unknown.
    ///
    /// `string` is the only such type, and it is not a limitation of this
    /// crate: a type that accepts any text verbatim has no text left over to
    /// mean "not known".
    pub const fn new(inner: PrimitiveType) -> Option<Self> {
        match inner {
            PrimitiveType::String => None,
            other => Some(SoftPrimitiveType(other)),
        }
    }

    /// The strict twin this type softens.
    pub const fn strict(self) -> PrimitiveType {
        self.0
    }

    /// [`TypeGroup::Primitive`], the same group as the strict twin. Softness
    /// is a property of the value space rather than a kind.
    pub const fn group(self) -> TypeGroup {
        TypeGroup::Primitive
    }

    /// The type's name as it is written in a schema: the strict twin's name
    /// under a `soft_` prefix.
    pub const fn name(self) -> &'static str {
        match self.0 {
            PrimitiveType::Boolean => "soft_bool",
            PrimitiveType::Unsigned(IntWidth::W8) => "soft_u8",
            PrimitiveType::Unsigned(IntWidth::W16) => "soft_u16",
            PrimitiveType::Unsigned(IntWidth::W32) => "soft_u32",
            PrimitiveType::Unsigned(IntWidth::W64) => "soft_u64",
            PrimitiveType::Unsigned(IntWidth::W128) => "soft_u128",
            PrimitiveType::Signed(IntWidth::W8) => "soft_i8",
            PrimitiveType::Signed(IntWidth::W16) => "soft_i16",
            PrimitiveType::Signed(IntWidth::W32) => "soft_i32",
            PrimitiveType::Signed(IntWidth::W64) => "soft_i64",
            PrimitiveType::Signed(IntWidth::W128) => "soft_i128",
            PrimitiveType::Float(FloatWidth::W32) => "soft_f32",
            PrimitiveType::Float(FloatWidth::W64) => "soft_f64",
            PrimitiveType::Byte => "soft_byte",
            PrimitiveType::Char => "soft_char",
            // Unconstructible: `new` refuses it and the field is private.
            PrimitiveType::String => "soft_string",
        }
    }

    /// The reverse of [`Self::name`]. Case-sensitive, for the reason
    /// [`PrimitiveType::from_name`] is. `soft_string` is not a name.
    pub fn from_name(name: &str) -> Option<Self> {
        SoftPrimitiveType::ALL
            .into_iter()
            .find(|t| t.name() == name)
    }

    /// The value an optional field of this type takes when its key is absent.
    ///
    /// Every soft primitive has one, which is what makes the whole family
    /// usable for optional fields where the strict twins are not.
    pub const fn fallback(self) -> SoftPrimitiveValue {
        SoftPrimitiveValue::Unknown(self.0)
    }

    /// Read a literal at this type.
    ///
    /// `unknown` in any casing is the unknown. Everything else is handed to
    /// the strict twin unchanged -- same shapes, same ranges, same codes --
    /// and only the type name on a failure is rewritten to say which of the
    /// twins was declared.
    pub fn read(self, literal: &str) -> Result<SoftPrimitiveValue, TypeError> {
        if literal.eq_ignore_ascii_case(UNKNOWN_LITERAL) {
            return Ok(SoftPrimitiveValue::Unknown(self.0));
        }
        match self.0.read(literal) {
            Ok(value) => Ok(SoftPrimitiveValue::Known(value)),
            Err(err) => Err(TypeError::new(
                err.code(),
                self.name(),
                err.literal(),
                err.message(),
            )),
        }
    }

    /// Read a parsed SYON node at this type, as
    /// [`PrimitiveType::read_value`] does.
    pub fn read_value(self, value: &Value) -> Result<SoftPrimitiveValue, TypeError> {
        match value {
            Value::Scalar(text) | Value::LiteralBlock(text) => self.read(text),
            Value::Mapping(_) => Err(TypeError::new(
                SheniCode::NotAScalar,
                self.name(),
                "",
                "expected a single value, found a mapping",
            )),
            Value::Sequence(_) => Err(TypeError::new(
                SheniCode::NotAScalar,
                self.name(),
                "",
                "expected a single value, found a sequence",
            )),
        }
    }
}

impl fmt::Display for SoftPrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<SoftPrimitiveType> for String {
    fn from(value: SoftPrimitiveType) -> Self {
        value.name().to_string()
    }
}

impl TryFrom<String> for SoftPrimitiveType {
    type Error = TypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SoftPrimitiveType::from_name(&value).ok_or_else(|| {
            TypeError::new(
                SheniCode::UnknownTypeName,
                "soft primitive",
                value,
                "no soft primitive type by that name",
            )
        })
    }
}

/// A value of a soft primitive type: either a primitive value, or the unknown.
///
/// The unknown carries the type it is unknown at, so an unknown `u8` and an
/// unknown `bool` are different values. This crate already holds that the type
/// is part of the value rather than something recoverable from it -- a `u8`
/// and a `u64` holding seven are not equal -- and an unknown that forgot its
/// type would be the one value that broke the rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SoftPrimitiveValue {
    /// Nobody knows, at this type.
    Unknown(PrimitiveType),
    /// A value of the strict twin.
    Known(PrimitiveValue),
}

impl SoftPrimitiveValue {
    /// The unknown at `inner`, or `None` where `inner` has no soft twin.
    pub fn unknown(inner: PrimitiveType) -> Self {
        SoftPrimitiveValue::Unknown(inner)
    }

    /// The type this value was read at, or `None` for an unknown constructed
    /// at `string`, which has no soft type to report.
    pub fn type_of(&self) -> Option<SoftPrimitiveType> {
        match self {
            SoftPrimitiveValue::Unknown(inner) => SoftPrimitiveType::new(*inner),
            SoftPrimitiveValue::Known(value) => SoftPrimitiveType::new(value.type_of()),
        }
    }

    /// Whether a value is present. The inverse of asking whether it is the
    /// unknown.
    pub const fn is_known(&self) -> bool {
        matches!(self, SoftPrimitiveValue::Known(_))
    }

    /// The underlying primitive value, or `None` for the unknown.
    pub const fn known(&self) -> Option<&PrimitiveValue> {
        match self {
            SoftPrimitiveValue::Known(value) => Some(value),
            SoftPrimitiveValue::Unknown(_) => None,
        }
    }

    // ---- Kleene's strong three-valued logic, on `soft_bool` alone ----

    /// This value's position in Kleene's truth ordering -- `false` below
    /// `unknown` below `true` -- or `None` if it is not a `soft_bool`.
    ///
    /// The ordering is what makes the operations arithmetic, and the
    /// operations are where three-valued logic is reliably got wrong.
    pub const fn kleene_rank(&self) -> Option<u8> {
        match self {
            SoftPrimitiveValue::Known(PrimitiveValue::Boolean(false)) => Some(0),
            SoftPrimitiveValue::Unknown(PrimitiveType::Boolean) => Some(1),
            SoftPrimitiveValue::Known(PrimitiveValue::Boolean(true)) => Some(2),
            _ => None,
        }
    }

    /// The reverse of [`Self::kleene_rank`], for ranks `0..=2`.
    const fn from_kleene_rank(rank: u8) -> Option<Self> {
        match rank {
            0 => Some(SoftPrimitiveValue::Known(PrimitiveValue::Boolean(false))),
            1 => Some(SoftPrimitiveValue::Unknown(PrimitiveType::Boolean)),
            2 => Some(SoftPrimitiveValue::Known(PrimitiveValue::Boolean(true))),
            _ => None,
        }
    }

    /// Kleene conjunction: the minimum of the two ranks. `None` unless both
    /// operands are `soft_bool` values.
    pub fn and(&self, other: &Self) -> Option<Self> {
        let (a, b) = (self.kleene_rank()?, other.kleene_rank()?);
        Self::from_kleene_rank(a.min(b))
    }

    /// Kleene disjunction: the maximum of the two ranks.
    pub fn or(&self, other: &Self) -> Option<Self> {
        let (a, b) = (self.kleene_rank()?, other.kleene_rank()?);
        Self::from_kleene_rank(a.max(b))
    }

    /// Kleene negation: the complement against 2, which leaves `unknown`
    /// where it is.
    pub fn not(&self) -> Option<Self> {
        Self::from_kleene_rank(2 - self.kleene_rank()?)
    }
}

/// The canonical text form, which reads back at the same type as an equal
/// value. The unknown writes as `unknown`, lowercase whatever was typed.
impl fmt::Display for SoftPrimitiveValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SoftPrimitiveValue::Unknown(_) => f.write_str(UNKNOWN_LITERAL),
            SoftPrimitiveValue::Known(value) => write!(f, "{value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A literal the strict twin accepts, for a type-generic sweep.
    fn valid_literal(t: PrimitiveType) -> &'static str {
        match t {
            PrimitiveType::Boolean => "true",
            PrimitiveType::Unsigned(_) | PrimitiveType::Signed(_) => "7",
            PrimitiveType::Float(_) => "1.5",
            PrimitiveType::Byte | PrimitiveType::Char => "a",
            PrimitiveType::String => "anything",
        }
    }

    #[test]
    fn every_soft_type_names_itself_and_reads_back() {
        for t in SoftPrimitiveType::ALL {
            assert_eq!(SoftPrimitiveType::from_name(t.name()), Some(t));
            assert!(t.name().starts_with("soft_"), "{}", t.name());
            assert_eq!(&t.name()[5..], t.strict().name());
            assert_eq!(t.group(), TypeGroup::Primitive);
        }
    }

    /// Fifteen of sixteen. The gap is the point of ADR sheni_09, not an
    /// oversight, so it is pinned here rather than left to the record.
    #[test]
    fn there_is_no_soft_string() {
        assert_eq!(SoftPrimitiveType::new(PrimitiveType::String), None);
        assert_eq!(SoftPrimitiveType::from_name("soft_string"), None);
        assert_eq!(SoftPrimitiveType::ALL.len(), PrimitiveType::ALL.len() - 1);

        let softened: Vec<PrimitiveType> =
            SoftPrimitiveType::ALL.iter().map(|t| t.strict()).collect();
        for strict in PrimitiveType::ALL {
            assert_eq!(
                softened.contains(&strict),
                strict != PrimitiveType::String,
                "{strict}"
            );
        }
    }

    #[test]
    fn unknown_reads_at_every_soft_type_in_any_casing() {
        for t in SoftPrimitiveType::ALL {
            for spelling in ["unknown", "Unknown", "UNKNOWN"] {
                assert_eq!(
                    t.read(spelling),
                    Ok(SoftPrimitiveValue::Unknown(t.strict())),
                    "{} at {}",
                    spelling,
                    t.name()
                );
            }
        }
    }

    #[test]
    fn a_soft_type_accepts_everything_its_strict_twin_accepts() {
        for t in SoftPrimitiveType::ALL {
            let literal = valid_literal(t.strict());
            assert_eq!(
                t.read(literal),
                Ok(SoftPrimitiveValue::Known(t.strict().read(literal).unwrap())),
                "{literal} at {}",
                t.name()
            );
        }
    }

    /// ADR sheni_08 forbids a `soft_` type from being more lenient than its
    /// strict twin. Fifteen types is where that rule is worth checking rather
    /// than asserting: every rejection matches, code for code.
    #[test]
    fn softness_is_not_leniency() {
        let junk = ["", "@@", "007", "1_000", "+1", "NaN", "  7", "yes please"];
        for t in SoftPrimitiveType::ALL {
            for literal in junk {
                let strict = t.strict().read(literal);
                let soft = t.read(literal);
                match (strict, soft) {
                    (Err(a), Err(b)) => {
                        assert_eq!(a.code(), b.code(), "{literal:?} at {}", t.name());
                        assert_eq!(b.type_name(), t.name(), "{literal:?}");
                        assert_eq!(a.message(), b.message(), "{literal:?}");
                    }
                    (Ok(a), Ok(b)) => assert_eq!(SoftPrimitiveValue::Known(a), b),
                    (a, b) => panic!("{} disagree on {literal:?}: {a:?} vs {b:?}", t.name()),
                }
            }
        }
    }

    #[test]
    fn unknown_at_a_strict_type_names_the_twin() {
        let err = PrimitiveType::Boolean.read("unknown").unwrap_err();
        assert_eq!(err.code(), SheniCode::UnknownAtStrictType);
        assert_eq!(
            err.message(),
            "`unknown` is a value of `soft_bool`, not of `bool`"
        );

        for t in PrimitiveType::ALL {
            if t == PrimitiveType::String {
                continue;
            }
            assert_eq!(
                t.read("unknown").unwrap_err().code(),
                SheniCode::UnknownAtStrictType,
                "{t}"
            );
        }
    }

    /// `string` accepts any text verbatim, which is why it has no soft twin
    /// and why `unknown` at it is still just text.
    #[test]
    fn unknown_is_ordinary_text_at_string() {
        assert_eq!(
            PrimitiveType::String.read("unknown"),
            Ok(PrimitiveValue::String("unknown".to_string()))
        );
    }

    #[test]
    fn every_soft_type_has_a_fallback_and_it_is_the_unknown() {
        for t in SoftPrimitiveType::ALL {
            assert_eq!(t.fallback(), SoftPrimitiveValue::Unknown(t.strict()));
            assert!(!t.fallback().is_known());
            assert_eq!(t.fallback().known(), None);
        }
    }

    /// The motivating case: an unset count must not read as a count of zero.
    #[test]
    fn an_unknown_count_is_not_a_count_of_zero() {
        let t = SoftPrimitiveType::from_name("soft_u32").unwrap();
        assert_ne!(t.fallback(), t.read("0").unwrap());
        assert_eq!(t.fallback().to_string(), "unknown");
        assert_eq!(t.read("0").unwrap().to_string(), "0");
    }

    #[test]
    fn an_unknown_carries_the_type_it_is_unknown_at() {
        let u8_unknown = SoftPrimitiveType::from_name("soft_u8").unwrap().fallback();
        let u64_unknown = SoftPrimitiveType::from_name("soft_u64").unwrap().fallback();
        let bool_unknown = SoftPrimitiveType::from_name("soft_bool")
            .unwrap()
            .fallback();

        assert_ne!(u8_unknown, u64_unknown);
        assert_ne!(u8_unknown, bool_unknown);
        assert_eq!(
            u8_unknown.type_of(),
            SoftPrimitiveType::from_name("soft_u8")
        );
    }

    #[test]
    fn the_canonical_form_reads_back_as_an_equal_value() {
        for t in SoftPrimitiveType::ALL {
            for literal in [valid_literal(t.strict()), "UNKNOWN"] {
                let value = t.read(literal).unwrap();
                assert_eq!(t.read(&value.to_string()), Ok(value), "{}", t.name());
            }
        }
    }

    fn soft_bool(literal: &str) -> SoftPrimitiveValue {
        SoftPrimitiveType::from_name("soft_bool")
            .unwrap()
            .read(literal)
            .unwrap()
    }

    #[test]
    fn kleene_ranks_run_false_below_unknown_below_true() {
        assert_eq!(soft_bool("false").kleene_rank(), Some(0));
        assert_eq!(soft_bool("unknown").kleene_rank(), Some(1));
        assert_eq!(soft_bool("true").kleene_rank(), Some(2));
    }

    #[test]
    fn kleene_conjunction_is_the_full_truth_table() {
        let table = [
            ("false", "false", "false"),
            ("false", "unknown", "false"),
            ("false", "true", "false"),
            ("unknown", "false", "false"),
            ("unknown", "unknown", "unknown"),
            ("unknown", "true", "unknown"),
            ("true", "false", "false"),
            ("true", "unknown", "unknown"),
            ("true", "true", "true"),
        ];
        for (a, b, expected) in table {
            assert_eq!(
                soft_bool(a).and(&soft_bool(b)),
                Some(soft_bool(expected)),
                "{a} and {b}"
            );
        }
    }

    #[test]
    fn kleene_disjunction_is_the_full_truth_table() {
        let table = [
            ("false", "false", "false"),
            ("false", "unknown", "unknown"),
            ("false", "true", "true"),
            ("unknown", "false", "unknown"),
            ("unknown", "unknown", "unknown"),
            ("unknown", "true", "true"),
            ("true", "false", "true"),
            ("true", "unknown", "true"),
            ("true", "true", "true"),
        ];
        for (a, b, expected) in table {
            assert_eq!(
                soft_bool(a).or(&soft_bool(b)),
                Some(soft_bool(expected)),
                "{a} or {b}"
            );
        }
    }

    /// Negation leaves the unknown where it is, which is the property that
    /// separates Kleene's logic from a two-valued one with a null bolted on.
    #[test]
    fn kleene_negation_fixes_the_unknown() {
        assert_eq!(soft_bool("false").not(), Some(soft_bool("true")));
        assert_eq!(soft_bool("true").not(), Some(soft_bool("false")));
        assert_eq!(soft_bool("unknown").not(), Some(soft_bool("unknown")));
    }

    #[test]
    fn the_operations_belong_to_soft_bool_alone() {
        let seven = SoftPrimitiveType::from_name("soft_u8")
            .unwrap()
            .read("7")
            .unwrap();
        let unknown_u8 = SoftPrimitiveType::from_name("soft_u8").unwrap().fallback();

        assert_eq!(seven.kleene_rank(), None);
        assert_eq!(unknown_u8.kleene_rank(), None);
        assert_eq!(seven.not(), None);
        assert_eq!(seven.and(&soft_bool("true")), None);
        assert_eq!(soft_bool("true").or(&seven), None);
    }

    #[test]
    fn a_mapping_or_sequence_is_not_a_single_value() {
        let t = SoftPrimitiveType::from_name("soft_u8").unwrap();
        for value in [Value::Mapping(Vec::new()), Value::Sequence(Vec::new())] {
            let err = t.read_value(&value).unwrap_err();
            assert_eq!(err.code(), SheniCode::NotAScalar);
            assert_eq!(err.type_name(), "soft_u8");
        }
    }

    #[test]
    fn a_scalar_node_reads_at_its_declared_soft_type() {
        let t = SoftPrimitiveType::from_name("soft_char").unwrap();
        assert_eq!(
            t.read_value(&Value::Scalar("q".to_string())),
            Ok(SoftPrimitiveValue::Known(PrimitiveValue::Char('q')))
        );
        assert_eq!(
            t.read_value(&Value::LiteralBlock("unknown".to_string())),
            Ok(SoftPrimitiveValue::Unknown(PrimitiveType::Char))
        );
    }

    #[test]
    fn a_type_serialises_as_its_name() {
        for t in SoftPrimitiveType::ALL {
            let json = serde_json::to_string(&t).unwrap();
            assert_eq!(json, format!("{:?}", t.name()));
            assert_eq!(serde_json::from_str::<SoftPrimitiveType>(&json).unwrap(), t);
        }
    }

    #[test]
    fn soft_string_fails_to_deserialise_as_a_type_name() {
        let err = serde_json::from_str::<SoftPrimitiveType>("\"soft_string\"").unwrap_err();
        assert!(err
            .to_string()
            .contains("no soft primitive type by that name"));
    }
}
