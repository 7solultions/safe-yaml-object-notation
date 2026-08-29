# hodesh_calendar

The calendars themselves, and the conversions between them. Built on
[`hodesh_types`](../hodesh_types), which holds the day count they all convert
through.

| Calendar | What it is |
|----------|------------|
| `GregorianDate` | The proleptic Gregorian calendar. Present because it is what every existing record uses, and so the anchor every conversion goes through. |
| `HodeshDate` | A lunisolar calendar with numbered months — the one this crate is named for. |

## Hodesh in one page

A variant of the Hebrew calendar that keeps the astronomy and drops the
inheritance.

**The month is the moon.** A month begins at a mean new moon, counted from the
epoch:

```text
start(m) = EPOCH + floor(m × 29.530588853)
```

That is the entire month rule. Lengths fall out of it as 29, 30, 29, 30 — odd
months short, even months long — with an extra long month roughly every 33
months. Over a 19-year cycle: 124 long months, 111 short.

The alternation is a *consequence*, not the rule. Stated as a rule it would
mean a mean month of exactly 29.5 days, drifting 10.7 days from the moon every
19 years — within three centuries the calendar's "new" moon would be full.

**The year is the sun.** Twelve lunations are 354 days, eleven short of a solar
year, so seven years in every nineteen carry a thirteenth month — the Metonic
cycle, at positions 3, 6, 8, 11, 14, 17, 19. The leap month is *appended* as
month 13, so months 1–12 sit at the same position in every year.

Common years are 354–355 days, leap years 383–384.

**The epoch** is proleptic Gregorian 2000-01-06, the first new moon of the year
2000. Year numbering starts at **0**, and runs negative before it.

**Months are numbered, never named.** No Roman emperors, no name tables, no
translation. The day of the month is the age of the moon: day 1 is new, day 15
is near full, in every month of every year.

## Use

```rust
use hodesh_calendar::{GregorianDate, HodeshDate};
use hodesh_types::CalendarDate;

let start = HodeshDate::new(0, 1, 1)?;
assert_eq!(start.to_gregorian().to_string(), "2000-01-06");

let today = GregorianDate::new(2026, 8, 29)?;
let same_day: HodeshDate = today.convert();
assert_eq!(same_day.weekday(), today.weekday());
```

Dates are written `YYYY-MM-DD`, zero-padded, parsed strictly — `0026-05-17` is
a date, `26-5-17` is an error.

## Decisions

- [`ADR_hodesh_01`](../../design/architecture/ADR_hodesh_01__two_crates_and_a_day_count.syon) — calendars convert through one day count
- [`ADR_hodesh_02`](../../design/architecture/ADR_hodesh_02__month_is_the_mean_lunation.syon) — the month is the mean lunation, and the alternation is its consequence
- [`ADR_hodesh_03`](../../design/architecture/ADR_hodesh_03__metonic_year_and_appended_leap_month.syon) — the year is Metonic, and the leap month is appended as month 13
- [`ADR_hodesh_04`](../../design/architecture/ADR_hodesh_04__numbered_months_and_a_new_moon_epoch.syon) — months are numbered, and year zero begins at the first new moon of 2000
