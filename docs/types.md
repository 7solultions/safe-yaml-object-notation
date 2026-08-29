# The type layer

SYON parses; it does not interpret. A successful parse hands back the text
between the delimiters, unchanged and untyped — `2026-08-28` comes out as the
five-character-per-part string it was written as, and so does `no`, and so
does `007`.

**Sheni** is the layer that says what that text means. It is the second layer
of the ONE stack (`syon` parses, `sheni` types, `shelishi` takes types from a
runtime schema), and it lives in the `sheni_types` crate.

## Four groups

A Sheni type belongs to exactly one group, and the group says what kind of
thing it is — never where its definition came from.

| Group | What it is | Examples |
|-------|------------|----------|
| **primitive** | A value with no interpretation beyond its own shape | `bool`, `u8`, `i64`, `f64`, `byte`, `char`, `string` |
| **simple** | An interpretation over a primitive carrier, governed by a published standard | `uuid`, `date`, `duration_iso`, `email`, `url`, `soft_date` |
| **complex** | A curated library of composites shipped in the crate | enums and structs *(designed, not built)* |
| **collection** | A container over other types | `list<T>`, `map<K,V>` *(designed, not built)* |

## Reading is checked, never guessed

The accepted text forms are a closed set. Anything outside it is an error with
a stable numeric code, not a silent coercion.

```rust
use sheni_types::{PrimitiveType, PrimitiveValue, SheniCode};

let u8_type = PrimitiveType::from_name("u8").unwrap();

// 300 does not fit in a u8, and says so rather than wrapping.
assert_eq!(u8_type.read("300").unwrap_err().code(), SheniCode::IntegerOutOfRange);

// `no` is a boolean only where a boolean was declared.
assert_eq!(PrimitiveType::Boolean.read("no"), Ok(PrimitiveValue::Boolean(false)));
assert_eq!(
    PrimitiveType::String.read("no"),
    Ok(PrimitiveValue::String("no".to_string()))
);
```

That last pair is the point of the whole layer. In YAML, whether `no` is a
boolean depends on the parser's mood. In SYON it depends on the declared type,
and nothing else.

A **primitive** value *is* its text, so the text survives exactly. A **simple**
value is the thing the text *denotes*, so the meaning survives and the
canonical spelling comes back:

```rust
use sheni_types::SimpleType;

// RFC 5952 says the canonical IPv6 form is lowercase and compressed.
assert_eq!(
    SimpleType::IpAddress.read("2001:0DB8::1").unwrap().to_string(),
    "2001:db8::1"
);
```

## Optional fields, and the fallback rule

Here is the rule that shapes everything below it:

> A field may be **optional** only where its type has a **fallback**, and a
> fallback must be a member of the value space meaning *"not known"* — never a
> legal value borrowed to stand in for one.

The second half is the part that does the work. `false` is not a missing
boolean; it is an answer. Zero is not a missing count; a count of zero is a
fact somebody established. An epoch date is not a missing date; some document
will eventually mean 1970 literally.

So `bool`, `u8` and `date` have no fallback, and a field of one of those is
**required**. There is no way to write a default beside the field, either —
the fallback belongs to the type, so an author cannot nominate one.

## The soft types

That rule leaves a real gap: an optional flag is the most commonly wanted
optional field there is. The gap is closed by widening the *type*, not by
letting the field go missing.

> Where a value may genuinely be unknown, model the unknown as a member of the
> type's value space, not as a field that might be missing.

Types that do this are named with a `soft_` prefix.

!!! warning "`soft_` does not mean lenient"

    A soft type accepts exactly what its strict twin accepts, plus one thing:
    a way to say *not known*. It does not guess, repair, trim, or sniff. Every
    range check and shape rule is the same.

### `soft_bool` and the soft primitives

Every primitive except `string` has a twin: `soft_bool`, `soft_u8` …
`soft_u128`, `soft_i8` … `soft_i128`, `soft_f32`, `soft_f64`, `soft_byte`,
`soft_char`. Each accepts its strict twin's literals plus the word `unknown`.

```rust
use sheni_types::SoftPrimitiveType;

let count = SoftPrimitiveType::from_name("soft_u32").unwrap();

assert_eq!(count.fallback().to_string(), "unknown");
assert_ne!(count.fallback(), count.read("0").unwrap());   // the whole point
```

There is no `soft_string`, and the reason is worth following: `string` accepts
any well-formed UTF-8 verbatim, so no text lies outside its value space, so no
word could mean "not known" without also being a string somebody meant
literally. The empty string, meanwhile, is a genuine identity — "set to
nothing" and "nothing set" coincide harmlessly — so `string` never needed one.

`soft_bool` carries **Kleene's strong three-valued logic**, the same logic SQL
uses. Rank `false` at 0, `unknown` at 1 and `true` at 2, and the operations
become arithmetic:

```rust
use sheni_types::SoftPrimitiveType;

let t = SoftPrimitiveType::from_name("soft_bool").unwrap();
let (no, dunno) = (t.read("false").unwrap(), t.read("unknown").unwrap());

assert_eq!(no.and(&dunno).unwrap(), no);      // false and unknown is false
assert_eq!(dunno.not().unwrap(), dunno);      // not unknown is unknown
```

`and` is the minimum, `or` the maximum, `not` the complement against 2. They
ship with the type because truth tables are where three-valued logic is
reliably got wrong.

### `soft_date` and `soft_date_range`

A date that is not fully known is not a missing date. "August 2026" is a date
at a coarser grain — it can still be compared, sorted, filtered and rolled up.
Modelling it as null throws away everything it actually says.

`soft_date` follows **ISO 8601-2:2019 Annex A**, the Extended Date/Time Format,
which libraries and archives have used for a decade to say "about 1920"
without lying about the day.

| Written | Means |
|---------|-------|
| `2026-08-12` | A complete day |
| `2026-08` | Known to the month |
| `2026-35` | Q3 2026 — sub-year grouping codes 33–36 are the quarters |
| `2026` | Known to the year |
| `2026-08-12?` | That day, uncertain |
| `2026~` | About 2026 |
| `XXXX` | Not known at all — and this is the fallback |

`soft_date_range` is its sibling for spans, because a range is a pair and a
date is a point:

| Written | Means |
|---------|-------|
| `2026-07/2026-09` | Between those two months |
| `2026-10/..` | From October, **open** — it genuinely has no end |
| `2026-10/` | From October, end **unknown** — it has one nobody recorded |
| `../2026-10` | Until October, open at the start |
| `XXXX/XXXX` | Neither end known — the fallback |

Open and unknown are different claims, and the standard keeps them apart.

!!! note "The fallback was looked up, not chosen"

    The obvious spelling for a wholly unknown range is `/`, and EDTF rejects
    it: an interval needs at least one dated endpoint. `XXXX/XXXX` is
    accepted, because a fully unspecified date *is* dated — it is a date that
    specifies nothing. The fallback for the range is therefore the fallback
    for the point at both ends.

```rust
use sheni_types::{Precision, SimpleType};

let due = SimpleType::SoftDate.read("2026-35").unwrap();
assert_eq!(due.precision(), Some(Precision::Season));  // a quarter

// `date` did not widen. A field declared `date` promises the day is known.
assert!(SimpleType::Date.read("2026-08").is_err());
assert!(SimpleType::SoftDate.read("2026-08").is_ok());
```

Two things EDTF can express that Sheni does not accept: a **set** of candidate
dates, written `[1667,1668]` or `{1667,1668}`, because both brackets are
forbidden constructs in SYON — the value is unwritable rather than merely
unsupported; and a date with a **time of day**, because Sheni already has
`timestamp` and two spellings of one thing is worse than one.

## Worked example: a planner

`examples/planner/` is a task tracker written to exercise exactly this. Three
task files, one schema.

A task where most things are known:

```syon
id: 018f5e2a-0000-7000-8000-000000000000
title: Draft the product roadmap
due: 2026-35
window: 2026-07/2026-09
estimate: P3DT4H
priority: 2
assignee_count: 3
blocked: false
```

A task where almost nothing is, written out rather than omitted so the
document says so explicitly:

```syon
id: 018f5e2a-0000-7000-8000-000000000001
title: Open the second engineering role
due: XXXX
window: 2026-10/..
estimate: P10D
priority: 5
assignee_count: unknown
blocked: unknown
```

And a task with only the required fields, where every optional one is absent:

```syon
id: 018f5e2a-0000-7000-8000-000000000002
title: Schedule the security audit
estimate: P2D
priority: 4
```

Read that third one through Sheni and `assignee_count` comes back `unknown`,
not `0`. `blocked` comes back `unknown`, not `false`. `due` comes back `XXXX`,
not `1970-01-01`. Meanwhile `estimate` and `priority` could not have been left
out at all, because `duration_iso` and `u8` have no fallback to take.

That is the difference between a planner that can say "we have not scheduled
this yet" and one that quietly claims the task is due at the epoch, unblocked,
and assigned to nobody.

The example is checked by `crates/sheni_types/tests/planner_example.rs`, which
reads every field at its declared type — so it cannot drift from the crate.

## Error codes

A failed read carries a `SheniCode`, banded by group so a caller can match a
number instead of message text.

| Band | Group |
|------|-------|
| 1–99 | General |
| 101–199 | Primitives |
| 201–299 | Simple types |
| 301–399 | Complex types *(reserved)* |
| 401–499 | Collections *(reserved)* |
| 501–599 | Reserved for the schema layer above |

```rust
use sheni_types::{PrimitiveType, SheniCode};

// Reaching for a soft type by writing its value at the strict one is its own
// error, and the message names the type you wanted.
let err = PrimitiveType::Boolean.read("unknown").unwrap_err();
assert_eq!(err.code(), SheniCode::UnknownAtStrictType);
assert_eq!(err.message(), "`unknown` is a value of `soft_bool`, not of `bool`");
```

## Where the decisions live

Every call above is recorded in an ADR, with the alternatives that were
rejected and why. The type-layer records are `ADR_sheni_*` in
[`design/architecture/`](decisions/README.md) — start with `sheni_01`
(primitives), `sheni_03` (the fallback rule) and `sheni_08` (what `soft_`
means).
