//! The simple group: an interpretation laid over a primitive carrier.
//!
//! Fourteen types, each governed by a published standard and validated by the
//! crate that implements it -- `time` for dates, `url` for URLs, `uuid` for
//! UUIDs, and so on. Sheni adds only what a delegate would wave through but
//! [`crate::primitives`]'s rules have already ruled out; the short list of
//! those additions is in `design/architecture/ADR_sheni_02__simple_types.syon`.
//!
//! Reading a simple type **normalises**, which is the difference from the
//! layer below. A primitive value is its text, so the text is kept exactly. A
//! simple value is the thing the text denotes, so the meaning is kept and the
//! canonical form is what comes back out: `2001:0DB8::1` reads as
//! `2001:db8::1`, `90m` reads as `1h 30m`.

use std::fmt;
use std::net::IpAddr;
use std::str::FromStr;
use std::time::Duration as StdDuration;

use edtf_core::{Edtf, Interval, Precision};
use email_address::EmailAddress;
use iso_currency::Currency;
use serde::{Deserialize, Serialize, Serializer};
use syon_parser::Value;
use time::format_description::well_known::Rfc3339;
use time::macros::format_description;
use time::{Date, OffsetDateTime, Time};
use url::Url;
use uuid::Uuid;

use crate::error::TypeError;
use crate::error_code::SheniCode;
use crate::group::TypeGroup;
use crate::primitives::PrimitiveType;

/// A language tag in the form the crate README specifies: a primary subtag,
/// optionally followed by `.` and a region.
///
/// This is deliberately **not** BCP 47, which writes `en-GB`. ADR sheni-0002
/// records the divergence and expects it to be revisited.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageTag {
    primary: String,
    region: Option<String>,
}

impl LanguageTag {
    /// The primary subtag, two or three lowercase letters -- the `en` of
    /// `en.EN`.
    pub fn primary(&self) -> &str {
        &self.primary
    }

    /// The region subtag, two uppercase letters, if one was written.
    pub fn region(&self) -> Option<&str> {
        self.region.as_deref()
    }
}

impl fmt::Display for LanguageTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.region {
            Some(region) => write!(f, "{}.{}", self.primary, region),
            None => f.write_str(&self.primary),
        }
    }
}

/// The units of an IEC 61131-3 duration, longest first. The order is also the
/// order they must appear in, and a unit may appear at most once.
const IEC_UNITS: [(&str, u128); 7] = [
    ("d", 86_400_000_000_000),
    ("h", 3_600_000_000_000),
    ("m", 60_000_000_000),
    ("s", 1_000_000_000),
    ("ms", 1_000_000),
    ("us", 1_000),
    ("ns", 1),
];

/// A simple type.
///
/// Serialises as its name, as [`PrimitiveType`] does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum SimpleType {
    /// `uuid`. RFC 9562, canonical hyphenated form only.
    Uuid,
    /// `date`. ISO 8601 `2026-08-28`.
    Date,
    /// `time`. ISO 8601 `14:30:00`, with no leading designator.
    Time,
    /// `timestamp`. RFC 3339 `2026-08-28T14:30:00Z`.
    Timestamp,
    /// `duration`. IEC 61131-3 `T#5s500ms`.
    Duration,
    /// `duration_iso`. ISO 8601 `P1DT2H30M`.
    DurationIso,
    /// `duration_human`. The humantime convention, `1h30m`.
    DurationHuman,
    /// `file_name`. One path segment, `README.md`.
    FileName,
    /// `path`. `/path/to/file`. A text shape; nothing here touches a disk.
    PathName,
    /// `email`. RFC 5322 / RFC 6531.
    Email,
    /// `url`. Absolute, per the WHATWG URL Living Standard.
    Url,
    /// `ip_address`. IPv4 or IPv6; the value records which.
    IpAddress,
    /// `language`. `en` or `en.EN`. Not BCP 47 -- see [`LanguageTag`].
    Language,
    /// `currency_code`. ISO 4217 `USD`.
    CurrencyCode,
    /// `soft_date`. A date known to any precision, or flagged uncertain, or
    /// not known at all -- ISO 8601-2:2019 Annex A. See ADR sheni_06.
    SoftDate,
    /// `soft_date_range`. An EDTF interval, either end of which may be a
    /// coarse date, open, or unknown. See ADR sheni_10.
    SoftDateRange,
}

impl SimpleType {
    /// Every simple type, in the order the crate README lists them.
    pub const ALL: [SimpleType; 16] = [
        SimpleType::Uuid,
        SimpleType::Date,
        SimpleType::Time,
        SimpleType::Timestamp,
        SimpleType::Duration,
        SimpleType::DurationIso,
        SimpleType::DurationHuman,
        SimpleType::FileName,
        SimpleType::PathName,
        SimpleType::Email,
        SimpleType::Url,
        SimpleType::IpAddress,
        SimpleType::Language,
        SimpleType::CurrencyCode,
        SimpleType::SoftDate,
        SimpleType::SoftDateRange,
    ];

    /// Always [`TypeGroup::Simple`].
    pub const fn group(self) -> TypeGroup {
        TypeGroup::Simple
    }

    /// The primitive that carries this type's text form.
    ///
    /// Every simple type is carried by a string today. The method exists
    /// because the carrier is the thing that makes a type simple rather than
    /// primitive, and an integer-carried type is a plausible addition.
    pub const fn carrier(self) -> PrimitiveType {
        PrimitiveType::String
    }

    /// The type's name as it is written in a schema.
    pub const fn name(self) -> &'static str {
        match self {
            SimpleType::Uuid => "uuid",
            SimpleType::Date => "date",
            SimpleType::Time => "time",
            SimpleType::Timestamp => "timestamp",
            SimpleType::Duration => "duration",
            SimpleType::DurationIso => "duration_iso",
            SimpleType::DurationHuman => "duration_human",
            SimpleType::FileName => "file_name",
            SimpleType::PathName => "path",
            SimpleType::Email => "email",
            SimpleType::Url => "url",
            SimpleType::IpAddress => "ip_address",
            SimpleType::Language => "language",
            SimpleType::CurrencyCode => "currency_code",
            SimpleType::SoftDate => "soft_date",
            SimpleType::SoftDateRange => "soft_date_range",
        }
    }

    /// The value an optional field of this type takes when its key is absent,
    /// or `None` where the type has none and a field of it must be required.
    ///
    /// ADR sheni_03 allows a fallback only where it is a member of the value
    /// space meaning "not known", never a legal value borrowed to stand in for
    /// one. Two simple types qualify. `soft_date` falls back to the fully
    /// unspecified date `XXXX`, and `soft_date_range` to `XXXX/XXXX` -- which
    /// the standard chose, since EDTF rejects `/` and `../..` outright and an
    /// interval must have a dated endpoint.
    pub fn fallback(self) -> Option<SimpleValue> {
        match self {
            SimpleType::SoftDate => Some(self.read("XXXX").expect("XXXX is EDTF")),
            SimpleType::SoftDateRange => Some(self.read("XXXX/XXXX").expect("XXXX/XXXX is EDTF")),
            _ => None,
        }
    }

    /// The reverse of [`Self::name`]. Case-sensitive, as elsewhere.
    pub fn from_name(name: &str) -> Option<Self> {
        SimpleType::ALL.into_iter().find(|t| t.name() == name)
    }

    /// Read a literal at this type.
    ///
    /// The text is taken exactly as the parser produced it -- no trimming, for
    /// the reason [`PrimitiveType::read`] gives. The value that comes back is
    /// normalised; see the module docs.
    pub fn read(self, literal: &str) -> Result<SimpleValue, TypeError> {
        if literal.is_empty() {
            return Err(self.err(
                SheniCode::SimpleEmptyValue,
                literal,
                "expected a value, found empty text",
            ));
        }
        match self {
            SimpleType::Uuid => self.read_uuid(literal),
            SimpleType::Date => self.read_date(literal),
            SimpleType::Time => self.read_time(literal),
            SimpleType::Timestamp => self.read_timestamp(literal),
            SimpleType::Duration => self.read_iec_duration(literal),
            SimpleType::DurationIso => self.read_iso_duration(literal),
            SimpleType::DurationHuman => self.read_human_duration(literal),
            SimpleType::FileName => self.read_file_name(literal),
            SimpleType::PathName => self.read_path(literal),
            SimpleType::Email => self.read_email(literal),
            SimpleType::Url => self.read_url(literal),
            SimpleType::IpAddress => self.read_ip_address(literal),
            SimpleType::Language => self.read_language(literal),
            SimpleType::CurrencyCode => self.read_currency_code(literal),
            SimpleType::SoftDate => self.read_soft_date(literal),
            SimpleType::SoftDateRange => self.read_soft_date_range(literal),
        }
    }

    /// Read a parsed SYON node at this type, as [`PrimitiveType::read_value`].
    pub fn read_value(self, value: &Value) -> Result<SimpleValue, TypeError> {
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

    // ---- delegated readers ----

    /// The `uuid` crate also accepts braced, URN-prefixed, and unhyphenated
    /// spellings. Those are recognisable and wrong rather than unreadable, so
    /// they get their own code.
    fn read_uuid(self, literal: &str) -> Result<SimpleValue, TypeError> {
        let canonical_shape = literal.len() == 36
            && literal.as_bytes().iter().enumerate().all(|(i, b)| {
                if matches!(i, 8 | 13 | 18 | 23) {
                    *b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            });
        match (canonical_shape, Uuid::parse_str(literal)) {
            (true, Ok(uuid)) => Ok(SimpleValue::Uuid(uuid)),
            (false, Ok(_)) => Err(self.err(
                SheniCode::NonCanonicalUuid,
                literal,
                "write the canonical hyphenated form, without braces or a `urn:uuid:` prefix",
            )),
            (_, Err(_)) => Err(self.err(
                SheniCode::MalformedUuid,
                literal,
                "expected 32 hexadecimal digits in 8-4-4-4-12 groups",
            )),
        }
    }

    /// `time` reads a leading `+` as an expanded-year sign. A year is a
    /// number, so the leading-plus rule from ADR sheni-0001 applies.
    fn read_date(self, literal: &str) -> Result<SimpleValue, TypeError> {
        if literal.starts_with('+') {
            return Err(self.err(
                SheniCode::SimpleLeadingPlus,
                literal,
                "a leading `+` is not accepted on a year",
            ));
        }
        Date::parse(literal, format_description!("[year]-[month]-[day]"))
            .map(SimpleValue::Date)
            .map_err(|_| {
                self.err(
                    SheniCode::MalformedDate,
                    literal,
                    "expected an ISO 8601 calendar date, `YYYY-MM-DD`",
                )
            })
    }

    fn read_time(self, literal: &str) -> Result<SimpleValue, TypeError> {
        Time::parse(literal, format_description!("[hour]:[minute]:[second]"))
            .map(SimpleValue::Time)
            .map_err(|_| {
                self.err(
                    SheniCode::MalformedTime,
                    literal,
                    "expected an ISO 8601 wall-clock time, `HH:MM:SS`, with no leading `T`",
                )
            })
    }

    fn read_timestamp(self, literal: &str) -> Result<SimpleValue, TypeError> {
        OffsetDateTime::parse(literal, &Rfc3339)
            .map(SimpleValue::Timestamp)
            .map_err(|_| {
                self.err(
                    SheniCode::MalformedTimestamp,
                    literal,
                    "expected an RFC 3339 timestamp, `2026-08-28T14:30:00Z`",
                )
            })
    }

    fn read_email(self, literal: &str) -> Result<SimpleValue, TypeError> {
        EmailAddress::from_str(literal)
            .map(SimpleValue::Email)
            .map_err(|e| self.err(SheniCode::MalformedEmail, literal, e.to_string()))
    }

    /// A relative URL is well formed but has no meaning without a base, which
    /// this layer has no way to supply. It is told apart from malformed text.
    fn read_url(self, literal: &str) -> Result<SimpleValue, TypeError> {
        match Url::parse(literal) {
            Ok(url) => Ok(SimpleValue::Url(url)),
            Err(url::ParseError::RelativeUrlWithoutBase) => Err(self.err(
                SheniCode::RelativeUrl,
                literal,
                "expected an absolute URL, with a scheme",
            )),
            Err(e) => Err(self.err(SheniCode::MalformedUrl, literal, e.to_string())),
        }
    }

    fn read_ip_address(self, literal: &str) -> Result<SimpleValue, TypeError> {
        IpAddr::from_str(literal)
            .map(SimpleValue::IpAddress)
            .map_err(|_| {
                self.err(
                    SheniCode::MalformedIpAddress,
                    literal,
                    "expected an IPv4 or IPv6 address",
                )
            })
    }

    fn read_currency_code(self, literal: &str) -> Result<SimpleValue, TypeError> {
        Currency::from_code(literal)
            .map(SimpleValue::CurrencyCode)
            .ok_or_else(|| {
                self.err(
                    SheniCode::UnknownCurrencyCode,
                    literal,
                    "expected a three-letter uppercase ISO 4217 code, such as `USD`",
                )
            })
    }

    // ---- EDTF: soft dates and soft date ranges ----

    /// ISO 8601-2:2019 Annex A, delegated to `edtf-core`.
    ///
    /// The two EDTF types share this reader and differ only in which shape
    /// they keep, so a set, a time of day, and a wrong-shaped-but-valid value
    /// report identically at both. See ADR sheni_06 and ADR sheni_10.
    fn read_edtf(self, literal: &str, want: EdtfShape) -> Result<Edtf, TypeError> {
        let parsed = Edtf::parse(literal).map_err(|err| {
            self.err(
                SheniCode::MalformedEdtf,
                literal,
                format!("expected an ISO 8601-2 date expression; {err}"),
            )
        })?;

        // A set is written with `[` or `{`, both of which the Go
        // implementation rejects outright and the Rust one carries through as
        // a scalar. The value is unwritable rather than unsupported, so it
        // says so specifically.
        if matches!(parsed, Edtf::Set(_)) {
            return Err(self.err(
                SheniCode::EdtfSetNotWritable,
                literal,
                "an EDTF set is written with `[` or `{`, which SYON does not carry",
            ));
        }

        let got = match &parsed {
            Edtf::Date(_) => EdtfShape::Date,
            Edtf::Interval(_) => EdtfShape::Interval,
            Edtf::DateTime(_) | Edtf::Set(_) => {
                return Err(self.err(
                    SheniCode::EdtfShapeMismatch,
                    literal,
                    "that is a date with a time of day; declare `timestamp`",
                ))
            }
        };
        if got != want {
            return Err(self.err(
                SheniCode::EdtfShapeMismatch,
                literal,
                format!("that is {}; declare `{}`", got.describe(), got.type_name()),
            ));
        }
        Ok(parsed)
    }

    fn read_soft_date(self, literal: &str) -> Result<SimpleValue, TypeError> {
        self.read_edtf(literal, EdtfShape::Date)
            .map(SimpleValue::SoftDate)
    }

    fn read_soft_date_range(self, literal: &str) -> Result<SimpleValue, TypeError> {
        self.read_edtf(literal, EdtfShape::Interval)
            .map(SimpleValue::SoftDateRange)
    }

    // ---- durations ----

    /// IEC 61131-3, hand-written: no established crate implements it.
    ///
    /// `T#` followed by one or more `<digits><unit>` groups, units strictly
    /// descending through d, h, m, s, ms, us, ns and each used at most once.
    /// No fractions, no sign, no `TIME#` long form.
    fn read_iec_duration(self, literal: &str) -> Result<SimpleValue, TypeError> {
        let malformed = |detail: &str| {
            self.err(
                SheniCode::MalformedDuration,
                literal,
                format!("expected an IEC 61131-3 duration such as `T#5s500ms`; {detail}"),
            )
        };
        let mut rest = literal
            .strip_prefix("T#")
            .ok_or_else(|| malformed("it must start with `T#`"))?;
        if rest.is_empty() {
            return Err(malformed("it needs at least one component"));
        }

        let mut nanos: u128 = 0;
        let mut last_rank: Option<usize> = None;
        while !rest.is_empty() {
            let digit_end = rest
                .bytes()
                .position(|b| !b.is_ascii_digit())
                .unwrap_or(rest.len());
            if digit_end == 0 {
                return Err(malformed("every component needs a number before its unit"));
            }
            let (digits, after_digits) = rest.split_at(digit_end);

            // Two-character units are tried first so `ms` is not read as `m`
            // followed by a stray `s`.
            let (rank, factor, unit_len) = IEC_UNITS
                .iter()
                .enumerate()
                .filter(|(_, (unit, _))| after_digits.starts_with(unit))
                .max_by_key(|(_, (unit, _))| unit.len())
                .map(|(rank, (unit, factor))| (rank, *factor, unit.len()))
                .ok_or_else(|| malformed("expected a unit of d, h, m, s, ms, us, or ns"))?;

            if last_rank.is_some_and(|last| rank <= last) {
                return Err(malformed(
                    "units must run from largest to smallest, each used at most once",
                ));
            }
            last_rank = Some(rank);

            let count: u128 = digits
                .parse()
                .map_err(|_| malformed("a component is too large"))?;
            nanos = count
                .checked_mul(factor)
                .and_then(|n| nanos.checked_add(n))
                .ok_or_else(|| malformed("the total is too large"))?;

            rest = &after_digits[unit_len..];
        }

        let secs = u64::try_from(nanos / 1_000_000_000)
            .map_err(|_| malformed("the total is too large"))?;
        let sub_nanos = (nanos % 1_000_000_000) as u32;
        Ok(SimpleValue::Duration(StdDuration::new(secs, sub_nanos)))
    }

    fn read_iso_duration(self, literal: &str) -> Result<SimpleValue, TypeError> {
        iso8601_duration::Duration::parse(literal)
            .map(SimpleValue::DurationIso)
            .map_err(|_| {
                self.err(
                    SheniCode::MalformedDuration,
                    literal,
                    "expected an ISO 8601 duration such as `P1DT2H30M`",
                )
            })
    }

    fn read_human_duration(self, literal: &str) -> Result<SimpleValue, TypeError> {
        humantime::parse_duration(literal)
            .map(SimpleValue::DurationHuman)
            .map_err(|e| self.err(SheniCode::MalformedDuration, literal, e.to_string()))
    }

    // ---- hand-written text shapes ----

    /// One path segment. Rejects separators and the two relative names, so a
    /// file name can never be a traversal. Nothing here consults a filesystem.
    fn read_file_name(self, literal: &str) -> Result<SimpleValue, TypeError> {
        let bad =
            |detail: &str| self.err(SheniCode::MalformedFileName, literal, detail.to_string());
        if literal.contains('/') || literal.contains('\\') {
            return Err(bad("a file name carries no path separator"));
        }
        if literal == "." || literal == ".." {
            return Err(bad("`.` and `..` name a directory position, not a file"));
        }
        if literal.chars().any(char::is_control) {
            return Err(bad("a file name carries no control characters"));
        }
        Ok(SimpleValue::FileName(literal.to_string()))
    }

    /// A path is a text shape and almost anything is one. Only control
    /// characters are rejected, since no filesystem accepts them and their
    /// presence is far more likely a truncated or spliced value.
    fn read_path(self, literal: &str) -> Result<SimpleValue, TypeError> {
        if literal.chars().any(char::is_control) {
            return Err(self.err(
                SheniCode::MalformedPath,
                literal,
                "a path carries no control characters",
            ));
        }
        Ok(SimpleValue::PathName(literal.to_string()))
    }

    /// `en` or `en.EN`, per the crate README. Not BCP 47; see [`LanguageTag`].
    fn read_language(self, literal: &str) -> Result<SimpleValue, TypeError> {
        let bad = || {
            self.err(
                SheniCode::MalformedLanguage,
                literal,
                "expected `en` or `en.EN` -- lowercase language, optional uppercase region",
            )
        };
        let (primary, region) = match literal.split_once('.') {
            Some((primary, region)) => (primary, Some(region)),
            None => (literal, None),
        };
        let primary_ok =
            (2..=3).contains(&primary.len()) && primary.bytes().all(|b| b.is_ascii_lowercase());
        let region_ok =
            region.is_none_or(|r| r.len() == 2 && r.bytes().all(|b| b.is_ascii_uppercase()));
        if !primary_ok || !region_ok {
            return Err(bad());
        }
        Ok(SimpleValue::Language(LanguageTag {
            primary: primary.to_string(),
            region: region.map(str::to_string),
        }))
    }

    fn err(self, code: SheniCode, literal: &str, message: impl Into<String>) -> TypeError {
        TypeError::new(code, self.name(), literal, message)
    }
}

impl fmt::Display for SimpleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<SimpleType> for String {
    fn from(value: SimpleType) -> Self {
        value.name().to_string()
    }
}

impl TryFrom<String> for SimpleType {
    type Error = TypeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        SimpleType::from_name(&value).ok_or_else(|| {
            TypeError::new(
                SheniCode::UnknownTypeName,
                "simple",
                value,
                "no simple type by that name",
            )
        })
    }
}

/// Which EDTF shape a type keeps. `soft_date` keeps the date, and
/// `soft_date_range` the interval; a value of the other shape is well formed
/// and declared wrongly, which is a different complaint from malformed text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EdtfShape {
    Date,
    Interval,
}

impl EdtfShape {
    const fn describe(self) -> &'static str {
        match self {
            EdtfShape::Date => "a single date",
            EdtfShape::Interval => "a range",
        }
    }

    const fn type_name(self) -> &'static str {
        match self {
            EdtfShape::Date => "soft_date",
            EdtfShape::Interval => "soft_date_range",
        }
    }
}

/// A value of a simple type: the thing the text denoted, not the text.
#[derive(Debug, Clone, PartialEq)]
pub enum SimpleValue {
    Uuid(Uuid),
    Date(Date),
    Time(Time),
    Timestamp(OffsetDateTime),
    /// An IEC 61131-3 duration. Fixed-length, so a plain [`StdDuration`].
    Duration(StdDuration),
    /// An ISO 8601 duration. Kept as the parsed calendar quantity rather than
    /// a [`StdDuration`], because a year and a month have no fixed length --
    /// `to_std` on this value returns `None` for exactly that reason.
    DurationIso(iso8601_duration::Duration),
    /// A humantime duration.
    DurationHuman(StdDuration),
    FileName(String),
    PathName(String),
    Email(EmailAddress),
    Url(Url),
    IpAddress(IpAddr),
    Language(LanguageTag),
    CurrencyCode(Currency),
    /// An EDTF date, restricted to the `Edtf::Date` shape by the reader.
    SoftDate(Edtf),
    /// An EDTF interval, restricted to the `Edtf::Interval` shape by the
    /// reader. The endpoints are reachable through [`SimpleValue::interval`].
    SoftDateRange(Edtf),
}

impl SimpleValue {
    /// The type this value was read at.
    pub fn type_of(&self) -> SimpleType {
        match self {
            SimpleValue::Uuid(_) => SimpleType::Uuid,
            SimpleValue::Date(_) => SimpleType::Date,
            SimpleValue::Time(_) => SimpleType::Time,
            SimpleValue::Timestamp(_) => SimpleType::Timestamp,
            SimpleValue::Duration(_) => SimpleType::Duration,
            SimpleValue::DurationIso(_) => SimpleType::DurationIso,
            SimpleValue::DurationHuman(_) => SimpleType::DurationHuman,
            SimpleValue::FileName(_) => SimpleType::FileName,
            SimpleValue::PathName(_) => SimpleType::PathName,
            SimpleValue::Email(_) => SimpleType::Email,
            SimpleValue::Url(_) => SimpleType::Url,
            SimpleValue::IpAddress(_) => SimpleType::IpAddress,
            SimpleValue::Language(_) => SimpleType::Language,
            SimpleValue::CurrencyCode(_) => SimpleType::CurrencyCode,
            SimpleValue::SoftDate(_) => SimpleType::SoftDate,
            SimpleValue::SoftDateRange(_) => SimpleType::SoftDateRange,
        }
    }

    /// The parsed EDTF expression behind a `soft_date` or `soft_date_range`,
    /// or `None` at any other type.
    pub fn edtf(&self) -> Option<&Edtf> {
        match self {
            SimpleValue::SoftDate(e) | SimpleValue::SoftDateRange(e) => Some(e),
            _ => None,
        }
    }

    /// How much of a `soft_date` is actually specified -- year, month,
    /// sub-year grouping, or complete day.
    ///
    /// This is the question the type exists to answer, and the reason
    /// imprecision is not modelled as absence: "August 2026" reports
    /// [`Precision::Month`] rather than arriving as nothing.
    pub fn precision(&self) -> Option<Precision> {
        match self {
            SimpleValue::SoftDate(Edtf::Date(d)) => Some(d.precision()),
            _ => None,
        }
    }

    /// The two endpoints of a `soft_date_range`, each of which may be a date,
    /// open (`..`), or unknown (empty).
    pub fn interval(&self) -> Option<&Interval> {
        match self {
            SimpleValue::SoftDateRange(Edtf::Interval(i)) => Some(i),
            _ => None,
        }
    }
}

/// Render an IEC 61131-3 duration in the canonical form: components largest
/// to smallest, zeroes omitted, and `T#0s` when there is nothing to write.
fn write_iec_duration(f: &mut fmt::Formatter<'_>, duration: StdDuration) -> fmt::Result {
    let mut nanos =
        u128::from(duration.as_secs()) * 1_000_000_000 + u128::from(duration.subsec_nanos());
    f.write_str("T#")?;
    let mut wrote_any = false;
    for (unit, factor) in IEC_UNITS {
        let count = nanos / factor;
        if count > 0 {
            write!(f, "{count}{unit}")?;
            nanos -= count * factor;
            wrote_any = true;
        }
    }
    if !wrote_any {
        f.write_str("0s")?;
    }
    Ok(())
}

/// Render an ISO 8601 duration. Whole quantities lose their fractional part
/// so `P1D` does not come back as `P1.0D`.
fn write_iso_duration(f: &mut fmt::Formatter<'_>, d: &iso8601_duration::Duration) -> fmt::Result {
    fn part(f: &mut fmt::Formatter<'_>, value: f32, unit: char) -> fmt::Result {
        if value == 0.0 {
            return Ok(());
        }
        if value.fract() == 0.0 {
            write!(f, "{}{unit}", value as i64)
        } else {
            write!(f, "{value}{unit}")
        }
    }
    f.write_str("P")?;
    part(f, d.year, 'Y')?;
    part(f, d.month, 'M')?;
    part(f, d.day, 'D')?;
    let has_time = d.hour != 0.0 || d.minute != 0.0 || d.second != 0.0;
    let has_date = d.year != 0.0 || d.month != 0.0 || d.day != 0.0;
    if has_time {
        f.write_str("T")?;
        part(f, d.hour, 'H')?;
        part(f, d.minute, 'M')?;
        part(f, d.second, 'S')?;
    } else if !has_date {
        // Every component is zero. `P` alone is not a duration, so name the
        // zero explicitly.
        f.write_str("T0S")?;
    }
    Ok(())
}

/// The canonical text form. Reading it back at the same type yields an equal
/// value; it is not always the text that was read in.
impl fmt::Display for SimpleValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimpleValue::Uuid(u) => write!(f, "{}", u.hyphenated()),
            SimpleValue::Date(d) => f.write_str(
                &d.format(format_description!("[year]-[month]-[day]"))
                    .map_err(|_| fmt::Error)?,
            ),
            SimpleValue::Time(t) => f.write_str(
                &t.format(format_description!("[hour]:[minute]:[second]"))
                    .map_err(|_| fmt::Error)?,
            ),
            SimpleValue::Timestamp(ts) => {
                f.write_str(&ts.format(&Rfc3339).map_err(|_| fmt::Error)?)
            }
            SimpleValue::Duration(d) => write_iec_duration(f, *d),
            SimpleValue::DurationIso(d) => write_iso_duration(f, d),
            SimpleValue::DurationHuman(d) => write!(f, "{}", humantime::format_duration(*d)),
            SimpleValue::FileName(s) | SimpleValue::PathName(s) => f.write_str(s),
            SimpleValue::Email(e) => write!(f, "{e}"),
            SimpleValue::Url(u) => write!(f, "{u}"),
            SimpleValue::IpAddress(ip) => write!(f, "{ip}"),
            SimpleValue::Language(l) => write!(f, "{l}"),
            SimpleValue::CurrencyCode(c) => f.write_str(c.code()),
            SimpleValue::SoftDate(e) | SimpleValue::SoftDateRange(e) => write!(f, "{e}"),
        }
    }
}

/// Serialised as its canonical text, which is the form a SYON document holds.
///
/// There is no matching `Deserialize`: the text alone does not say which
/// simple type it is, and that comes from the schema.
impl Serialize for SimpleValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

#[cfg(test)]
mod tests {

    // ---- soft_date and soft_date_range (ADR sheni_06, ADR sheni_10) ----

    #[test]
    fn soft_date_takes_every_precision_the_standard_defines() {
        use edtf_core::Precision;
        for (literal, precision) in [
            ("2026-08-12", Precision::Day),
            ("2026-08", Precision::Month),
            ("2026", Precision::Year),
            ("2026-35", Precision::Season),
            ("XXXX", Precision::Year),
        ] {
            let value = SimpleType::SoftDate.read(literal).unwrap();
            assert_eq!(value.precision(), Some(precision), "{literal}");
            assert_eq!(value.to_string(), literal);
        }
    }

    /// The quarters are sub-year grouping codes 33-36 in the month slot, so
    /// "Q3 2026" needs nothing invented for it. This is the case that caught
    /// ADR sheni_06's first draft, which capped acceptance at level 1.
    #[test]
    fn a_quarter_is_a_soft_date() {
        for (code, quarter) in [(33, "Q1"), (34, "Q2"), (35, "Q3"), (36, "Q4")] {
            let literal = format!("2026-{code}");
            let value = SimpleType::SoftDate.read(&literal).unwrap();
            assert_eq!(
                value.precision(),
                Some(edtf_core::Precision::Season),
                "{quarter}"
            );
        }
    }

    #[test]
    fn soft_date_takes_the_uncertainty_qualifiers() {
        for literal in ["2026-08-12?", "2026~", "2026-08-12%"] {
            assert!(SimpleType::SoftDate.read(literal).is_ok(), "{literal}");
        }
        // The canonical form gathers a per-component qualifier onto the whole.
        assert_eq!(
            SimpleType::SoftDate
                .read("?2026-?08-?12")
                .unwrap()
                .to_string(),
            "2026-08-12?"
        );
    }

    /// A set is written with `[` or `{`. The Go implementation rejects both
    /// outright and the Rust one carries them through as a scalar, so the
    /// value is unwritable rather than merely unsupported -- and it says so
    /// rather than reporting malformed text.
    #[test]
    fn an_edtf_set_is_unwritable_not_merely_unsupported() {
        for literal in ["[1667,1668]", "{1667,1668}"] {
            let err = SimpleType::SoftDate.read(literal).unwrap_err();
            assert_eq!(err.code(), SheniCode::EdtfSetNotWritable, "{literal}");
        }
    }

    /// Right family, wrong member. The message names the type wanted, which
    /// is the whole reason this is not reported as malformed.
    #[test]
    fn a_valid_edtf_of_the_wrong_shape_names_the_type_wanted() {
        let err = SimpleType::SoftDate.read("2026-08/2026-10").unwrap_err();
        assert_eq!(err.code(), SheniCode::EdtfShapeMismatch);
        assert!(
            err.message().contains("soft_date_range"),
            "{}",
            err.message()
        );

        let err = SimpleType::SoftDateRange.read("2026-08").unwrap_err();
        assert_eq!(err.code(), SheniCode::EdtfShapeMismatch);
        assert!(err.message().contains("soft_date"), "{}", err.message());

        // Sheni already has `timestamp`, so EDTF's date-with-time is refused
        // at both, pointing at the type that owns it.
        for t in [SimpleType::SoftDate, SimpleType::SoftDateRange] {
            let err = t.read("2026-08-12T14:30:00").unwrap_err();
            assert_eq!(err.code(), SheniCode::EdtfShapeMismatch);
            assert!(err.message().contains("timestamp"), "{}", err.message());
        }
    }

    #[test]
    fn text_that_is_not_edtf_at_all_is_malformed() {
        for literal in ["not a date", "2026-13-01", "1985-02-30"] {
            assert_eq!(
                SimpleType::SoftDate.read(literal).unwrap_err().code(),
                SheniCode::MalformedEdtf,
                "{literal}"
            );
        }
    }

    /// `date` is deliberately unchanged: a field declared `date` promises the
    /// day is known.
    #[test]
    fn date_did_not_widen_to_accept_a_soft_one() {
        assert_eq!(
            SimpleType::Date.read("2026-08").unwrap_err().code(),
            SheniCode::MalformedDate
        );
        assert!(SimpleType::SoftDate.read("2026-08").is_ok());
    }

    #[test]
    fn a_range_keeps_open_and_unknown_apart() {
        use edtf_core::IntervalEndpoint;
        let open = SimpleType::SoftDateRange.read("2026-08/..").unwrap();
        let unknown = SimpleType::SoftDateRange.read("2026-08/").unwrap();

        assert!(matches!(
            open.interval().unwrap().end,
            IntervalEndpoint::Open
        ));
        assert!(matches!(
            unknown.interval().unwrap().end,
            IntervalEndpoint::Unknown
        ));
        assert_ne!(open, unknown);
    }

    #[test]
    fn a_range_takes_a_coarse_endpoint_at_either_end() {
        for literal in [
            "2026-08/2026-10",
            "2026/2027",
            "2026-35/2026-36",
            "../2026-10",
        ] {
            let value = SimpleType::SoftDateRange.read(literal).unwrap();
            assert_eq!(value.to_string(), literal, "{literal}");
        }
    }

    /// The two EDTF types are the only simple types with a fallback, and both
    /// fallbacks are members of the value space rather than legal values
    /// borrowed to stand in for nothing.
    #[test]
    fn only_the_edtf_types_have_a_fallback() {
        for t in SimpleType::ALL {
            let expected = matches!(t, SimpleType::SoftDate | SimpleType::SoftDateRange);
            assert_eq!(t.fallback().is_some(), expected, "{t}");
        }
        assert_eq!(SimpleType::SoftDate.fallback().unwrap().to_string(), "XXXX");
        assert_eq!(
            SimpleType::SoftDateRange.fallback().unwrap().to_string(),
            "XXXX/XXXX"
        );
    }

    /// EDTF refuses `/` and `../..` -- an interval needs at least one dated
    /// endpoint -- which is why the fallback is `XXXX/XXXX` and not something
    /// this crate picked.
    #[test]
    fn the_standard_refuses_the_range_with_no_dated_endpoint() {
        for literal in ["/", "../.."] {
            assert_eq!(
                SimpleType::SoftDateRange.read(literal).unwrap_err().code(),
                SheniCode::MalformedEdtf,
                "{literal}"
            );
        }
    }

    use super::*;

    fn code(t: SimpleType, literal: &str) -> SheniCode {
        t.read(literal)
            .expect_err("expected this literal to be rejected")
            .code()
    }

    /// One canonical literal per type, used by the round-trip tests.
    const CANONICAL: [(SimpleType, &str); 16] = [
        (SimpleType::Uuid, "018f5e2a-0000-7000-8000-000000000000"),
        (SimpleType::Date, "2026-08-28"),
        (SimpleType::Time, "14:30:00"),
        (SimpleType::Timestamp, "2026-08-28T14:30:00Z"),
        (SimpleType::Duration, "T#5s500ms"),
        (SimpleType::DurationIso, "P1DT2H30M"),
        (SimpleType::DurationHuman, "1h 30m"),
        (SimpleType::FileName, "README.md"),
        (SimpleType::PathName, "/path/to/file"),
        (SimpleType::Email, "user@example.com"),
        (SimpleType::Url, "https://example.com/path?query=1"),
        (SimpleType::IpAddress, "192.168.1.1"),
        (SimpleType::Language, "en.EN"),
        (SimpleType::CurrencyCode, "USD"),
        (SimpleType::SoftDate, "2026-08"),
        (SimpleType::SoftDateRange, "2026-08/2026-10"),
    ];

    #[test]
    fn every_type_names_itself_and_reads_back() {
        for t in SimpleType::ALL {
            assert_eq!(SimpleType::from_name(t.name()), Some(t), "{t}");
            assert_eq!(t.group(), TypeGroup::Simple);
            assert_eq!(t.carrier(), PrimitiveType::String);
        }
    }

    #[test]
    fn canonical_literals_cover_every_type() {
        for t in SimpleType::ALL {
            assert!(
                CANONICAL.iter().any(|(ct, _)| *ct == t),
                "{t} has no canonical literal in the test table"
            );
        }
    }

    #[test]
    fn simple_and_primitive_names_do_not_collide() {
        for t in SimpleType::ALL {
            assert_eq!(PrimitiveType::from_name(t.name()), None, "{t}");
        }
    }

    #[test]
    fn every_type_rejects_empty_text() {
        for t in SimpleType::ALL {
            assert_eq!(code(t, ""), SheniCode::SimpleEmptyValue, "{t}");
        }
    }

    /// One rejected literal per type. Several are another type's canonical
    /// form, which is the cheapest way to show the types do not overlap.
    const REJECTED: [(SimpleType, &str); 16] = [
        (SimpleType::Uuid, "not-a-uuid"),
        (SimpleType::Date, "2026-8-28"),
        (SimpleType::Time, "T14:30:00"),
        (SimpleType::Timestamp, "2026-08-28T14:30:00"),
        (SimpleType::Duration, "5s"),
        (SimpleType::DurationIso, "1h30m"),
        (SimpleType::DurationHuman, "P1D"),
        (SimpleType::FileName, "a/b"),
        (SimpleType::PathName, "/path/with\u{0}nul"),
        (SimpleType::Email, "user@"),
        (SimpleType::Url, "example.com"),
        (SimpleType::IpAddress, "1.2.3"),
        (SimpleType::Language, "en-GB"),
        (SimpleType::CurrencyCode, "usd"),
        (SimpleType::SoftDate, "2026-08/2026-10"),
        (SimpleType::SoftDateRange, "2026-08"),
    ];

    #[test]
    fn every_error_sits_in_the_simple_band_and_names_its_type() {
        for (t, literal) in REJECTED {
            let err = t
                .read(literal)
                .expect_err("this literal should be rejected");
            assert_eq!(err.code().group(), Some(TypeGroup::Simple), "{t}");
            assert_eq!(err.type_name(), t.name(), "{t}");
            assert_eq!(err.literal(), literal, "{t}");
        }
    }

    #[test]
    fn rejected_literals_cover_every_type() {
        for t in SimpleType::ALL {
            assert!(
                REJECTED.iter().any(|(rt, _)| *rt == t),
                "{t} has no rejected literal in the test table"
            );
        }
    }

    // ---- delegated types ----

    #[test]
    fn uuid_takes_the_canonical_form_in_either_case() {
        let lower = "018f5e2a-0000-7000-8000-000000000000";
        let upper = "018F5E2A-0000-7000-8000-000000000000";
        assert_eq!(SimpleType::Uuid.read(lower), SimpleType::Uuid.read(upper));
        assert_eq!(SimpleType::Uuid.read(lower).unwrap().to_string(), lower);
    }

    /// The `uuid` crate accepts all four of these; ADR sheni-0002 accepts one.
    #[test]
    fn uuid_rejects_the_other_spellings_the_crate_would_take() {
        for other in [
            "018f5e2a000070008000000000000000",
            "urn:uuid:018f5e2a-0000-7000-8000-000000000000",
            "{018f5e2a-0000-7000-8000-000000000000}",
        ] {
            assert_eq!(code(SimpleType::Uuid, other), SheniCode::NonCanonicalUuid);
        }
        assert_eq!(
            code(SimpleType::Uuid, "not-a-uuid"),
            SheniCode::MalformedUuid
        );
    }

    #[test]
    fn date_takes_iso_8601_only() {
        assert!(SimpleType::Date.read("2026-08-28").is_ok());
        for bad in [
            "2026-8-28",
            "20260828",
            "2026-02-30",
            " 2026-08-28",
            "28-08-2026",
        ] {
            assert_eq!(
                code(SimpleType::Date, bad),
                SheniCode::MalformedDate,
                "{bad}"
            );
        }
    }

    /// `time` reads this as an expanded year. ADR sheni-0001's rule wins.
    #[test]
    fn date_rejects_a_leading_plus_the_delegate_would_accept() {
        assert!(Date::parse("+2026-08-28", format_description!("[year]-[month]-[day]")).is_ok());
        assert_eq!(
            code(SimpleType::Date, "+2026-08-28"),
            SheniCode::SimpleLeadingPlus
        );
    }

    /// The README's outline writes `THH:MM:SS`; its standards table writes
    /// `14:30:00`. ADR sheni-0002 takes the table.
    #[test]
    fn time_takes_the_standards_table_form_without_a_designator() {
        assert!(SimpleType::Time.read("14:30:00").is_ok());
        for bad in ["T14:30:00", "14:30", "4:30:00", "24:00:00"] {
            assert_eq!(
                code(SimpleType::Time, bad),
                SheniCode::MalformedTime,
                "{bad}"
            );
        }
    }

    #[test]
    fn timestamp_takes_rfc_3339() {
        for good in [
            "2026-08-28T14:30:00Z",
            "2026-08-28T14:30:00+02:00",
            "2026-08-28T14:30:00.500Z",
        ] {
            assert!(SimpleType::Timestamp.read(good).is_ok(), "{good}");
        }
        // No offset: RFC 3339 requires one.
        assert_eq!(
            code(SimpleType::Timestamp, "2026-08-28T14:30:00"),
            SheniCode::MalformedTimestamp
        );
    }

    #[test]
    fn email_defers_to_rfc_5322() {
        assert!(SimpleType::Email.read("user@example.com").is_ok());
        for bad in ["user@", "@example.com", "user name@example.com", "plain"] {
            assert_eq!(
                code(SimpleType::Email, bad),
                SheniCode::MalformedEmail,
                "{bad}"
            );
        }
    }

    #[test]
    fn url_must_be_absolute_and_says_so_specifically() {
        assert!(SimpleType::Url.read("https://example.com/").is_ok());
        assert!(SimpleType::Url.read("https://example.com:8443/x").is_ok());
        for relative in ["example.com", "/relative", "../up"] {
            assert_eq!(
                code(SimpleType::Url, relative),
                SheniCode::RelativeUrl,
                "{relative}"
            );
        }
        assert_eq!(
            code(SimpleType::Url, "http://[bad"),
            SheniCode::MalformedUrl
        );
    }

    #[test]
    fn ip_address_takes_both_families() {
        assert!(SimpleType::IpAddress.read("192.168.1.1").is_ok());
        assert!(SimpleType::IpAddress
            .read("2001:db8::8a2e:370:7334")
            .is_ok());
        assert!(SimpleType::IpAddress.read("::1").is_ok());
        // Leading zeros in a dotted quad are octal in some resolvers.
        for bad in ["192.168.001.1", "1.2.3", "not-an-ip"] {
            assert_eq!(
                code(SimpleType::IpAddress, bad),
                SheniCode::MalformedIpAddress,
                "{bad}"
            );
        }
    }

    #[test]
    fn currency_code_takes_iso_4217_uppercase() {
        for good in ["USD", "EUR", "GBP", "JPY"] {
            assert!(SimpleType::CurrencyCode.read(good).is_ok(), "{good}");
        }
        for bad in ["usd", "ZZZ", "US", "DOLLAR"] {
            assert_eq!(
                code(SimpleType::CurrencyCode, bad),
                SheniCode::UnknownCurrencyCode,
                "{bad}"
            );
        }
    }

    // ---- normalisation, the difference from the primitive layer ----

    /// Each of these is the same value written a different way. The primitive
    /// layer would keep the text; this layer keeps the meaning.
    #[test]
    fn reading_normalises_where_the_standard_allows_two_spellings() {
        let cases = [
            (SimpleType::IpAddress, "2001:0DB8::1", "2001:db8::1"),
            (
                SimpleType::Url,
                "https://example.com",
                "https://example.com/",
            ),
            (
                SimpleType::Url,
                "HTTPS://Example.COM/A",
                "https://example.com/A",
            ),
            (SimpleType::DurationHuman, "90m", "1h 30m"),
            (
                SimpleType::Timestamp,
                "2026-08-28t14:30:00z",
                "2026-08-28T14:30:00Z",
            ),
            (
                SimpleType::Uuid,
                "018F5E2A-0000-7000-8000-000000000000",
                "018f5e2a-0000-7000-8000-000000000000",
            ),
        ];
        for (t, written, canonical) in cases {
            let value = t.read(written).unwrap_or_else(|e| panic!("{t}: {e}"));
            assert_eq!(value.to_string(), canonical, "{t} {written:?}");
        }
    }

    /// Normalising moves the spelling, never the meaning: the canonical form
    /// reads back as an equal value.
    #[test]
    fn the_canonical_form_reads_back_as_an_equal_value() {
        for (t, literal) in CANONICAL {
            let value = t.read(literal).unwrap_or_else(|e| panic!("{t}: {e}"));
            assert_eq!(value.to_string(), literal, "{t} is not already canonical");
            assert_eq!(t.read(&value.to_string()), Ok(value), "{t}");
        }
    }

    // ---- IEC 61131-3 durations, the hand-written one ----

    #[test]
    fn iec_duration_reads_each_unit() {
        let cases = [
            ("T#1d", 86_400),
            ("T#2h", 7_200),
            ("T#30m", 1_800),
            ("T#5s", 5),
            ("T#1d2h30m", 95_400),
        ];
        for (literal, secs) in cases {
            assert_eq!(
                SimpleType::Duration.read(literal),
                Ok(SimpleValue::Duration(StdDuration::from_secs(secs))),
                "{literal}"
            );
        }
    }

    #[test]
    fn iec_duration_reads_sub_second_units() {
        assert_eq!(
            SimpleType::Duration.read("T#500ms"),
            Ok(SimpleValue::Duration(StdDuration::from_millis(500)))
        );
        assert_eq!(
            SimpleType::Duration.read("T#5us"),
            Ok(SimpleValue::Duration(StdDuration::from_micros(5)))
        );
        assert_eq!(
            SimpleType::Duration.read("T#5ns"),
            Ok(SimpleValue::Duration(StdDuration::from_nanos(5)))
        );
    }

    /// The reason two-character units are matched first.
    #[test]
    fn ms_is_milliseconds_not_minutes_followed_by_seconds() {
        assert_ne!(
            SimpleType::Duration.read("T#5ms"),
            SimpleType::Duration.read("T#5m")
        );
        assert_eq!(
            SimpleType::Duration.read("T#5ms"),
            Ok(SimpleValue::Duration(StdDuration::from_millis(5)))
        );
        // A minute component still reads as one when a smaller unit follows.
        assert_eq!(
            SimpleType::Duration.read("T#5m30s"),
            Ok(SimpleValue::Duration(StdDuration::from_secs(330)))
        );
    }

    #[test]
    fn iec_duration_requires_descending_units_used_once() {
        for bad in ["T#30m1d", "T#5s5s", "T#500ms1s", "T#1m1m"] {
            assert_eq!(
                code(SimpleType::Duration, bad),
                SheniCode::MalformedDuration,
                "{bad}"
            );
        }
    }

    #[test]
    fn iec_duration_rejects_the_forms_outside_the_closed_set() {
        for bad in [
            "T#",      // no components
            "T#5",     // no unit
            "T#s",     // no number
            "5s",      // no prefix
            "t#5s",    // lowercase prefix
            "TIME#5s", // long prefix
            "T#1.5s",  // fractional
            "T#-5s",   // signed
            "T#5y",    // unknown unit
            "T# 5s",   // internal space
        ] {
            assert_eq!(
                code(SimpleType::Duration, bad),
                SheniCode::MalformedDuration,
                "{bad}"
            );
        }
    }

    #[test]
    fn iec_duration_reports_overflow_rather_than_wrapping() {
        assert_eq!(
            code(SimpleType::Duration, "T#99999999999999999999999999d"),
            SheniCode::MalformedDuration
        );
    }

    #[test]
    fn iec_duration_renders_largest_to_smallest_omitting_zeroes() {
        let cases = [
            (StdDuration::from_secs(95_400), "T#1d2h30m"),
            (StdDuration::from_millis(5_500), "T#5s500ms"),
            (StdDuration::ZERO, "T#0s"),
            (StdDuration::from_nanos(1), "T#1ns"),
        ];
        for (duration, expected) in cases {
            assert_eq!(SimpleValue::Duration(duration).to_string(), expected);
        }
    }

    #[test]
    fn iec_duration_round_trips_through_its_canonical_form() {
        for literal in ["T#1d", "T#5s500ms", "T#1d2h30m45s", "T#0s", "T#1ns"] {
            let value = SimpleType::Duration.read(literal).unwrap();
            assert_eq!(value.to_string(), literal, "{literal}");
        }
    }

    // ---- the other two durations ----

    #[test]
    fn iso_duration_takes_calendar_quantities_a_std_duration_cannot_hold() {
        let value = SimpleType::DurationIso.read("P1Y").unwrap();
        assert_eq!(value.to_string(), "P1Y");
        // A year has no fixed length, which is why the value is not converted.
        let SimpleValue::DurationIso(d) = value else {
            panic!("wrong variant")
        };
        assert_eq!(d.to_std(), None);
    }

    #[test]
    fn iso_duration_round_trips_and_rejects_the_other_conventions() {
        for literal in ["P1DT2H30M", "PT1H", "P1Y", "PT1.5S", "PT0S"] {
            let value = SimpleType::DurationIso.read(literal).unwrap();
            assert_eq!(value.to_string(), literal, "{literal}");
        }
        for bad in ["1D", "P", "p1dt2h30m", "T#5s", "1h30m"] {
            assert_eq!(
                code(SimpleType::DurationIso, bad),
                SheniCode::MalformedDuration,
                "{bad}"
            );
        }
    }

    #[test]
    fn human_duration_rejects_the_other_conventions() {
        assert!(SimpleType::DurationHuman.read("1h30m").is_ok());
        assert!(SimpleType::DurationHuman.read("500ms").is_ok());
        for bad in ["P1DT2H30M", "T#5s", "5", "-5s"] {
            assert_eq!(
                code(SimpleType::DurationHuman, bad),
                SheniCode::MalformedDuration,
                "{bad}"
            );
        }
    }

    /// Three conventions, three types. `PT1M` and `1m` mean the same minute,
    /// but `1m` at `duration_iso` is not a duration at all -- which is the
    /// point of not sniffing.
    #[test]
    fn the_three_duration_types_do_not_accept_each_others_text() {
        let literals = [
            (SimpleType::Duration, "T#1m"),
            (SimpleType::DurationIso, "PT1M"),
            (SimpleType::DurationHuman, "1m"),
        ];
        for (owner, literal) in literals {
            for (other, _) in literals {
                if other != owner {
                    assert!(other.read(literal).is_err(), "{other} accepted {literal:?}");
                }
            }
        }
    }

    // ---- hand-written text shapes ----

    #[test]
    fn file_name_is_one_segment() {
        assert!(SimpleType::FileName.read("README.md").is_ok());
        assert!(SimpleType::FileName.read(".gitignore").is_ok());
        for bad in ["a/b", "a\\b", ".", "..", "/etc/passwd", "../secret"] {
            assert_eq!(
                code(SimpleType::FileName, bad),
                SheniCode::MalformedFileName,
                "{bad}"
            );
        }
    }

    #[test]
    fn path_takes_a_text_shape_without_touching_a_filesystem() {
        for good in [
            "/path/to/file",
            "relative/path",
            "C:\\Windows",
            "/does/not/exist",
        ] {
            assert!(SimpleType::PathName.read(good).is_ok(), "{good}");
        }
        assert_eq!(
            code(SimpleType::PathName, "/path/with\0nul"),
            SheniCode::MalformedPath
        );
        assert_eq!(
            code(SimpleType::PathName, "/path/with\nnewline"),
            SheniCode::MalformedPath
        );
    }

    #[test]
    fn language_follows_the_readme_form_not_bcp_47() {
        for good in ["en", "de", "eng", "en.EN", "de.DE"] {
            assert!(SimpleType::Language.read(good).is_ok(), "{good}");
        }
        // BCP 47's own spelling is rejected. ADR sheni-0002 flags this.
        for bad in ["en-GB", "EN", "e", "engl", "en.gb", "en.GBR", "en."] {
            assert_eq!(
                code(SimpleType::Language, bad),
                SheniCode::MalformedLanguage,
                "{bad}"
            );
        }
    }

    #[test]
    fn language_exposes_its_parts() {
        let SimpleValue::Language(tag) = SimpleType::Language.read("en.EN").unwrap() else {
            panic!("wrong variant")
        };
        assert_eq!(tag.primary(), "en");
        assert_eq!(tag.region(), Some("EN"));

        let SimpleValue::Language(tag) = SimpleType::Language.read("de").unwrap() else {
            panic!("wrong variant")
        };
        assert_eq!(tag.region(), None);
    }

    // ---- values, nodes, and serialisation ----

    #[test]
    fn a_value_reports_the_type_it_was_read_at() {
        for (t, literal) in CANONICAL {
            let value = t.read(literal).unwrap_or_else(|e| panic!("{t}: {e}"));
            assert_eq!(value.type_of(), t);
        }
    }

    #[test]
    fn a_scalar_node_reads_at_its_declared_type() {
        let node = Value::Scalar("2026-08-28".to_string());
        assert_eq!(
            SimpleType::Date.read_value(&node),
            SimpleType::Date.read("2026-08-28")
        );
    }

    #[test]
    fn a_mapping_or_sequence_is_not_a_single_value() {
        for node in [Value::Mapping(vec![]), Value::Sequence(vec![])] {
            assert_eq!(
                SimpleType::Date.read_value(&node).unwrap_err().code(),
                SheniCode::NotAScalar
            );
        }
    }

    #[test]
    fn a_type_serialises_as_its_name() {
        let json = serde_json::to_string(&SimpleType::CurrencyCode).unwrap();
        assert_eq!(json, "\"currency_code\"");
        assert_eq!(
            serde_json::from_str::<SimpleType>(&json).unwrap(),
            SimpleType::CurrencyCode
        );
        assert!(serde_json::from_str::<SimpleType>("\"date_time\"").is_err());
    }

    #[test]
    fn a_value_serialises_as_its_canonical_text() {
        for (t, literal) in CANONICAL {
            let value = t.read(literal).unwrap();
            assert_eq!(
                serde_json::to_string(&value).unwrap(),
                serde_json::to_string(literal).unwrap(),
                "{t}"
            );
        }
    }
}
