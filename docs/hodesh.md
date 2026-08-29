# The hodesh calendar

Hodesh is a lunisolar calendar with numbered months. It is a deliberate
variant of the Hebrew calendar that keeps the astronomy and drops the
inheritance — no month names, no observational machinery, no timezone.

It sits alongside the SYON stack rather than on top of it, in two crates:
`hodesh_types` holds what is true of every calendar, and `hodesh_calendar`
holds the calendars themselves.

The whole month rule is one line:

```text
start(m) = EPOCH + floor(m × 29.530588853)
```

Everything below follows from it.

## Five facts

| | |
|---|---|
| **Month** | Begins at a mean new moon. 29 or 30 days — the moon decides, never the calendar. |
| **Year** | 12 months, or 13 in seven years out of nineteen, which holds it against the sun. |
| **Names** | None. Months are numbered `1`–`12`, and `13` in a leap year. |
| **Day of month** | *Is* the age of the moon. Day 1 is new, day 15 near full. |
| **Day boundary** | Midnight UTC. No local offset, no summer time. |

*Hodesh* (חודש) is the Hebrew word for **month**, sharing a root with *hadash*,
meaning **new**. A month is a new moon; the name is the definition.

## The month is the moon

A lunation — one new moon to the next — averages 29.530588853 days. Hodesh
takes that literally. Month `m`, counted from the epoch, begins on the day
containing mean new moon `m`. Floor the running total and you have the
calendar; a month's length is just the gap between consecutive starts, which
is always 29 or 30 days because the lunation falls between them.

Because a lunation is a little over 29½ days, the lengths that fall out
alternate — odd months short, even months long:

```text
month    1  2  3  4  5  6  7  8  9 10 11 12
days    29 30 29 30 29 30 29 30 29 30 29 30   = 354
```

But not forever. Every so often the accumulated half-days force two long
months in a row. Here are the first 40 months from the epoch, with the
corrections marked:

```text
m01..m16   29 30 29 30 29 30 29 30 29 30 29 30 29 30 29 30
m17..m32   30 29 30 29 30 29 30 29 30 29 30 29 30 29 30 29
           ^^
m33..m40   30 30 29 30 29 30 29 30
           ^^ ^^
```

Each `^^` pair is two long months in a row — the moon correcting the
alternation. (`m16` and `m17` straddle the line break.) That is not the
pattern failing; it is the pattern being a consequence of something real.

!!! warning "Why the alternation is not the rule"

    It is tempting to state the pattern as the rule: odd months 29 days, even
    months 30, done. That gives a mean month of exactly **29.5** days, which
    is 0.03 days short of a real lunation.

    Small, and it never stops accumulating: **10.7 days every 19 years**.
    Within three centuries the calendar's "new" moon would fall at the full
    moon, and the one thing hodesh claims about itself would be false.

    So the alternation is a *consequence*, not a rule. You get the pattern you
    wanted, and it never drifts, because the moon is what is actually counted.

Over a full 19-year cycle the rule yields **124 long months and 111 short
ones** — seven more long months than a strict alternation would give, and
6939 days rather than 6932.

!!! note "Not the *molad*"

    The constant is 29.530588853 days, the modern mean synodic month, held as
    an integer count of billionths of a day so the arithmetic is exact.

    It is deliberately **not** the Hebrew calendar's *molad* interval of
    29d 12h 793p. That value is 29.530594 days — long by about half a second
    a month, an error that has accumulated to roughly four days since it was
    fixed. Hodesh departs from the Hebrew calendar precisely to drop inherited
    error, so adopting this one would defeat the exercise.

    It is also deliberately the **mean** lunation rather than the true one.
    True new moons vary by up to thirteen hours either side of the mean, so a
    calendar built on them cannot be computed by hand, cannot be printed years
    ahead without an ephemeris, and turns a calendar into a lookup table.

## The day is the age of the moon

This is the property the whole design exists to protect. Because every month
starts at a new moon, the day of the month *is* how many days old the moon is:

| Day | Moon |
|-----|------|
| `01` | new |
| `08` | first quarter |
| `15` | full, or near it |
| `23` | last quarter |
| `29`/`30` | old crescent, and the next month begins |

This holds in month 3 of year 26 exactly as it does in month 11 of year 4000.
No other calendar in common use can tell you this — in the Gregorian calendar,
the 15th tells you nothing about the sky.

```rust
use hodesh_calendar::HodeshDate;

let date = HodeshDate::new(26, 9, 18)?;
assert_eq!(date.moon_age(), 17); // three days past full
```

## The year is the sun

Twelve lunations are 354 days — eleven short of a solar year. Left alone, the
calendar would slide backwards through the seasons, which is what a purely
lunar calendar does.

Hodesh fixes this the way every lunisolar calendar that survived does: with
the **Metonic cycle**. Nineteen years hold 235 months, because 235 lunations
(6939.69 days) and 19 solar years (6939.60 days) agree to within about two
hours. Seven of the nineteen carry a thirteenth month:

```text
year in cycle   0  1  2  3  4  5  6  7  8  9 10 11 12 13 14 15 16 17 18
months         12 12 13 12 12 13 12 13 12 12 13 12 12 13 12 12 13 12 13
                     ^^       ^^    ^^       ^^       ^^       ^^    ^^
```

That is the Hebrew calendar's own distribution — cycle positions 3, 6, 8, 11,
14, 17 and 19 counting from one — kept because it spreads seven intercalations
across nineteen years about as evenly as seven into nineteen allows.

The leap month is **appended as month 13**, not inserted mid-year. The Hebrew
calendar inserts its leap month before the last one so the named festival
months keep their seasons; hodesh has no names and no festivals, so there is
nothing to protect, and appending keeps months 1–12 at the same position in
every year.

Year lengths follow from this rather than being declared:

| | Months | Days |
|---|---|---|
| Common year | 12 | 354 or 355 |
| Leap year | 13 | 383 or 384 |
| Full cycle | 235 | 6939 or 6940 |

Nothing about the year is a rule. The lunation is the rule; the year only
decides how many months it holds.

!!! warning "The residual, stated honestly"

    Those two hours are not zero, and they run **long**. The mean hodesh year
    is 365.24662 days against a tropical year of 365.24219, so the new year
    drifts *later* through the seasons by about **one day every 220 years**.

    It is left uncorrected on purpose. A month here is a lunation, so there is
    no spare day to remove — the only available lever is omitting an
    intercalation, which costs a whole month, and that much drift is some 340
    cycles away. Correcting it would assert a precision the astronomy does not
    support over the six thousand years the correction would need.

## Reading and writing a date

A hodesh date is written `YYYY-MM-DD`, zero-padded, and parsed strictly.
`0026-05-17` is a date; `26-5-17` is an error. A format meant for interchange
has exactly one spelling — the same call `sheni`'s `date` type makes, for the
same reason.

Year **0** — not year 1 — begins at the first new moon of the year 2000, which
is proleptic Gregorian `2000-01-06`. Years before it run negative, with no gap
where a year zero should be. Arithmetic that has to special-case a missing
zero is arithmetic that is wrong somewhere.

| Gregorian | Hodesh | Weekday | What it is |
|-----------|--------|---------|------------|
| `2000-01-06` | `0000-01-01` | Thursday | The epoch — year 0, month 1, day 1, a new moon |
| `1969-07-20` | `-031-08-08` | Sunday | Apollo 11: a negative year, and a waxing moon |
| `2000-01-01` | `-001-13-26` | Saturday | Five days before the epoch, still in leap month 13 |
| `2026-08-29` | `0026-09-18` | Saturday | An ordinary day, three past full |
| `2026-12-25` | `0026-13-18` | Friday | Month 13 — year 26 is a leap year |

## Using it in code

Two crates. `hodesh_types` holds the calendar-agnostic pieces — a day count,
the week, and the contract a calendar implements. `hodesh_calendar` holds the
calendars: the proleptic Gregorian one, and hodesh.

Conversion is never written per pair. Each calendar says only how to reach a
day count and how to come back, and every pair is connected for free:

```rust
use hodesh_calendar::{GregorianDate, HodeshDate};
use hodesh_types::CalendarDate;

// Year 0 begins at the first new moon of 2000.
let start = HodeshDate::new(0, 1, 1)?;
assert_eq!(start.to_gregorian().to_string(), "2000-01-06");

// Conversion is by day, so the weekday is necessarily the same.
let today = GregorianDate::new(2026, 8, 29)?;
let same_day: HodeshDate = today.convert();
assert_eq!(same_day.weekday(), today.weekday());
assert_eq!(same_day.to_string(), "0026-09-18");
```

To add a calendar of your own, implement two methods against `hodesh_types`
alone:

```rust
impl CalendarDate for MyDate {
    const CALENDAR: &'static str = "mine";
    fn to_fixed(&self) -> FixedDay { /* ... */ }
    fn from_fixed(fixed: FixedDay) -> Self { /* ... */ }
}
```

`weekday`, `add_days`, `days_until` and `convert` to every other calendar come
with it, and neither side has to import the other. The contract is that the
two methods round-trip for every representable date, which every calendar here
tests by exhaustion over centuries rather than at sampled points.

Errors carry stable numeric codes in the `601`–`699` band, above the `501`–`599`
that `sheni` reserved for `shelishi`, so a hodesh code and a sheni code never
collide.

## Still open

Two decisions are proposed but **not implemented**. Everything above describes
the calendar as it actually runs.

??? question "Holocene year numbering — add 12,000 to the year"

    Adding 12,000 would put every date in recorded human history in positive
    numbers and retire AD/BC, which exports one culture's religion the same way
    *September* exports one culture's dead vocabulary. Hodesh already dropped
    the Roman month names on that argument; keeping AD/BC is inconsistent.

    The epoch would be a fixed round offset **motivated** by the Holocene —
    never *defined* as "the oldest known human building", because archaeology
    moves and a calendar's epoch must not. Göbekli Tepe only displaced the
    previous candidates during 1990s excavation, and its own dating has shifted
    since.

    Implementation note: leap years key on `year mod 19`, and 12000 mod 19 = 11,
    so the leap set would shift from {2, 5, 7, 10, 13, 16, 18} to
    {2, 5, 8, 10, 13, 16, 18} — still a 19-entry table.

??? question "Lettered month codes — `a01` through `u13`"

    Writing months as `a01`…`u13`, a vowel marking each group of three followed
    by the number, would make a hodesh date self-identifying: `12000-a01-01`
    cannot be misread as Gregorian. That closes a real gap, since today
    `0026-09-18` and a Gregorian date are indistinguishable.

    It sorts correctly (a, e, i, o, u happen to fall in alphabetical order),
    keeps month arithmetic if you ignore the first character, and only 20% of
    the `vowel + two digits` space is valid — so a typo is usually caught,
    where a mistyped two-letter code would be silently valid and wrong.

    The costs: it is ISO-*shaped* but not ISO-valid, so every ISO 8601 validator
    will reject it; and the vowels imply quarters that a lunisolar year does not
    really have — three lunar months is 88.6 days and maps to no season.

One thing is decided and deliberately left alone: the Metonic residual of
about a day per 220 years, for the reason given above.

## Decisions

The reasoning behind every choice on this page is recorded in the ADR log:

- **hodesh_01** — calendars convert through one day count, and the count lives in a crate below them
- **hodesh_02** — the month is the mean lunation, and the 29/30 alternation is its consequence rather than its rule
- **hodesh_03** — the year is Metonic, and the leap month is appended as month 13 rather than inserted
- **hodesh_04** — months are numbered rather than named, and year zero begins at the first new moon of 2000
