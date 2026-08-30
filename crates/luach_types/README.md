# luach_types

The calendar-agnostic base every calendar in this workspace stands on. It
knows what a **day** is and nothing about what a **date** is.

*Luach* (לוּחַ) is the board a calendar is written on, which is what this crate
is: the surface, not any of the calendars drawn on it.

| Item | What it is |
|------|------------|
| `FixedDay` | A count of days, day 1 at proleptic Gregorian 0001-01-01 (the *rata die*). Every calendar converts through it. |
| `Weekday` | The seven-day week, Monday first, per ISO 8601. A property of the day, so every calendar gets it free. |
| `CalendarDate` | The contract a calendar implements: a date to a `FixedDay` and back. Everything else is provided. |
| `DateError` / `LuachCode` | Why a date cannot exist, with a stable numeric code in the `601-699` band. |
| `parse_ymd` / `format_ymd` | The canonical `YYYY-MM-DD` text form, one strict spelling, shared by every calendar. |

## Why a day count

Converting between calendars pairwise needs a routine per pair — quadratic,
and a fresh chance to get an intercalation wrong each time. Converting through
one count is linear: a calendar states two things about itself and is then
connected to every other calendar without knowing they exist.

```rust
use luach_types::{CalendarDate, FixedDay, Weekday};

let day = FixedDay::new(730_120); // proleptic Gregorian 2000-01-01
assert_eq!(day.weekday(), Weekday::Saturday);
```

## Implementing a calendar

Two methods, which must round-trip for every representable date:

```rust
impl CalendarDate for MyDate {
    const CALENDAR: &'static str = "mine";
    fn to_fixed(&self) -> FixedDay { /* ... */ }
    fn from_fixed(fixed: FixedDay) -> Self { /* ... */ }
}
```

`weekday`, `add_days`, `days_until` and `convert` to any other calendar come
with it. A calendar defined outside this workspace interoperates with every
calendar in it without either side importing the other.

## Days are UTC

A `FixedDay` is a whole day beginning at midnight UTC. No local offset, no
summer time. A calendar meant as a standard cannot have a date that depends on
where the reader is standing.

## Decisions

- [`ADR_hodesh_01`](../../design/architecture/ADR_hodesh_01__two_crates_and_a_day_count.syon) — calendars convert through one day count, and the count lives in a crate below them
