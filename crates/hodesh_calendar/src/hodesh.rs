//! Hodesh -- a lunisolar calendar with numbered months, meant as a standard.
//!
//! Hodesh is a deliberate variant of the Hebrew calendar, keeping its
//! structure and discarding two things: the month *names*, which are a
//! cultural inheritance a world standard cannot ask everyone to adopt, and
//! the observational and rabbinic machinery that decides when a year begins,
//! which cannot be computed from first principles by someone who does not
//! already have the tables. What is left is the part that is astronomy:
//! months that follow the moon, years that follow the sun, and a rule that
//! reconciles them.
//!
//! # The month is the moon
//!
//! A month begins at a new moon. Not a *named* new moon and not an *observed*
//! one -- the **mean** new moon, the arithmetic average lunation of
//! 29.530588853 days, counted forward from the epoch. Month `m` begins on the
//! day containing mean new moon `m`:
//!
//! ```text
//! start(m) = EPOCH + floor(m * 29.530588853)
//! ```
//!
//! That single line is the whole month rule, and every month length falls out
//! of it. Because 29.530588853 is a little over 29.5, the lengths come out
//! alternating 29, 30, 29, 30 -- odd months short, even months long -- with an
//! extra 30-day month roughly every 33 months to absorb the remainder. Over a
//! full 19-year cycle that is 124 long months and 111 short ones, seven more
//! long months than a strict alternation would give.
//!
//! It is worth being precise about why the alternation is a *consequence*
//! rather than the rule. Stated as a rule -- odd months 29 days, even months
//! 30, always -- it gives a mean month of exactly 29.5 days, which is 0.03
//! days short of a real lunation. That is 10.7 days of drift every 19 years:
//! within three centuries the calendar's new moon would fall at the full
//! moon, and a calendar that claims to follow the moon would no longer be
//! doing so. Deriving the lengths from the lunation gives the same pattern
//! the rule intended and never drifts.
//!
//! # The year is the sun
//!
//! Twelve lunations are 354 days, eleven short of a solar year, so a
//! lunisolar calendar has to add a whole month periodically -- days cannot be
//! added loose without breaking the month rule above. Hodesh uses the
//! Metonic cycle: 19 years hold 235 months, because 235 lunations
//! (6939.69 days) and 19 solar years (6939.60 days) agree to within two
//! hours. Seven years of the nineteen have a thirteenth month, at cycle
//! positions 3, 6, 8, 11, 14, 17 and 19 -- the same distribution the Hebrew
//! calendar uses, which spreads the intercalations as evenly as seven into
//! nineteen allows.
//!
//! Those two hours are not zero, and they run long rather than short: the
//! mean hodesh year is 365.24662 days against a tropical year of 365.24219,
//! so the new year drifts *later* through the seasons by one day every 220
//! years. The residual is left uncorrected, and ADR hodesh_03 records why --
//! it cannot be corrected in days, because a month here is a lunation and
//! there is no spare day to remove, so the only lever is omitting an
//! intercalation, and the drift needs some 340 cycles to accumulate the whole
//! month that would cost.
//!
//! The leap month is month 13, appended at the end of the year rather than
//! inserted in the middle. The Hebrew calendar inserts its leap month before
//! the last month so the festival months keep their names; hodesh has no
//! festivals and no names, so there is nothing to protect and appending keeps
//! months 1 through 12 at fixed positions in every year.
//!
//! # The epoch
//!
//! Year 0, month 1, day 1 is proleptic Gregorian 2000-01-06, the first new
//! moon of the year 2000. Year 0 rather than year 1: the year numbering is a
//! signed count from a reference point, so arithmetic across the epoch needs
//! no special case, which is the same reasoning behind allowing a Gregorian
//! year 0 in [`crate::gregorian`].
//!
//! # What is not here
//!
//! Nothing in this calendar depends on a location or a season boundary. Days
//! begin at midnight UTC ([`hodesh_types::FixedDay`]), so a hodesh date is
//! the same date everywhere on Earth at the same instant, and there is no
//! summer time to shift it.

use std::fmt;
use std::str::FromStr;

use hodesh_types::{CalendarDate, DateError, FixedDay, HodeshCode};

use crate::gregorian::GregorianDate;

/// The calendar's name, as it appears in a [`DateError`].
pub const CALENDAR: &str = "hodesh";

/// The mean synodic month, in units of 10^-9 days: 29.530588853 days.
///
/// This is the modern mean lunation, not the Hebrew calendar's `molad`
/// interval of 29d 12h 793p. The `molad` value is 29.530594 days, long by
/// half a second a month, which is a known and accumulating error -- roughly
/// four days since it was fixed. A calendar written now has no reason to
/// inherit it.
pub const MEAN_LUNATION_NANODAYS: i64 = 29_530_588_853;

/// The same constant as days, for estimates only. Never used where an exact
/// answer is required.
const MEAN_LUNATION_DAYS: f64 = 29.530_588_853;

/// The number of years in the Metonic cycle.
pub const CYCLE_YEARS: i64 = 19;

/// The number of months in the Metonic cycle.
pub const CYCLE_MONTHS: i64 = 235;

/// Which positions of the 19-year cycle carry a thirteenth month. Indexed
/// 0-based, so position 2 here is the third year of the cycle.
const LEAP_POSITIONS: [bool; 19] = [
    false, false, true, false, false, true, false, true, false, false, true, false, false, true,
    false, false, true, false, true,
];

/// Months elapsed before each position of the cycle, cumulative. The final
/// entry is [`CYCLE_MONTHS`], which is what makes the cycle close.
const MONTHS_BEFORE_POSITION: [i64; 20] = [
    0, 12, 24, 37, 49, 61, 74, 86, 99, 111, 123, 136, 148, 160, 173, 185, 197, 210, 222, 235,
];

/// Proleptic Gregorian 2000-01-06, the new moon that starts hodesh year 0.
fn epoch() -> FixedDay {
    FixedDay::new(730_125)
}

/// The day month `index` begins on, counting months from the epoch. Negative
/// indices run backwards before it.
///
/// This is the definition of the calendar; everything else in the module is
/// bookkeeping around it.
fn month_start(index: i64) -> FixedDay {
    let elapsed = (index as i128 * MEAN_LUNATION_NANODAYS as i128).div_euclid(1_000_000_000);
    epoch() + elapsed as i64
}

/// The absolute month index of month 1 of `year`.
fn first_month_of_year(year: i64) -> i64 {
    let cycle = year.div_euclid(CYCLE_YEARS);
    let position = year.rem_euclid(CYCLE_YEARS) as usize;
    CYCLE_MONTHS * cycle + MONTHS_BEFORE_POSITION[position]
}

/// Whether `year` has a thirteenth month.
pub fn is_leap_year(year: i64) -> bool {
    LEAP_POSITIONS[year.rem_euclid(CYCLE_YEARS) as usize]
}

/// How many months `year` has: 12, or 13 in a leap year.
pub fn months_in_year(year: i64) -> u8 {
    if is_leap_year(year) {
        13
    } else {
        12
    }
}

/// How many days month `month` of `year` has -- 29 or 30, as the lunation
/// dictates. Returns 0 for a month that does not exist in that year.
pub fn days_in_month(year: i64, month: u8) -> u8 {
    if month < 1 || month > months_in_year(year) {
        return 0;
    }
    let index = first_month_of_year(year) + (month as i64 - 1);
    (month_start(index + 1) - month_start(index)) as u8
}

/// How many days `year` has: 354 or 355 in a common year, 383 or 384 in a
/// leap year, as the lunations fall.
pub fn days_in_year(year: i64) -> u16 {
    (month_start(first_month_of_year(year + 1)) - month_start(first_month_of_year(year))) as u16
}

/// A date in the hodesh calendar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HodeshDate {
    year: i64,
    month: u8,
    day: u8,
}

impl HodeshDate {
    /// A date from its parts, checked.
    ///
    /// Both bounds depend on the year: month 13 exists only in a leap year,
    /// and whether day 30 exists depends on where the moon was.
    pub fn new(year: i64, month: u8, day: u8) -> Result<HodeshDate, DateError> {
        let last_month = months_in_year(year);
        if month < 1 || month > last_month {
            return Err(DateError::new(
                HodeshCode::MonthOutOfRange,
                CALENDAR,
                format!(
                    "month {month} does not exist in year {year}, which has {last_month} months"
                ),
            ));
        }
        let last_day = days_in_month(year, month);
        if day < 1 || day > last_day {
            return Err(DateError::new(
                HodeshCode::DayOutOfRange,
                CALENDAR,
                format!(
                    "day {day} does not exist in month {month} of {year}, which has {last_day} days"
                ),
            ));
        }
        Ok(HodeshDate { year, month, day })
    }

    /// The year. Year 0 begins at the first new moon of Gregorian 2000.
    pub fn year(self) -> i64 {
        self.year
    }

    /// The month, `1..=12`, or `1..=13` in a leap year.
    pub fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, `1..=30`.
    pub fn day(self) -> u8 {
        self.day
    }

    /// Whether this date's year carries a thirteenth month.
    pub fn is_leap_year(self) -> bool {
        is_leap_year(self.year)
    }

    /// The day of the year, counting from month 1 day 1.
    pub fn ordinal(self) -> u16 {
        let year_start = month_start(first_month_of_year(self.year));
        (self.to_fixed() - year_start + 1) as u16
    }

    /// How far into the lunation this day is, `0..=29`, where 0 is the new
    /// moon the month began at.
    ///
    /// This is `day - 1` and exists to say so: in hodesh the day of the month
    /// *is* the age of the moon, which is the property the calendar is built
    /// to have.
    pub fn moon_age(self) -> u8 {
        self.day - 1
    }
}

impl CalendarDate for HodeshDate {
    const CALENDAR: &'static str = CALENDAR;

    fn to_fixed(&self) -> FixedDay {
        let index = first_month_of_year(self.year) + (self.month as i64 - 1);
        month_start(index) + (self.day as i64 - 1)
    }

    fn from_fixed(fixed: FixedDay) -> HodeshDate {
        // Estimate the month from the mean lunation, then walk to the true
        // one. The estimate is never more than a month out, so the loops run
        // at most once each; they are loops rather than a single correction
        // so that the result is right by construction and not by argument.
        let elapsed = (fixed - epoch()) as f64;
        let mut index = (elapsed / MEAN_LUNATION_DAYS).floor() as i64;
        while month_start(index) > fixed {
            index -= 1;
        }
        while month_start(index + 1) <= fixed {
            index += 1;
        }

        let cycle = index.div_euclid(CYCLE_MONTHS);
        let within = index.rem_euclid(CYCLE_MONTHS);
        // Which year of the cycle holds this month: the last position whose
        // cumulative month count does not exceed it.
        let position = MONTHS_BEFORE_POSITION
            .iter()
            .rposition(|&before| before <= within)
            .expect("position 0 holds 0 months, so some entry always matches")
            .min(18);

        HodeshDate {
            year: CYCLE_YEARS * cycle + position as i64,
            month: (within - MONTHS_BEFORE_POSITION[position] + 1) as u8,
            day: (fixed - month_start(index) + 1) as u8,
        }
    }
}

impl HodeshDate {
    /// The same day in the proleptic Gregorian calendar.
    ///
    /// A convenience for the one conversion every caller wants; the general
    /// form is [`CalendarDate::convert`].
    pub fn to_gregorian(self) -> GregorianDate {
        self.convert()
    }

    /// The hodesh date for a proleptic Gregorian one.
    pub fn from_gregorian(date: GregorianDate) -> HodeshDate {
        date.convert()
    }
}

impl fmt::Display for HodeshDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

impl FromStr for HodeshDate {
    type Err = DateError;

    fn from_str(s: &str) -> Result<HodeshDate, DateError> {
        let (year, month, day) = crate::parse_ymd(s, CALENDAR)?;
        HodeshDate::new(year, month, day)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_epoch_is_the_first_new_moon_of_2000() {
        let start = HodeshDate::new(0, 1, 1).unwrap();
        assert_eq!(start.to_gregorian().to_string(), "2000-01-06");
        assert_eq!(start.to_fixed(), FixedDay::new(730_125));
    }

    #[test]
    fn months_alternate_29_and_30_through_the_first_year() {
        let lengths: Vec<u8> = (1..=12).map(|m| days_in_month(0, m)).collect();
        assert_eq!(
            lengths,
            vec![29, 30, 29, 30, 29, 30, 29, 30, 29, 30, 29, 30]
        );
        assert_eq!(days_in_year(0), 354);
    }

    #[test]
    fn the_alternation_is_broken_only_to_follow_the_moon() {
        // Over 235 months the pattern must yield 6939 days, not the 6932 a
        // strict odd-29/even-30 alternation would give. The seven extra days
        // are seven extra long months, one roughly every 33 months.
        let mut lengths = Vec::new();
        for index in 0..CYCLE_MONTHS {
            lengths.push((month_start(index + 1) - month_start(index)) as u16);
        }
        let total: u16 = lengths.iter().sum();
        assert_eq!(total, 6939);
        assert!(lengths.iter().all(|&n| n == 29 || n == 30));

        let long = lengths.iter().filter(|&&n| n == 30).count();
        let short = lengths.iter().filter(|&&n| n == 29).count();
        assert_eq!((long, short), (124, 111));

        // A strict alternation over 235 months starting short gives 117 long
        // months and 6932 days. The moon asks for seven more.
        assert_eq!(long - 117, 7);
    }

    #[test]
    fn seven_years_in_nineteen_are_leap_years() {
        let leaps: Vec<i64> = (0..19).filter(|&y| is_leap_year(y)).collect();
        assert_eq!(leaps, vec![2, 5, 7, 10, 13, 16, 18]);
        assert_eq!((0..19).map(|y| months_in_year(y) as i64).sum::<i64>(), 235);
    }

    #[test]
    fn a_leap_year_has_a_thirteenth_month_and_a_common_year_does_not() {
        assert!(HodeshDate::new(2, 13, 1).is_ok());
        assert_eq!(
            HodeshDate::new(0, 13, 1).unwrap_err().code(),
            HodeshCode::MonthOutOfRange
        );
        assert_eq!(days_in_month(0, 13), 0);
    }

    #[test]
    fn year_lengths_stay_in_the_lunisolar_band() {
        for year in -400..400 {
            let days = days_in_year(year);
            if is_leap_year(year) {
                assert!((383..=384).contains(&days), "year {year} had {days} days");
            } else {
                assert!((354..=355).contains(&days), "year {year} had {days} days");
            }
        }
    }

    #[test]
    fn the_cycle_closes_on_the_metonic_period() {
        for cycle in -20..20 {
            let start = month_start(first_month_of_year(cycle * CYCLE_YEARS));
            let end = month_start(first_month_of_year((cycle + 1) * CYCLE_YEARS));
            let days = end - start;
            assert!(
                (6939..=6940).contains(&days),
                "cycle at year {} was {days} days",
                cycle * CYCLE_YEARS
            );
        }
    }

    #[test]
    fn the_mean_year_drifts_later_by_about_a_day_per_220_years() {
        // Measured over whole cycles only: a partial span ends at an
        // arbitrary number of intercalations, and would report a mean year
        // that says more about where it stopped than about the rule.
        const CYCLES: i64 = 105;
        let years = CYCLES * CYCLE_YEARS;
        let days = month_start(first_month_of_year(years)) - month_start(first_month_of_year(0));
        assert_eq!(days, 728_667);

        let mean_year = days as f64 / years as f64;
        let drift = mean_year - 365.242_190_f64;

        assert!(
            (mean_year - 365.246_62).abs() < 1e-5,
            "mean year {mean_year}"
        );
        assert!(
            drift > 0.0,
            "the cycle runs long, so the new year drifts later"
        );
        // 219 years asymptotically; this span reads 226 because the floor
        // at its end discards 0.28 of a day, which is 0.00014 d/yr over 1995.
        assert!(
            (210.0..235.0).contains(&(1.0 / drift)),
            "one day of drift per {} years",
            1.0 / drift
        );

        // Correcting it would mean skipping an intercalation, which costs a
        // whole month, and that much drift is 340 cycles away. Which is why
        // nothing here corrects it.
        let cycles_per_month = (MEAN_LUNATION_DAYS / drift) / CYCLE_YEARS as f64;
        assert!(
            (330.0..360.0).contains(&cycles_per_month),
            "{cycles_per_month}"
        );
    }

    #[test]
    fn round_trips_over_two_thousand_years() {
        let start = HodeshDate::new(-1000, 1, 1).unwrap().to_fixed();
        for n in start.get()..start.get() + 730_000 {
            let fixed = FixedDay::new(n);
            let date = HodeshDate::from_fixed(fixed);
            assert_eq!(date.to_fixed(), fixed, "{date} did not round-trip");
            assert!(
                HodeshDate::new(date.year(), date.month(), date.day()).is_ok(),
                "{date} is unreachable through the checked constructor"
            );
        }
    }

    #[test]
    fn day_of_month_is_the_age_of_the_moon() {
        assert_eq!(HodeshDate::new(0, 1, 1).unwrap().moon_age(), 0);
        assert_eq!(HodeshDate::new(0, 1, 15).unwrap().moon_age(), 14);
    }

    #[test]
    fn ordinal_counts_from_the_first_day_of_the_year() {
        assert_eq!(HodeshDate::new(0, 1, 1).unwrap().ordinal(), 1);
        assert_eq!(
            HodeshDate::new(0, 12, days_in_month(0, 12))
                .unwrap()
                .ordinal(),
            days_in_year(0)
        );
    }

    #[test]
    fn conversion_goes_both_ways() {
        let gregorian = GregorianDate::new(2026, 8, 29).unwrap();
        let hodesh = HodeshDate::from_gregorian(gregorian);
        assert_eq!(hodesh.to_gregorian(), gregorian);
        assert_eq!(hodesh.weekday(), gregorian.weekday());
    }

    #[test]
    fn text_form_round_trips() {
        let date = HodeshDate::new(26, 5, 17).unwrap();
        assert_eq!(date.to_string(), "0026-05-17");
        assert_eq!("0026-05-17".parse::<HodeshDate>().unwrap(), date);
        assert_eq!(
            "0026-05-99".parse::<HodeshDate>().unwrap_err().code(),
            HodeshCode::DayOutOfRange
        );
    }
}
