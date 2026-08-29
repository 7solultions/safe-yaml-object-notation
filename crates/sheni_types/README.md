# Sheni (2.Layer): Types 

SYON parses; it does not interpret. A successful parse hands back
`Value::Scalar(String)` — the text between the delimiters, unchanged and
untyped. Sheni is the layer that says what that text means.

## Status

| Group | State |
|-------|-------|
| primitive | implemented — [ADR sheni_01](../../design/architecture/ADR_sheni_01__primitives.syon) |
| soft primitive | implemented — [ADR sheni_08](../../design/architecture/ADR_sheni_08__soft_types.syon), [ADR sheni_09](../../design/architecture/ADR_sheni_09__soft_primitives.syon) |
| simple | implemented — [ADR sheni_02](../../design/architecture/ADR_sheni_02__simple_types.syon) |
| soft date | implemented — [ADR sheni_06](../../design/architecture/ADR_sheni_06__soft_dates.syon), [ADR sheni_10](../../design/architecture/ADR_sheni_10__soft_date_ranges.syon) |
| core complex | designed, not started — [ADR sheni_03](../../design/architecture/ADR_sheni_03__complex_types.syon) |
| collection | designed, not started — [ADR sheni_04](../../design/architecture/ADR_sheni_04__collections.syon) |

User-declared types are **not** a fifth group. A struct an application
declares is still a struct, so provenance is a different axis from kind, and
it lives one layer up in
[`shelishi_schema`](../shelishi_schema/README.md) — see
[ADR sheni_05](../../design/architecture/ADR_sheni_05__type_layer_boundary.syon).

The group itself is `TypeGroup`; every type descriptor reports which group it
belongs to, and each group has its decisions recorded in an ADR — see the
Status table below.

## Groups

1. primitives 
2. simple 
3. core complex (complex structs or enums for everyday usage like, address or day of the week, month or a year)
4. collections 
(User-declared types are layer 3, not a fifth group — see `shelishi_schema`.)

## Primitives 

1. **Primitives** (`primitives.rs`)
    a. `boolean` : `false` / `true`, `No` / `Yes` (maybe later: off / on)
    b. number: (bit width / bytes)
        1. `unsigned integer`
        2. `signed integer` 
        3. `float` 
    c. character 
        1. byte based character [0..255]
        2. unicode 
    d. string (more on simple types)

2. **Simple** (with interpretation, `simple.rs`)
    a. `id`/`UUID`: based on `integer` or `string` (e.g. UUID v7)
    b. `date`: `YYYY-MM-DD`
    c. `time`: `THH:MM:SS`
    d. `duration`: `T#{{value}}{{unit}}`
    e. `file_name`: `README.md`
    f. `path`: `/path/to/file`
    g. `email`: `email@example.com`
    h. `url`: `https://example.com:port`
    i. `ip_address`: IPv4 or IPv6
    j. `language`: `en` or `en.EN`
    k. `currency_code`: `USD`, `EUR`, `GBP`, `JPY`

3. **Complex** (core/complex `complex.rs`)
    a. `Enum`: set of values connected to a value (e.g. integer, string)
    b. `Struct`: composition of named and typed elements. 

4. **Collection** (`collection.rs`)
    a. `list`: position (integer), value (...)
        typically `[...,...]`
        syon: `- element\n`
    b. `map`: key (string), value (...) 
        typically `{...:...,...:...}`
        syon: `key: value\n`

## Implemented: primitives

Sixteen types, spelled as Rust spells them. A number's width is part of its
type; there is no width-less `int` or `float`.

| Type | Accepts |
|------|---------|
|`bool`|`true`, `false`, `yes`, `no`, case-insensitively. `on` / `off` are reserved and rejected.|
|`u8` `u16` `u32` `u64` `u128`|Decimal digits. No sign, no separators, no leading zeros. Out-of-range is an error, not a wrap.|
|`i8` `i16` `i32` `i64` `i128`|As above, with an optional leading `-`.|
|`f32` `f64`|Finite decimal, optional exponent. `NaN` and the infinities are rejected.|
|`byte`|One character whose code point fits in `0..=255`.|
|`char`|Exactly one Unicode scalar value.|
|`string`|Any well-formed UTF-8 text, verbatim — including text that would also read as a boolean or a number.|

Reading is fallible and says why: a `TypeError` carries a `SheniCode`, banded
by group (1-99 general, 101-199 primitives, 201-299 simple, 301-399 complex,
401-499 collections; 501-599 is reserved for the schema layer above), the same discipline ADR 0008
set for parse errors. A soft primitive reports its strict twin's codes
unchanged, with only the type name on the error saying which twin was
declared.

```rust
use sheni_types::{PrimitiveType, PrimitiveValue, SheniCode};

let t = PrimitiveType::from_name("u8").unwrap();
assert_eq!(t.read("300").unwrap_err().code(), SheniCode::IntegerOutOfRange);

// `no` is a boolean only where a boolean was declared.
assert_eq!(PrimitiveType::Boolean.read("no"), Ok(PrimitiveValue::Boolean(false)));
assert_eq!(PrimitiveType::String.read("no"), Ok(PrimitiveValue::String("no".into())));
```

`read_value` takes a parsed SYON node instead of raw text, so the layer sits
directly on the parser's output.

## Implemented: soft primitives

Fifteen more, one per primitive except `string`. A soft type accepts exactly
what its strict twin accepts, plus the word `unknown`, case-insensitively.

The reason they exist is that a fallback has to be honest. ADR sheni_03 lets a
field be optional only where its type has a fallback, and gave the numerics a
zero. Zero is an answer, though — a count of zero items is a fact somebody
established, not a sign that nobody looked. A soft type's fallback is the
unknown, which is a member of the value space rather than a legal value
borrowed to stand in for one.

```rust
use sheni_types::{SheniCode, SoftPrimitiveType};

let count = SoftPrimitiveType::from_name("soft_u32").unwrap();
assert_eq!(count.fallback().to_string(), "unknown");
assert_ne!(count.fallback(), count.read("0").unwrap());

// Softness is not leniency. Every check the strict twin makes is made here.
assert_eq!(count.read("007").unwrap_err().code(), SheniCode::LeadingZero);
```

**There is no `soft_string`**, and the exclusion is discovered rather than
chosen. `string` accepts any well-formed UTF-8 verbatim, so no text lies
outside its value space, so no word can mean "not known" without also being a
string somebody meant literally — which is the definition of a sentinel. The
empty string is a genuine identity in a way zero is not, so `string` did not
need a soft twin anyway.

**`soft_bool` carries Kleene's three-valued logic.** `false` ranks 0, `unknown`
1, `true` 2, so `and` is the minimum, `or` the maximum, and `not` the
complement against 2 — which leaves `unknown` where it is. The operations ship
with the type because truth tables are where three-valued logic is reliably got
wrong.

```rust
use sheni_types::SoftPrimitiveType;

let t = SoftPrimitiveType::from_name("soft_bool").unwrap();
let (no, dunno) = (t.read("false").unwrap(), t.read("unknown").unwrap());
assert_eq!(no.and(&dunno).unwrap(), no);   // false and unknown is false
assert_eq!(dunno.not().unwrap(), dunno);   // not unknown is unknown
```

Writing `unknown` at a strict type is its own error, `SHENI-115`, rather than a
malformed literal: the author is reaching for a type that exists under another
name, and the message says which. At `string`, `unknown` is text and always
was.

## Implemented: simple types

Sixteen types, each an interpretation over the `string` carrier, each
validated by the crate the [Standards](#standards) table below names.

| Type | Accepts | Delegate |
|------|---------|----------|
|`uuid`|`018f5e2a-0000-7000-8000-000000000000` — canonical hyphenated form only; braced, `urn:uuid:`, and unhyphenated spellings are rejected|`uuid`|
|`date`|`2026-08-28`|`time`|
|`time`|`14:30:00`, with no leading `T`|`time`|
|`timestamp`|`2026-08-28T14:30:00Z`|`time`|
|`duration`|`T#5s500ms` — IEC 61131-3, units descending, each used once|hand-written|
|`duration_iso`|`P1DT2H30M`|`iso8601-duration`|
|`duration_human`|`1h30m`, `500ms`|`humantime`|
|`file_name`|`README.md` — one segment, no separators, not `.` or `..`|hand-written|
|`path`|`/path/to/file` — a text shape; nothing here touches a filesystem|hand-written|
|`email`|`user@example.com`|`email_address`|
|`url`|`https://example.com/path` — absolute only|`url`|
|`ip_address`|`192.168.1.1` or `2001:db8::1`|`std::net::IpAddr`|
|`language`|`en` or `en.EN`|hand-written|
|`currency_code`|`USD`|`iso_currency`|
|`soft_date`|`2026-08` to the month, `2026-35` for Q3, `2026-08-12?` uncertain, `XXXX` not known|`edtf-core`|
|`soft_date_range`|`2026-07/2026-09`, `2026-10/..` open, `2026-10/` end unknown|`edtf-core`|

Three points worth knowing before using these:

**Reading a simple type normalises.** A primitive value *is* its text, so the
text is kept exactly. A simple value is the thing its text denotes, so the
meaning is kept and the canonical form is what comes back out.

```rust
use sheni_types::SimpleType;

// RFC 5952's canonical IPv6 form.
assert_eq!(SimpleType::IpAddress.read("2001:0DB8::1").unwrap().to_string(), "2001:db8::1");
// WHATWG says the root path is there whether it was typed or not.
assert_eq!(SimpleType::Url.read("https://example.com").unwrap().to_string(), "https://example.com/");
```

**Duration is three types, not one.** `T#1m`, `PT1M`, and `1m` are the same
minute in three conventions, and none of the three accepts the others' text. A
type that sniffed which one it had been handed would be guessing.

**`language` is not BCP 47.** It follows this README's `en.EN` — a dot and a
non-region region — rather than BCP 47's `en-GB`. ADR sheni-0002 records the
divergence and expects it to be revisited.

**Two of them are soft, and they are the only simple types with a fallback.**
`soft_date` is a date known to any precision — a year, a year and month, a
quarter, a full day — or flagged uncertain, or not known at all, per ISO
8601-2's Extended Date/Time Format. It exists because imprecision is not
absence: "August 2026" is a date at a coarser grain, not a missing one, and it
can still be compared, sorted and rolled up.

```rust
use sheni_types::{Precision, SimpleType};

// Sub-year grouping codes 33-36 are the quarters, so Q3 needs nothing invented.
let due = SimpleType::SoftDate.read("2026-35").unwrap();
assert_eq!(due.precision(), Some(Precision::Season));

// `date` did not widen: a field declared `date` promises the day is known.
assert!(SimpleType::Date.read("2026-08").is_err());
```

`soft_date_range` is its sibling for spans, because a range is a pair and a
date is a point. Either end may be a coarse date, `..` for open (it genuinely
has no end), or empty for unknown (it has one nobody recorded) — different
claims that the standard keeps apart.

The fallbacks are `XXXX` and `XXXX/XXXX`, and the second was looked up rather
than chosen: EDTF rejects `/` and `../..` because an interval needs at least
one dated endpoint, while a fully unspecified date *is* dated. So the range's
fallback is the point's at both ends.

Sets (`[1667,1668]`, `{1667,1668}`) are excluded permanently — both brackets
are forbidden constructs in SYON, so the value is unwritable rather than merely
unsupported — and a date with a time of day is excluded because `timestamp`
already owns it. Both say so with their own code rather than reporting
malformed text. See
[ADR sheni_06](../../design/architecture/ADR_sheni_06__soft_dates.syon) and
[ADR sheni_10](../../design/architecture/ADR_sheni_10__soft_date_ranges.syon).

`id` is deliberately not a type: the outline above defines it as *carried by*
an integer or a string, which makes it a role a field plays rather than a
shape to check. `uuid` is the type.

## Standards 

|Data Type             |Standard                               |Format Example                    |Notes                                                                   |Rust Crate                                          |
|----------------------|---------------------------------------|----------------------------------|------------------------------------------------------------------------|----------------------------------------------------|
|Date                  |ISO 8601                               |`2026-08-28`                      |Calendar date                                                           |`chrono`, `time`                                    |
|Time                  |ISO 8601                               |`14:30:00`                        |24-hour clock                                                           |`chrono`, `time`                                    |
|Timestamp             |ISO 8601 / RFC 3339                    |`2026-08-28T14:30:00Z`            |RFC 3339 is a stricter internet-facing profile of ISO 8601              |`chrono`, `time`                                    |
|Duration (period)     |ISO 8601                               |`P1DT2H30M`                       |Period-designator style — 1 day, 2 hours, 30 minutes                    |`iso8601-duration`                                  |
|Duration (engineering)|IEC 61131-3                            |`T#5s500ms`                       |PLC-style compact duration literal, not an ISO/RFC standard             |none well-established — likely custom parser        |
|Duration (human-typed)|de facto (`humantime` convention)      |`5s`, `500ms`, `1h30m`            |No formal standard, common Rust ecosystem convention                    |`humantime`                                         |
|Email address         |RFC 5322 / RFC 6531                    |`user@example.com`                |5322 defines the format; 6531 extends it for internationalized addresses|`email_address`, `lettre` (has a validator)         |
|UUID                  |RFC 9562 (obsoletes RFC 4122)          |`018f5e2a-...`                    |Covers UUID v1–v8; already using v7 for Beriah issue reports            |`uuid`                                              |
|Currency code         |ISO 4217                               |`USD`, `EUR`                      |Defines the code only, not an amount serialization format               |`iso_currency`, `rusty-money` (also handles amounts)|
|IPv4 address          |RFC 791 (original), format per RFC 6943|`192.168.1.1`                     |Dotted-decimal notation                                                 |`std::net::Ipv4Addr`                                |
|IPv6 address          |RFC 8200 (spec), format per RFC 5952   |`2001:db8::8a2e:370:7334`         |RFC 5952 defines canonical text form                                    |`std::net::Ipv6Addr`                                |
|URL                   |RFC 3986 / WHATWG URL Living Standard  |`https://example.com/path?query=1`|The `url` crate implements WHATWG, not RFC 3986 directly                |`url`                                               |
|Soft date / range     |ISO 8601-2:2019 Annex A (EDTF)         |`2026-35`, `2026-10/..`           |Reduced precision, uncertainty, and intervals with open or unknown ends |`edtf-core`                                         |
