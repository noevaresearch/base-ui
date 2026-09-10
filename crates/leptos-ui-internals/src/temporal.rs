//! Port of `packages/react/src/internals/temporal/` — the shared temporal vocabulary
//! (`temporal.ts`, `temporal-adapter.ts`) consumed by the date/time components
//! (`TODO.md`, item `infra: internals`).
//!
//! Upstream splits the vocabulary into two pieces:
//!
//! - `temporal.ts` (`packages/react/src/internals/temporal/temporal.ts:13-49`): the
//!   `TemporalSupportedObjectLookup` module-augmentation registry — each date library
//!   declares its date-object type and `TemporalSupportedObject` collapses to the union
//!   (or `any` when nothing is registered; the React package registers `'date-fns': Date`
//!   at `TemporalAdapterDateFns.ts:96-100`, while the luxon registration is commented out
//!   at `TemporalAdapterLuxon.ts:44-48`) — plus the `TemporalTimezone` string union and
//!   the value/range aliases.
//! - `temporal-adapter.ts` (`packages/react/src/internals/temporal/temporal-adapter.ts`):
//!   the `TemporalAdapterFormats` table type and the ~60-method `TemporalAdapter`
//!   interface every adapter implements.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//!
//! - **The registry collapses to one concrete value type.** TypeScript resolves the
//!   adapter's values through the open-ended interface registry; in Rust the crate owns
//!   the vocabulary, so [`DateValue`] is the concrete "date library object": an instant
//!   (milliseconds since the Unix epoch, the JS `Date` representation) plus the attached
//!   zone (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts`
//!   pairs plain `Date`s with zone-attached `TZDate`s, and the two are the same value type
//!   here — a `Date` *is* an instant, and `TZDate` (`@date-fns/tz` `date/mini.js`) is an
//!   instant whose wall-clock fields are computed in `this.timeZone`). The
//!   [`Zone::System`] arm is the plain-`Date` world: its wall clock comes from the ambient
//!   system zone, which is why `getTimezone` on it reports `'system'`
//!   (`TemporalAdapterDateFns.ts:190-196`).
//! - **`TemporalSupportedObjectLookup` is N/A** — module augmentation has no Rust shape;
//!   the enum-of-one collapse (only `date-fns` is registered upstream) is absorbed by the
//!   single [`DateValue`] type. The `TemporalValue`/range aliases are likewise absorbed:
//!   nullability is `Option<DateValue>` at the trait boundary, which is upstream's
//!   `value: T | null` (the `isValid(null)` → `false` pin, `TemporalAdapterDateFns.ts:224-230`).
//! - **Upstream's `Invalid Date` sentinel collapses to `None`.** Upstream dates that fail
//!   to parse carry NaN time and every getter reports NaN; the port represents
//!   "no date" as `None` at the trait boundary (`date(null, …) → null`,
//!   `TemporalAdapterDateFns.ts:130`; a failed `parse` → Invalid Date), so
//!   [`TemporalAdapter::is_valid`] is the same `null → false` + validity check.
//! - **The timezone string union is a newtype.** [`TemporalTimezone`] carries the same
//!   accepted strings (`'default'`, `'system'`, `'UTC'`, any IANA name or fixed offset);
//!   [`TemporalAdapter::date`]-level `timezone === 'system' || timezone === 'default'`
//!   branches (`TemporalAdapterDateFns.ts:142`, `:172`, `:199`) read through
//!   [`TemporalTimezone::is_system`].
//! - **Zones resolve through `jiff`** with a bundled IANA database (the Rust equivalent
//!   of `@date-fns/tz`'s `Intl`-backed `tzOffset` lookups, `tz/tzOffset/index.js:20-37`),
//!   falling back to the fixed-offset forms `±HH(:MM(:SS)?)?` / `UTC±HH(:MM)?` the
//!   upstream fallback regex accepts (`tzOffset/index.js:66`). Zone resolution failing
//!   makes the *constructor* produce an invalid date (upstream `TZDateMini` sets NaN time
//!   when `tzOffset` is NaN, `date/mini.js:36-39`), which is `None` here.
//! - **The system zone is a seam.** Upstream tests drive the ambient zone through
//!   `process.env.TZ` (`TemporalAdapterDateFns.test.ts:17-26`); the port resolves
//!   `Zone::System` through [`system_zone`], which a test can override via
//!   [`set_system_zone`] (the `use_is_hydrating` runtime-seam precedent) because neither
//!   the host test runner nor a browser page offers a portable way to change the ambient
//!   zone. The default is `jiff`'s system zone.
//! - **`toJsDate` keeps the value type.** Upstream returns a plain `Date` copy
//!   (`TemporalAdapterDateFns.ts:213-218`); the port returns the same instant stripped to
//!   [`Zone::System`] — the plain-`Date` twin — rather than a JS object, since the crate
//!   is reactive-graph-only and the value never crosses into JS.

use std::cell::RefCell;

use jiff::tz::{Offset, TimeZone};
use jiff::{civil, Zoned};

/// The valid value for the timezone argument in components and utilities that deal with
/// dates and times (`packages/react/src/internals/temporal/temporal.ts:22-25`):
/// `'default' | 'system' | 'UTC' | string`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemporalTimezone(String);

impl TemporalTimezone {
    /// The `'default'` arm — behaves as the system timezone (`TemporalAdapterDateFns.ts:142`,
    /// `:172`).
    pub fn default_tz() -> TemporalTimezone {
        TemporalTimezone("default".to_string())
    }

    /// The `'system'` arm.
    pub fn system_tz() -> TemporalTimezone {
        TemporalTimezone("system".to_string())
    }

    /// The `'UTC'` arm — a named zone, not the system one.
    pub fn utc() -> TemporalTimezone {
        TemporalTimezone("UTC".to_string())
    }

    /// `'default' | 'system'` — the branches upstream spells
    /// `timezone === 'system' || timezone === 'default'`.
    pub fn is_system(&self) -> bool {
        self.0 == "system" || self.0 == "default"
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for TemporalTimezone {
    fn from(value: &str) -> Self {
        TemporalTimezone(value.to_string())
    }
}

impl From<String> for TemporalTimezone {
    fn from(value: String) -> Self {
        TemporalTimezone(value)
    }
}

impl std::fmt::Display for TemporalTimezone {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// The adapter format keys (`packages/react/src/internals/temporal/temporal-adapter.ts:100-94`),
/// the string keys of `TemporalAdapterFormats`. `TemporalAdapter::format` dispatches on
/// this instead of a string so a miss is a compile error, as the `keyof` union is upstream.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalFormatKey {
    /// The 4-digit year (`"2019"`).
    YearPadded,
    /// The month with leading zeros (`"08"`).
    MonthPadded,
    /// The day of the month with leading zeros (`"01"`).
    DayOfMonthPadded,
    /// The hours with leading zeros, 24-hour clock (`"01"`, `"23"`).
    Hours24hPadded,
    /// The hours with leading zeros, 12-hour clock (`"01"`, `"11"`).
    Hours12hPadded,
    /// The minutes with leading zeros (`"01"`, `"59"`).
    MinutesPadded,
    /// The seconds with leading zeros (`"01"`, `"59"`).
    SecondsPadded,
    /// The day of the month without leading zeros (`"1"`).
    DayOfMonth,
    /// The hours without leading zeros, 24-hour clock (`"1"`, `"23"`).
    Hours24h,
    /// The hours without leading zeros, 12-hour clock (`"1"`, `"11"`).
    Hours12h,
    /// The abbreviated month name (`"Aug"`).
    Month3Letters,
    /// The full month name (`"August"`).
    MonthFullLetter,
    /// The week day name (`"Wednesday"`).
    Weekday,
    /// The abbreviated week day name (`"Wed"`).
    Weekday3Letters,
    /// The initial of the week day name (`"W"`).
    Weekday1Letter,
    /// The meridiem (`"AM"`).
    Meridiem,
    /// The localized date format including year, month, day and weekday
    /// (`"Wednesday, August 6, 2014"`).
    LocalizedDateWithFullMonthAndWeekDay,
    /// The localized numeric date format including year, month and day (`"8/6/2014"`).
    LocalizedNumericDate,
}

/// The adapter format table (`packages/react/src/internals/temporal/temporal-adapter.ts:3-94`):
/// the date-library format strings each key resolves to. Each adapter carries its own
/// static table (`TemporalAdapterDateFns.ts:68-94`).
#[derive(Debug)]
pub struct TemporalAdapterFormats {
    pub year_padded: &'static str,
    pub month_padded: &'static str,
    pub day_of_month_padded: &'static str,
    pub hours24h_padded: &'static str,
    pub hours12h_padded: &'static str,
    pub minutes_padded: &'static str,
    pub seconds_padded: &'static str,
    pub day_of_month: &'static str,
    pub hours24h: &'static str,
    pub hours12h: &'static str,
    pub month3_letters: &'static str,
    pub month_full_letter: &'static str,
    pub weekday: &'static str,
    pub weekday3_letters: &'static str,
    pub weekday1_letter: &'static str,
    pub meridiem: &'static str,
    pub localized_date_with_full_month_and_week_day: &'static str,
    pub localized_numeric_date: &'static str,
}

impl TemporalAdapterFormats {
    /// The format string a key resolves to.
    pub fn get(&self, key: TemporalFormatKey) -> &'static str {
        match key {
            TemporalFormatKey::YearPadded => self.year_padded,
            TemporalFormatKey::MonthPadded => self.month_padded,
            TemporalFormatKey::DayOfMonthPadded => self.day_of_month_padded,
            TemporalFormatKey::Hours24hPadded => self.hours24h_padded,
            TemporalFormatKey::Hours12hPadded => self.hours12h_padded,
            TemporalFormatKey::MinutesPadded => self.minutes_padded,
            TemporalFormatKey::SecondsPadded => self.seconds_padded,
            TemporalFormatKey::DayOfMonth => self.day_of_month,
            TemporalFormatKey::Hours24h => self.hours24h,
            TemporalFormatKey::Hours12h => self.hours12h,
            TemporalFormatKey::Month3Letters => self.month3_letters,
            TemporalFormatKey::MonthFullLetter => self.month_full_letter,
            TemporalFormatKey::Weekday => self.weekday,
            TemporalFormatKey::Weekday3Letters => self.weekday3_letters,
            TemporalFormatKey::Weekday1Letter => self.weekday1_letter,
            TemporalFormatKey::Meridiem => self.meridiem,
            TemporalFormatKey::LocalizedDateWithFullMonthAndWeekDay => {
                self.localized_date_with_full_month_and_week_day
            }
            TemporalFormatKey::LocalizedNumericDate => self.localized_numeric_date,
        }
    }
}

/// The characters used to escape a string inside a format
/// (`packages/react/src/internals/temporal/temporal-adapter.ts:108-110`); both adapters
/// use single quotes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EscapedCharacters {
    pub start: &'static str,
    pub end: &'static str,
}

/// The zone a [`DateValue`] computes its wall-clock fields in.
///
/// [`Zone::System`] is the plain JS `Date` world (wall clock from the ambient zone;
/// `getTimezone` reports `'system'`); [`Zone::Named`] is the `TZDate` world — an IANA
/// name, `'UTC'`, or a fixed-offset string (`"-05:00"`), the forms `@date-fns/tz` accepts
/// (`tz/tzOffset/index.js:20-37`, fallback regex `:66`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Zone {
    System,
    Named(String),
}

/// The date value every adapter produces and consumes — the concrete stand-in for
/// upstream's `TemporalSupportedObject` (see the module docs).
#[derive(Clone, Debug)]
pub struct DateValue {
    /// Milliseconds since the Unix epoch — `value.getTime()`.
    pub(crate) millis: i64,
    /// The zone the wall-clock fields are computed in.
    pub(crate) zone: Zone,
}

impl DateValue {
    /// A plain `Date`-shaped value: the instant, wall clock in the system zone.
    pub fn from_millis(millis: i64) -> DateValue {
        DateValue {
            millis,
            zone: Zone::System,
        }
    }

    /// A `TZDate`-shaped value: the instant with an attached zone. Fails when the zone
    /// name is not resolvable (upstream's invalid-date path, `date/mini.js:36-39`).
    pub fn from_millis_in_zone(millis: i64, zone: impl Into<Zone>) -> Option<DateValue> {
        let zone = zone.into();
        resolve_zone(&zone)?;
        Some(DateValue { millis, zone })
    }

    /// The instant — `value.getTime()`.
    pub fn millis(&self) -> i64 {
        self.millis
    }

    /// The attached zone.
    pub fn zone(&self) -> &Zone {
        &self.zone
    }

    /// The value's wall clock in its zone — the TZDate `internal` date's fields
    /// (`date/mini.js:6-15`) / the plain `Date`'s local fields.
    pub(crate) fn wall(&self) -> civil::DateTime {
        self.zoned().datetime()
    }

    /// The instant projected into the value's zone.
    pub(crate) fn zoned(&self) -> Zoned {
        self.millis_zoned_in(self.millis, &self.zone)
    }

    /// An instant projected into an arbitrary zone (the `withTimeZone` shape —
    /// `date/mini.js:69-71`, `TemporalAdapterDateFns.ts:207`).
    pub(crate) fn millis_zoned_in(&self, millis: i64, zone: &Zone) -> Zoned {
        let tz = resolve_zone(zone).expect("DateValue zone is validated at construction");
        timestamp(millis).to_zoned(tz)
    }

    /// Rebuilds a value from wall-clock fields resolved in the value's zone — the TZDate
    /// multi-arg constructor world (`date/mini.js:55-60`). The wall clock must be
    /// representable; DST gaps resolve forward and folds keep the earlier offset
    /// (jiff's default "compatible" disambiguation, the JS `Date` behavior
    /// `adjustToSystemTZ` approximates, `date/mini.js:60-63`).
    pub(crate) fn with_wall(&self, wall: civil::DateTime) -> DateValue {
        DateValue {
            millis: wall_in_zone_millis(wall, &self.zone),
            zone: self.zone.clone(),
        }
    }

    // ── JS `Date.prototype` wall-clock setters ────────────────────────────────────────
    //
    // The TZDate setters write the internal (target-zone wall clock) fields and resync
    // the instant (`date/mini.js:124-129`); a plain `Date`'s setters do the same through
    // the ambient zone. Both collapse to: rebuild the wall clock with JS rollover
    // semantics, then re-resolve the instant in the value's zone. The port's
    // `wall_in_zone_millis` disambiguation (gaps forward, folds earlier) is the
    // `adjustToSystemTZ` stand-in.

    /// `setFullYear(year, monthIndex, day)` — month/day overflow rolls over.
    pub(crate) fn set_wall_ymd(&self, year: i64, month_index: i64, day: i64) -> DateValue {
        let wall = self.wall();
        self.rebuilt_wall(year, month_index, day, ms_of_day(&wall))
    }

    /// `setFullYear(year)` — the month/day stay (except Feb 29 rolling into Mar 1).
    pub(crate) fn set_wall_year(&self, year: i64) -> DateValue {
        let wall = self.wall();
        self.rebuilt_wall(year, i64::from(wall.month()) - 1, i64::from(wall.day()), ms_of_day(&wall))
    }

    /// `setDate(day)` — overflow rolls into the next/previous months.
    pub(crate) fn set_wall_day(&self, day: i64) -> DateValue {
        let wall = self.wall();
        self.rebuilt_wall(i64::from(wall.year()), i64::from(wall.month()) - 1, day, ms_of_day(&wall))
    }

    /// `setMonth(monthIndex, day)` — both components roll over (date-fns pre-clamps the
    /// day before reaching this).
    pub(crate) fn set_wall_month(&self, month_index: i64, day: i64) -> DateValue {
        let wall = self.wall();
        self.rebuilt_wall(i64::from(wall.year()), month_index, day, ms_of_day(&wall))
    }

    /// `setHours(h, m, s, ms)` — each field rolls over (25h is the next day 01:00,
    /// -1h is the previous day 23:00).
    pub(crate) fn set_wall_time(&self, hour: i64, minute: i64, second: i64, millis: i64) -> DateValue {
        let wall = self.wall();
        self.rebuilt_wall(
            i64::from(wall.year()),
            i64::from(wall.month()) - 1,
            i64::from(wall.day()),
            hour * 3_600_000 + minute * 60_000 + second * 1_000 + millis,
        )
    }

    /// `setMinutes(m, s, ms)` — the hour of day stays, the fields roll over.
    pub(crate) fn set_wall_minutes(&self, minute: i64, second: i64, millis: i64) -> DateValue {
        let wall = self.wall();
        let ms_of_day = ms_of_day(&wall);
        let hour = ms_of_day.div_euclid(3_600_000);
        self.rebuilt_wall(
            i64::from(wall.year()),
            i64::from(wall.month()) - 1,
            i64::from(wall.day()),
            hour * 3_600_000 + minute * 60_000 + second * 1_000 + millis,
        )
    }

    /// `setSeconds(s, ms)` — hour and minute stay, the fields roll over.
    pub(crate) fn set_wall_seconds(&self, second: i64, millis: i64) -> DateValue {
        let wall = self.wall();
        let ms_of_day = ms_of_day(&wall);
        let hour = ms_of_day.div_euclid(3_600_000);
        let minute = ms_of_day.rem_euclid(3_600_000).div_euclid(60_000);
        self.rebuilt_wall(
            i64::from(wall.year()),
            i64::from(wall.month()) - 1,
            i64::from(wall.day()),
            hour * 3_600_000 + minute * 60_000 + second * 1_000 + millis,
        )
    }

    /// `setMilliseconds(ms)` — the field is REPLACED (the sub-second within the current
    /// second); overflow rolls over (`setMilliseconds(1250)` is the next second +250ms).
    pub(crate) fn set_wall_milliseconds(&self, millis: i64) -> DateValue {
        let wall = self.wall();
        let day_millis = ms_of_day(&wall);
        let day_millis = day_millis - day_millis.rem_euclid(1_000) + millis;
        self.rebuilt_wall(
            i64::from(wall.year()),
            i64::from(wall.month()) - 1,
            i64::from(wall.day()),
            day_millis,
        )
    }

    /// Wall-clock rebuild with JS rollover: the civil date rolls over through
    /// [`civil_rollover`], then the (possibly overflowing) milliseconds of the day roll
    /// across midnight boundaries.
    fn rebuilt_wall(&self, year: i64, month_index: i64, day: i64, ms_of_day: i64) -> DateValue {
        let (year, month, day) = civil_rollover(year, month_index, day);
        let day_shift = ms_of_day.div_euclid(86_400_000);
        let (year, month, day) = if day_shift != 0 {
            civil_rollover(year, month - 1, day + day_shift)
        } else {
            (year, month, day)
        };
        let rem = ms_of_day.rem_euclid(86_400_000);
        let wall = civil::DateTime::new(
            year as i16,
            month as i8,
            day as i8,
            (rem / 3_600_000) as i8,
            (rem.rem_euclid(3_600_000) / 60_000) as i8,
            (rem.rem_euclid(60_000) / 1_000) as i8,
            (rem.rem_euclid(1_000) * 1_000_000) as i32,
        )
        .expect("rebuilt wall clock in range");
        self.with_wall(wall)
    }
}

impl From<&Zone> for Zone {
    fn from(value: &Zone) -> Self {
        value.clone()
    }
}

impl From<&str> for Zone {
    fn from(value: &str) -> Self {
        Zone::Named(value.to_string())
    }
}

/// `jiff`'s `Timestamp` from raw epoch milliseconds (its constructor validates; the
/// values this crate produces are always in range).
pub(crate) fn timestamp(millis: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(millis).expect("in-range epoch milliseconds")
}

/// The UTC offset in milliseconds at an instant for a zone, in the sign convention
/// date-fns' `getTimezoneOffsetInMilliseconds` computes (`+date - +utcDate`, i.e.
/// negative east of UTC — the value the calendar-day difference math subtracts).
pub(crate) fn offset_millis(zone: &Zone, millis: i64) -> i64 {
    -i64::from(millis_zoned_in_static(millis, zone).offset().seconds()) * 1000
}

fn millis_zoned_in_static(millis: i64, zone: &Zone) -> Zoned {
    let tz = resolve_zone(zone).expect("DateValue zone is validated at construction");
    timestamp(millis).to_zoned(tz)
}

/// Resolves wall-clock fields into an instant for a zone (the TZDate multi-arg
/// constructor's sync-from-internal, `date/mini.js:55-60`).
pub(crate) fn wall_in_zone_millis(wall: civil::DateTime, zone: &Zone) -> i64 {
    let tz = resolve_zone(zone).expect("DateValue zone is validated at construction");
    wall_in_resolved_zone_millis(wall, &tz)
}

/// The resolved-zone half of [`wall_in_zone_millis`] — for callers that already hold the
/// [`TimeZone`] and need an invalid-zone *failure* (the `None` of the adapter seam) instead
/// of a panic (`temporal_adapter_date_fns`'s `date`/`parse`, whose upstream constructor
/// yields Invalid Date for an unresolvable zone).
pub(crate) fn wall_in_resolved_zone_millis(wall: civil::DateTime, tz: &TimeZone) -> i64 {
    // jiff's default disambiguation is "compatible": gaps shift forward, folds keep the
    // earlier offset — the JS `Date` behavior for out-of-range wall clocks.
    tz.to_ambiguous_zoned(wall)
        .compatible()
        .expect("wall clock within the representable range")
        .timestamp()
        .as_millisecond()
}

thread_local! {
    /// The test-visible system-zone override (`process.env.TZ` stand-in — see the module
    /// docs). `None` defers to `jiff`'s system zone.
    static SYSTEM_ZONE_OVERRIDE: RefCell<Option<TimeZone>> = const { RefCell::new(None) };
}

/// Overrides the zone `Zone::System` resolves to. The `process.env.TZ` seam upstream
/// tests use (`TemporalAdapterDateFns.test.ts:17-26`); `None` restores the ambient zone.
pub fn set_system_zone(zone: Option<TimeZone>) {
    SYSTEM_ZONE_OVERRIDE.with_borrow_mut(|slot| *slot = zone);
}

/// The ambient system zone (the plain `Date` wall-clock zone).
pub fn system_zone() -> TimeZone {
    SYSTEM_ZONE_OVERRIDE
        .with_borrow(|slot| slot.clone())
        .unwrap_or_else(TimeZone::system)
}

/// Resolves a zone name the way `@date-fns/tz` does: IANA names (and `'UTC'`) through the
/// database (`tzOffset`'s `Intl.DateTimeFormat` path), then the fixed-offset fallback
/// forms `±HH(:MM(:SS)?)?` with an optional `UTC` prefix (the `offsetRe` path,
/// `tz/tzOffset/index.js:66-69`).
pub(crate) fn resolve_zone(zone: &Zone) -> Option<TimeZone> {
    match zone {
        Zone::System => Some(system_zone()),
        Zone::Named(name) => resolve_zone_name(name),
    }
}

/// The `Zone::Named` resolution half of [`resolve_zone`].
pub(crate) fn resolve_zone_name(name: &str) -> Option<TimeZone> {
    if let Ok(tz) = TimeZone::get(name) {
        return Some(tz);
    }
    parse_fixed_offset(name).map(TimeZone::fixed)
}

/// The fixed-offset forms the upstream fallback regex accepts — an unanchored
/// `([+-]\d\d):?(\d\d)?` match anywhere in the string, so a `UTC`-prefixed form like
/// `'UTC-05:30'` matches at `'-05:30'` (`tz/tzOffset/index.js:66`). `calcOffset`
/// (`:71-77`) keeps the sign in the hours value and decides which way the minutes
/// contribute by the sign of `hours * 60 + minutes`, so `'-05:30'` is `-330` minutes
/// while `'-00:30'` reads as `+30`; the regex has no minute-range validation, so
/// `'+03:99'` resolves to `279` minutes the same way.
pub(crate) fn parse_fixed_offset(name: &str) -> Option<Offset> {
    // `name.match(offsetRe)` — unanchored, so the leftmost position where the FULL
    // pattern matches wins (`Foo-2+05:30` matches `+05:30`, not the stray `-`).
    for start in 0..name.len() {
        if let Some(offset) = match_at(name, start) {
            return Some(offset);
        }
    }
    None
}

/// One attempt of the `([+-]\d\d):?(\d\d)?` regex at `start`, with its backtracking:
/// a complete colon+minutes or bare-minutes group is preferred, and a partial group
/// (e.g. `+05:x`, `+050`) falls back to matching the hours alone.
fn match_at(name: &str, start: usize) -> Option<Offset> {
    let bytes = &name.as_bytes()[start..];
    // `([+-]\d\d)` — sign plus two hour digits.
    if bytes.len() < 3 || (bytes[0] != b'+' && bytes[0] != b'-') || !bytes[1].is_ascii_digit() || !bytes[2].is_ascii_digit()
    {
        return None;
    }
    let minutes_str = if bytes.len() >= 6
        && bytes[3] == b':'
        && bytes[4].is_ascii_digit()
        && bytes[5].is_ascii_digit()
    {
        Some(&name[start + 4..start + 6])
    } else if bytes.len() >= 5 && bytes[3].is_ascii_digit() && bytes[4].is_ascii_digit() {
        Some(&name[start + 3..start + 5])
    } else {
        None
    };
    // `calcOffset(timeZone, captures.slice(1))`: `+(values[0] || 0)` — the hours capture
    // already carries the sign — and `+(values[1] || 0)` for the minutes. The seconds
    // capture only exists in the `Intl` path's split, never in the regex, so it is 0.
    let hours: i64 = name[start..start + 3].parse().ok()?;
    let minutes: i64 = match minutes_str {
        Some(m) => m.parse().ok()?,
        None => 0,
    };
    // `hours * 60 + minutes > 0 ? hours * 60 + minutes + seconds : hours * 60 - minutes
    // - seconds` — the sign lives in the hours value, the minutes never do. The result
    // is a minute count; the fixed offset is that many seconds.
    let total_minutes = if hours * 60 + minutes > 0 {
        hours * 60 + minutes
    } else {
        hours * 60 - minutes
    };
    Offset::from_seconds(i32::try_from(total_minutes * 60).ok()?).ok()
}

/// The wall clock's milliseconds since local midnight.
pub(crate) fn ms_of_day(wall: &civil::DateTime) -> i64 {
    i64::from(wall.hour()) * 3_600_000
        + i64::from(wall.minute()) * 60_000
        + i64::from(wall.second()) * 1_000
        + i64::from(wall.millisecond())
}

/// Days since the Unix epoch for a proleptic-Gregorian civil date/// (Howard Hinnant's `days_from_civil`). `month` must sit in its canonical 1-12 range
/// (the callers normalize first — [`civil_rollover`] owns the JS `Date` rollover
/// semantics for out-of-range components).
pub(crate) fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_index = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_index + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146097 + day_of_era - 719468
}

/// The inverse of [`days_from_civil`] (Howard Hinnant's `civil_from_days`): the
/// proleptic-Gregorian civil date for a day number relative to the Unix epoch.
pub(crate) fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let day_of_era = z - era * 146097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36524 - day_of_era / 146096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

/// The number of days in a proleptic-Gregorian month (any year; `month` 1-12).
pub(crate) fn days_in_civil_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if year % 400 == 0 || (year % 4 == 0 && year % 100 != 0) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

/// The JS `Date(year, monthIndex, day)` rollover for arbitrary component values: the
/// month overflow carries into the year first, then the day overflow carries through the
/// (already-carried) month's real length — `setDate(0)` is the last day of the previous
/// month, `new Date(2020, 12, 1)` is Jan 2021, `setFullYear` on Feb 29 into a non-leap
/// year lands Mar 1.
pub(crate) fn civil_rollover(year: i64, month_index: i64, day: i64) -> (i64, i64, i64) {
    // Month overflow: JS `Date(2020, 12, …)` = `(2021, 0, …)`; `Date(2020, -1, …)` =
    // `(2019, 11, …)`. `Local::floor_div`-style Euclidean division keeps negatives right.
    let year = year + month_index.div_euclid(12);
    let month_index = month_index.rem_euclid(12);
    // Day overflow: day 1 of (year, month) plus `day - 1` absolute days.
    let days = days_from_civil(year, month_index + 1, 1) + (day - 1);
    let (y, m, d) = civil_from_days(days);
    (y, m, d)
}

/// The `TemporalAdapter` interface
/// (`packages/react/src/internals/temporal/temporal-adapter.ts:100-382`), over the
/// crate's [`DateValue`] vocabulary (see the module docs for the type-vocabulary
/// adaptation). Method order follows the upstream interface.
pub trait TemporalAdapter {
    /// Whether the adapter's date objects carry timezone information. `true` for both
    /// upstream adapters (`TemporalAdapterDateFns.ts:103`, `TemporalAdapterLuxon.ts:51`).
    fn is_timezone_compatible(&self) -> bool {
        true
    }

    /// The format table (`TemporalAdapterFormats`).
    fn formats(&self) -> &'static TemporalAdapterFormats;

    /// Name of the library in use (`TemporalAdapterDateFns.ts:105`).
    fn lib(&self) -> &'static str;

    /// Characters used to escape a string inside a format (`:111`).
    fn escaped_characters(&self) -> EscapedCharacters;

    /// Creates a date from an ISO string in the given timezone, or `None` for the null
    /// input or a failed parse / unresolvable zone — the port's Invalid Date collapse
    /// (`:125-165`, `date/mini.js:36-39`).
    fn date(&self, value: Option<&str>, timezone: &TemporalTimezone) -> Option<DateValue>;

    /// Parses a date from a string in the given format (`:167-188`); a failed parse is
    /// `None` (upstream's Invalid Date).
    fn parse(&self, value: &str, format: &str, timezone: &TemporalTimezone)
        -> Option<DateValue>;

    /// Creates a date for the current time in the given timezone (`:117-123`); `None`
    /// for an unresolvable zone (upstream's Invalid Date, `date/mini.js:36-39`).
    fn now(&self, timezone: &TemporalTimezone) -> Option<DateValue>;

    /// Extracts the timezone from a date (`:190-196`).
    fn get_timezone(&self, value: &DateValue) -> TemporalTimezone;

    /// Converts a date to another timezone (`:198-211`). `None` for an unresolvable
    /// target zone — upstream's `new TZDate(value, timezone)` yields Invalid Date there
    /// (`date/mini.js:36-39`).
    fn set_timezone(
        &self,
        value: &DateValue,
        timezone: &TemporalTimezone,
    ) -> Option<DateValue>;

    /// Converts a date into a plain JS `Date`-shaped value (`:213-218`).
    fn to_js_date(&self, value: &DateValue) -> DateValue;

    /// Gets the code of the locale currently used by the adapter (`:220-222`).
    fn get_current_locale_code(&self) -> &'static str;

    /// Checks if the date is valid (`:224-230`).
    fn is_valid(&self, value: Option<&DateValue>) -> bool;

    /// Formats a date using an adapter format key (`:232-234`).
    fn format(&self, value: &DateValue, format_key: TemporalFormatKey) -> String;

    /// Formats a date using a format of the date library (`:236-238`).
    fn format_by_string(&self, value: &DateValue, format_string: &str) -> String;

    /// Checks if the two dates are equal (same timestamp); `null == null` is true
    /// (`:240-250`).
    fn is_equal(&self, value: Option<&DateValue>, comparing: Option<&DateValue>) -> bool;

    /// Checks if the two dates are in the same year — both wall clocks computed in the
    /// value's zone, the date-fns `normalizeDates` context (`:252-254`,
    /// `_lib/normalizeDates.js` — the first object argument's context wins).
    fn is_same_year(&self, value: &DateValue, comparing: &DateValue) -> bool;

    /// Checks if the two dates are in the same month (both in the value's zone,
    /// `:256-258`).
    fn is_same_month(&self, value: &DateValue, comparing: &DateValue) -> bool;

    /// Checks if the two dates are in the same day (both in the value's zone,
    /// `:260-262`).
    fn is_same_day(&self, value: &DateValue, comparing: &DateValue) -> bool;

    /// Checks if the two dates are at the same hour (both in the value's zone,
    /// `:264-266`).
    fn is_same_hour(&self, value: &DateValue, comparing: &DateValue) -> bool;

    /// Checks if the `value` date is after the `comparing` date (`:268-270`).
    fn is_after(&self, value: &DateValue, comparing: &DateValue) -> bool;

    /// Checks if the `value` date is before the `comparing` date (`:272-274`).
    fn is_before(&self, value: &DateValue, comparing: &DateValue) -> bool;

    /// Checks if the value is within the provided (closed, endpoint-sorted) range
    /// (`:276-278`, `date-fns/isWithinInterval.js`).
    fn is_within_range(&self, value: &DateValue, range: (&DateValue, &DateValue)) -> bool;

    /// Returns the start of the year for the given date (`:280-282`).
    fn start_of_year(&self, value: &DateValue) -> DateValue;

    /// Returns the start of the month for the given date (`:284-286`).
    fn start_of_month(&self, value: &DateValue) -> DateValue;

    /// Returns the start of the week for the given date, per the locale's
    /// `weekStartsOn` (`:288-291`).
    fn start_of_week(&self, value: &DateValue) -> DateValue;

    /// Returns the start of the day for the given date (`:292-294`).
    fn start_of_day(&self, value: &DateValue) -> DateValue;

    /// Returns the start of the hour for the given date (`:296-298`).
    fn start_of_hour(&self, value: &DateValue) -> DateValue;

    /// Returns the start of the minute for the given date (`:300-302`).
    fn start_of_minute(&self, value: &DateValue) -> DateValue;

    /// Returns the start of the second for the given date (`:304-306`).
    fn start_of_second(&self, value: &DateValue) -> DateValue;

    /// Returns the end of the year for the given date (`:308-311`).
    fn end_of_year(&self, value: &DateValue) -> DateValue;

    /// Returns the end of the month for the given date (`:313-315`).
    fn end_of_month(&self, value: &DateValue) -> DateValue;

    /// Returns the end of the week for the given date, per the locale's
    /// `weekStartsOn` (`:317-320`).
    fn end_of_week(&self, value: &DateValue) -> DateValue;

    /// Returns the end of the day for the given date (`:322-324`).
    fn end_of_day(&self, value: &DateValue) -> DateValue;

    /// Returns the end of the hour for the given date (`:326-328`).
    fn end_of_hour(&self, value: &DateValue) -> DateValue;

    /// Returns the end of the minute for the given date (`:330-332`).
    fn end_of_minute(&self, value: &DateValue) -> DateValue;

    /// Returns the end of the second for the given date (`:334-336`).
    fn end_of_second(&self, value: &DateValue) -> DateValue;

    /// Adds the specified number of years to the given date (calendar arithmetic with
    /// day clamping, `:336-338`).
    fn add_years(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Adds the specified number of months to the given date (`:340-342`).
    fn add_months(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Adds the specified number of weeks to the given date (`:344-346`).
    fn add_weeks(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Adds the specified number of days to the given date (`:348-350`).
    fn add_days(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Adds the specified number of hours to the given date (`:352-354`).
    fn add_hours(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Adds the specified number of minutes to the given date (`:356-358`).
    fn add_minutes(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Adds the specified number of seconds to the given date (`:360-362`).
    fn add_seconds(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Adds the specified number of milliseconds to the given date (`:364-366`).
    fn add_milliseconds(&self, value: &DateValue, amount: i64) -> DateValue;

    /// Gets the year of the given date (`:368-370`).
    fn get_year(&self, value: &DateValue) -> i64;

    /// Gets the month of the given date — 0-based, January = 0 (`:372-374`).
    fn get_month(&self, value: &DateValue) -> i64;

    /// Gets the date (day in the month) of the given date (`:376-378`).
    fn get_date(&self, value: &DateValue) -> i64;

    /// Gets the hours of the given date (`:380-382`).
    fn get_hours(&self, value: &DateValue) -> i64;

    /// Gets the minutes of the given date (`:384-386`).
    fn get_minutes(&self, value: &DateValue) -> i64;

    /// Gets the seconds of the given date (`:388-390`).
    fn get_seconds(&self, value: &DateValue) -> i64;

    /// Gets the milliseconds of the given date (`:392-394`).
    fn get_milliseconds(&self, value: &DateValue) -> i64;

    /// Gets the time since epoch of the given date (`:396-398`).
    fn get_time(&self, value: &DateValue) -> i64;

    /// Sets the year to the given date (JS `setFullYear` rollover semantics) (`:400-402`).
    fn set_year(&self, value: &DateValue, year: i64) -> DateValue;

    /// Sets the month to the given date (day clamped to the target month's length,
    /// `:404-406`).
    fn set_month(&self, value: &DateValue, month: i64) -> DateValue;

    /// Sets the date (day in the month) to the given date (JS `setDate` rollover
    /// semantics) (`:408-410`).
    fn set_date(&self, value: &DateValue, date: i64) -> DateValue;

    /// Sets the hours to the given date (JS `setHours` rollover semantics) (`:412-414`).
    fn set_hours(&self, value: &DateValue, hours: i64) -> DateValue;

    /// Sets the minutes to the given date (`:416-418`).
    fn set_minutes(&self, value: &DateValue, minutes: i64) -> DateValue;

    /// Sets the seconds to the given date (`:420-422`).
    fn set_seconds(&self, value: &DateValue, seconds: i64) -> DateValue;

    /// Sets the milliseconds to the given date (`:424-426`).
    fn set_milliseconds(&self, value: &DateValue, milliseconds: i64) -> DateValue;

    /// Gets the number of full years between the given dates (`:428-430`).
    fn difference_in_years(&self, value: &DateValue, comparing: &DateValue) -> i64;

    /// Gets the number of full months between the given dates (`:432-434`).
    fn difference_in_months(&self, value: &DateValue, comparing: &DateValue) -> i64;

    /// Gets the number of full weeks between the given dates (`:436-438`).
    fn difference_in_weeks(&self, value: &DateValue, comparing: &DateValue) -> i64;

    /// Gets the number of full days between the given dates (`:440-442`).
    fn difference_in_days(&self, value: &DateValue, comparing: &DateValue) -> i64;

    /// Gets the number of full hours between the given dates (`:444-446`).
    fn difference_in_hours(&self, value: &DateValue, comparing: &DateValue) -> i64;

    /// Gets the number of full minutes between the given dates (`:448-450`).
    fn difference_in_minutes(&self, value: &DateValue, comparing: &DateValue) -> i64;

    /// Gets the number of days in a month of the given date (`:452-454`).
    fn get_days_in_month(&self, value: &DateValue) -> i64;

    /// Gets the number of the week of the given date, per the locale's week-numbering
    /// rules (`:456-458`).
    fn get_week_number(&self, value: &DateValue) -> i64;

    /// Gets the number of the day of the week of the given date — 1-based, per the
    /// locale's `weekStartsOn` (`:460-463`).
    fn get_day_of_week(&self, value: &DateValue) -> i64;
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    // The helper-level pins: zone resolution (`@date-fns/tz`'s accepted forms) and the
    // JS `Date` component-rollover arithmetic the adapters build their setters on.

    #[test]
    fn resolves_iana_zones_and_utc() {
        assert!(resolve_zone_name("America/Sao_Paulo").is_some());
        assert!(resolve_zone_name("UTC").is_some());
        assert!(resolve_zone_name("Not/A_Zone").is_none());
    }

    #[test]
    fn resolves_fixed_offset_forms() {
        let five_thirty = resolve_zone_name("-05:30").expect("-05:30");
        assert_eq!(timestamp(0).to_zoned(five_thirty).to_string(), "1969-12-31T18:30:00-05:30[-05:30]");
        let four = resolve_zone_name("+0400").expect("+0400");
        assert_eq!(
            timestamp(0).to_zoned(four).offset().seconds(),
            4 * 3600
        );
        let utc_minus_three = resolve_zone_name("UTC-03:00").expect("UTC-03:00");
        assert_eq!(timestamp(0).to_zoned(utc_minus_three).offset().seconds(), -3 * 3600);
        // `calcOffset`'s sign quirk: the hour-part decides the branch, so a negative
        // zero-hour with positive minutes reads as positive.
        assert_eq!(
            timestamp(0).to_zoned(resolve_zone_name("-00:30").expect("-00:30")).offset().seconds(),
            30 * 60
        );
        assert!(resolve_zone_name("-5:00").is_none());
        // The fallback regex has no minute-range validation — `+03:99` resolves through
        // `calcOffset` to `3 * 60 + 99 = 279` minutes.
        assert_eq!(
            timestamp(0).to_zoned(resolve_zone_name("+03:99").expect("+03:99")).offset().seconds(),
            279 * 60
        );
    }

    #[test]
    fn civil_rollover_matches_js_date_component_overflow() {
        // `new Date(2020, 12, 1)` = Jan 2021.
        assert_eq!(civil_rollover(2020, 12, 1), (2021, 1, 1));
        // `new Date(2020, -1, 15)` = Dec 2019.
        assert_eq!(civil_rollover(2020, -1, 15), (2019, 12, 15));
        // `setDate(0)` = last day of the previous month (day 0 of March, index 2).
        assert_eq!(civil_rollover(2024, 2, 0), (2024, 2, 29));
        // Feb 29 in a non-leap target year rolls into March (`setFullYear` on a Feb 29
        // date — month index 1 — re-set to 2023).
        assert_eq!(civil_rollover(2023, 1, 29), (2023, 3, 1));
    }

    #[test]
    fn system_zone_override_controls_zone_system() {
        let sao = TimeZone::get("America/Sao_Paulo").expect("zone");
        set_system_zone(Some(sao));
        let value = DateValue::from_millis(0);
        assert_eq!(value.wall().to_string(), "1969-12-31T21:00:00");
        set_system_zone(None);
    }
}
