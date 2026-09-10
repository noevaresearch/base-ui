//! Port of `packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts`
//! — the registered `TemporalAdapter` implementation
//! (TODO.md item `infra: internals`, the temporal-adapter checkpoint).
//!
//! Upstream is a thin class over `date-fns` + `@date-fns/tz`'s `TZDate`: `now`/`date`/
//! `parse` build values through the JS `Date` string parser and the TZDate multi-arg
//! constructor, `setTimezone`/`toJsDate` move values between the plain-`Date` and
//! TZDate worlds, and every other method forwards to the same-named date-fns function
//! (ported in [`crate::date_fns_calendar`], [`crate::date_fns_format`],
//! [`crate::date_fns_parse`] over the [`crate::temporal::DateValue`] vocabulary).
//!
//! The `date()` parsing rule the behavior spec pins
//! (`TemporalAdapterDateFns.ts:136-139`, `TemporalAdapterDateFns.test.ts:15-48`): the
//! JS spec parses date-only strings (`"2026-04-06"`, no `T`) as **UTC midnight**, so the
//! face-value components are extracted with UTC getters and rebuilt in the target zone;
//! datetime strings parse as **local time** (or the given offset), so the local getters
//! are correct. `date()` must therefore validate strictly like `new Date` does
//! (`"2018-42-30T11:60:00.000Z"` is an Invalid Date,
//! `testComputations.ts:222-229`), which is [`parse_js_date_string`]'s range checks.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - **The constructor locale is a table reference.** Upstream's `locale ?? enUS`
//!   (`:113-115`) becomes [`TemporalAdapterDateFns::new`] /
//!   [`TemporalAdapterDateFns::with_locale`] over the embedded
//!   [`DateFnsLocale`] tables — the open-ended JS locale bag collapses to the two
//!   transcribed locales (see the `date_fns_locale` module docs).
//! - **`setTimezone`'s duck typing is N/A.** `canChangeTz = typeof value?.withTimeZone
//!   === 'function'` (`:198-211`) exists for cross-library-copy `TZDate` interop
//!   (`TemporalAdapterDateFns.test.ts:50-63`); the port has exactly one value type, so
//!   the `withTimeZone` arm is the unconditional path (a zone re-attachment of the same
//!   instant) and the `new TZDate(value, timezone)` fallback is unreachable.
//! - **`isValid` is structural.** date-fns `isValid` checks NaN time; the port's
//!   `DateValue` cannot carry NaN (failed parses/zones are `None` at the trait
//!   boundary), so `isValid(Some) == true` and the `null → false` pin remains.
//! - **The unresolvable-zone paths return `None`** where upstream produces an Invalid
//!   Date (`TZDateMini`'s `tzOffset` NaN → NaN time, `date/mini.js:36-39`) — the
//!   documented Invalid Date collapse, so `isValid` downstream observes the same state.
//! - **The JS string parser is the ES ISO grammar** plus V8's space separator and
//!   lowercase `t`; V8's legacy lenient formats (month names, `2020-1-1`, …) are not
//!   ported — the adapter contract is fed ISO strings by the components and the shared
//!   harness (`TEST_DATE_ISO_STRING`/`TEST_DATE_LOCALE_STRING`).
//!
//! **The Luxon adapter is deliberately not ported.** `TemporalAdapterLuxon.ts` is
//! `@ts-nocheck` and its module-augmentation registration is commented out
//! (`TemporalAdapterLuxon.ts:44-48`), so no component can bind to it upstream; its
//! methods are thin wrappers over the luxon engine (`DateTime.fromISO/fromFormat/
//! toFormat/localWeekNumber`), and the Rust workspace carries no luxon-equivalent
//! engine — porting the adapter would mean porting luxon itself. The registered
//! adapter upstream is the date-fns one (`TemporalAdapterDateFns.ts:96-100`), which
//! this module ports. Logged in `ralph/logs/spec-discrepancies.md`.

use jiff::tz::TimeZone;
use jiff::{civil, Timestamp};

use crate::date_fns_calendar::{
    add_days, add_hours, add_milliseconds, add_minutes, add_months, add_seconds, add_weeks,
    add_years, difference_in_days, difference_in_hours, difference_in_minutes,
    difference_in_months, difference_in_weeks, difference_in_years, end_of_day, end_of_hour,
    end_of_minute, end_of_month, end_of_second, end_of_week, end_of_year, get_day,
    get_days_in_month, get_week, project_to_value_zone, set_date, set_month, set_year,
    start_of_day, start_of_hour, start_of_minute, start_of_month, start_of_second, start_of_week,
    start_of_year,
};
use crate::date_fns_locale::DateFnsLocale;
use crate::temporal::{
    days_in_civil_month, resolve_zone_name, wall_in_resolved_zone_millis, wall_in_zone_millis,
    DateValue, EscapedCharacters, TemporalAdapter, TemporalAdapterFormats, TemporalFormatKey,
    TemporalTimezone, Zone,
};

/// The adapter's static format table (`TemporalAdapterDateFns.ts:68-94`).
static DATE_FNS_FORMATS: TemporalAdapterFormats = TemporalAdapterFormats {
    year_padded: "yyyy",
    month_padded: "MM",
    day_of_month_padded: "dd",
    hours24h_padded: "HH",
    hours12h_padded: "hh",
    minutes_padded: "mm",
    seconds_padded: "ss",
    day_of_month: "d",
    hours24h: "H",
    hours12h: "h",
    month3_letters: "MMM",
    month_full_letter: "MMMM",
    weekday: "EEEE",
    weekday3_letters: "EEE",
    weekday1_letter: "EEEEE",
    meridiem: "a",
    localized_date_with_full_month_and_week_day: "PPPP",
    localized_numeric_date: "P",
};

/// The `TemporalAdapterDateFns` class (`TemporalAdapterDateFns.ts:102-115`).
#[derive(Clone, Copy)]
pub struct TemporalAdapterDateFns {
    locale: &'static DateFnsLocale,
}

impl Default for TemporalAdapterDateFns {
    fn default() -> Self {
        Self::new()
    }
}

impl TemporalAdapterDateFns {
    /// `new TemporalAdapterDateFns()` — the `enUS` default locale (`:114`).
    pub fn new() -> TemporalAdapterDateFns {
        TemporalAdapterDateFns { locale: &crate::date_fns_locale::EN_US }
    }

    /// `new TemporalAdapterDateFns({ locale })` (`:113-115`).
    pub fn with_locale(locale: &'static DateFnsLocale) -> TemporalAdapterDateFns {
        TemporalAdapterDateFns { locale }
    }

    /// The adapter's locale (the private `this.locale`).
    pub fn locale(&self) -> &'static DateFnsLocale {
        self.locale
    }
}

impl TemporalAdapter for TemporalAdapterDateFns {
    fn formats(&self) -> &'static TemporalAdapterFormats {
        &DATE_FNS_FORMATS
    }

    fn lib(&self) -> &'static str {
        "date-fns"
    }

    fn escaped_characters(&self) -> EscapedCharacters {
        EscapedCharacters { start: "'", end: "'" }
    }

    /// `now` (`:117-123`): the current instant, attached to the target zone for the
    /// non-system arms (`TZDate.tz(timezone)`).
    fn now(&self, timezone: &TemporalTimezone) -> Option<DateValue> {
        let millis = now_millis();
        if timezone.is_system() {
            return Some(DateValue::from_millis(millis));
        }
        resolve_zone_name(timezone.as_str())?;
        Some(DateValue {
            millis,
            zone: Zone::Named(timezone.as_str().to_string()),
        })
    }

    /// `date` (`:125-165`) — see the module docs for the date-only rule.
    fn date(&self, value: Option<&str>, timezone: &TemporalTimezone) -> Option<DateValue> {
        let value = value?;
        let millis = parse_js_date_string(value)?;
        let is_date_only = !value.contains('T');

        // The face-value components: UTC getters for date-only strings (parsed as UTC
        // midnight), local (system) getters for datetime strings.
        let wall: civil::DateTime = if is_date_only {
            utc_wall(millis)
        } else {
            DateValue::from_millis(millis).wall()
        };

        if timezone.is_system() {
            if is_date_only {
                // `new Date(utcFullYear, utcMonth, utcDate)` — local midnight of the
                // face-value date.
                return Some(DateValue {
                    millis: wall_in_zone_millis(
                        civil::DateTime::new(
                            wall.year(),
                            wall.month(),
                            wall.day(),
                            0,
                            0,
                            0,
                            0,
                        )
                        .expect("face-value wall clock in range"),
                        &Zone::System,
                    ),
                    zone: Zone::System,
                });
            }
            return Some(DateValue::from_millis(millis));
        }

        // `new TZDate(components…, timezone)` — the face-value wall clock rebuilt in the
        // target zone; an unresolvable zone is upstream's Invalid Date → `None`.
        let tz = resolve_zone_name(timezone.as_str())?;
        Some(DateValue {
            millis: wall_in_resolved_zone_millis(wall, &tz),
            zone: Zone::Named(timezone.as_str().to_string()),
        })
    }

    /// `parse` (`:167-188`): the date-fns parser against a system-zone "now" reference,
    /// then the parsed wall clock rebuilt in the target zone for the non-system arms.
    fn parse(
        &self,
        value: &str,
        format: &str,
        timezone: &TemporalTimezone,
    ) -> Option<DateValue> {
        let reference = DateValue::from_millis(now_millis());
        let parsed = crate::date_fns_parse::parse(value, format, &reference, self.locale)?;
        if timezone.is_system() {
            return Some(parsed);
        }
        let tz = resolve_zone_name(timezone.as_str())?;
        Some(DateValue {
            millis: wall_in_resolved_zone_millis(parsed.wall(), &tz),
            zone: Zone::Named(timezone.as_str().to_string()),
        })
    }

    /// `getTimezone` (`:190-196`): the TZDate zone name, or `'system'` for plain dates.
    fn get_timezone(&self, value: &DateValue) -> TemporalTimezone {
        match value.zone() {
            Zone::System => TemporalTimezone::system_tz(),
            Zone::Named(name) => TemporalTimezone::from(name.as_str()),
        }
    }

    /// `setTimezone` (`:198-211`): the system arms strip to a plain date; the named arms
    /// re-attach the zone (the `withTimeZone` shape) — `None` for an unresolvable zone.
    fn set_timezone(
        &self,
        value: &DateValue,
        timezone: &TemporalTimezone,
    ) -> Option<DateValue> {
        if timezone.is_system() {
            return Some(self.to_js_date(value));
        }
        resolve_zone_name(timezone.as_str())?;
        Some(DateValue {
            millis: value.millis(),
            zone: Zone::Named(timezone.as_str().to_string()),
        })
    }

    /// `toJsDate` (`:213-218`): the plain-`Date` twin — the same instant, system zone.
    fn to_js_date(&self, value: &DateValue) -> DateValue {
        DateValue {
            millis: value.millis(),
            zone: Zone::System,
        }
    }

    fn get_current_locale_code(&self) -> &'static str {
        self.locale.code
    }

    /// `isValid` (`:224-230`): `null → false`, then date-fns `isValid` — structural in
    /// the port (see the module docs).
    fn is_valid(&self, value: Option<&DateValue>) -> bool {
        value.is_some()
    }

    /// `format` (`:232-234`).
    fn format(&self, value: &DateValue, format_key: TemporalFormatKey) -> String {
        self.format_by_string(value, self.formats().get(format_key))
    }

    /// `formatByString` (`:236-238`).
    fn format_by_string(&self, value: &DateValue, format_string: &str) -> String {
        crate::date_fns_format::format(value, format_string, self.locale)
    }

    /// `isEqual` (`:240-250`).
    fn is_equal(&self, value: Option<&DateValue>, comparing: Option<&DateValue>) -> bool {
        match (value, comparing) {
            (None, None) => true,
            (Some(value), Some(comparing)) => value.millis() == comparing.millis(),
            _ => false,
        }
    }

    /// `isSameYear` (`:252-254`) — `normalizeDates` projects the second date into the
    /// first argument's zone (see the `date_fns_calendar` module docs).
    fn is_same_year(&self, value: &DateValue, comparing: &DateValue) -> bool {
        let comparing = project_to_value_zone(value, comparing);
        value.wall().year() == comparing.wall().year()
    }

    /// `isSameMonth` (`:256-258`).
    fn is_same_month(&self, value: &DateValue, comparing: &DateValue) -> bool {
        let comparing = project_to_value_zone(value, comparing);
        let value_wall = value.wall();
        let comparing_wall = comparing.wall();
        value_wall.year() == comparing_wall.year()
            && value_wall.month() == comparing_wall.month()
    }

    /// `isSameDay` (`:260-262`).
    fn is_same_day(&self, value: &DateValue, comparing: &DateValue) -> bool {
        let comparing = project_to_value_zone(value, comparing);
        start_of_day(value).millis() == start_of_day(&comparing).millis()
    }

    /// `isSameHour` (`:264-266`).
    fn is_same_hour(&self, value: &DateValue, comparing: &DateValue) -> bool {
        let comparing = project_to_value_zone(value, comparing);
        start_of_hour(value).millis() == start_of_hour(&comparing).millis()
    }

    /// `isAfter` (`:268-270`).
    fn is_after(&self, value: &DateValue, comparing: &DateValue) -> bool {
        value.millis() > comparing.millis()
    }

    /// `isBefore` (`:272-274`).
    fn is_before(&self, value: &DateValue, comparing: &DateValue) -> bool {
        value.millis() < comparing.millis()
    }

    /// `isWithinRange` (`:276-278`) — `isWithinInterval` sorts the endpoints and
    /// compares inclusively (`date-fns/isWithinInterval.js`).
    fn is_within_range(&self, value: &DateValue, range: (&DateValue, &DateValue)) -> bool {
        let (start, end) = if range.0.millis() <= range.1.millis() {
            (range.0.millis(), range.1.millis())
        } else {
            (range.1.millis(), range.0.millis())
        };
        value.millis() >= start && value.millis() <= end
    }

    fn start_of_year(&self, value: &DateValue) -> DateValue {
        start_of_year(value)
    }

    fn start_of_month(&self, value: &DateValue) -> DateValue {
        start_of_month(value)
    }

    /// `startOfWeek` (`:288-291`) — the locale's `weekStartsOn`.
    fn start_of_week(&self, value: &DateValue) -> DateValue {
        start_of_week(value, self.locale.week_starts_on)
    }

    fn start_of_day(&self, value: &DateValue) -> DateValue {
        start_of_day(value)
    }

    fn start_of_hour(&self, value: &DateValue) -> DateValue {
        start_of_hour(value)
    }

    fn start_of_minute(&self, value: &DateValue) -> DateValue {
        start_of_minute(value)
    }

    fn start_of_second(&self, value: &DateValue) -> DateValue {
        start_of_second(value)
    }

    fn end_of_year(&self, value: &DateValue) -> DateValue {
        end_of_year(value)
    }

    fn end_of_month(&self, value: &DateValue) -> DateValue {
        end_of_month(value)
    }

    /// `endOfWeek` (`:316-318`).
    fn end_of_week(&self, value: &DateValue) -> DateValue {
        end_of_week(value, self.locale.week_starts_on)
    }

    fn end_of_day(&self, value: &DateValue) -> DateValue {
        end_of_day(value)
    }

    fn end_of_hour(&self, value: &DateValue) -> DateValue {
        end_of_hour(value)
    }

    fn end_of_minute(&self, value: &DateValue) -> DateValue {
        end_of_minute(value)
    }

    fn end_of_second(&self, value: &DateValue) -> DateValue {
        end_of_second(value)
    }

    fn add_years(&self, value: &DateValue, amount: i64) -> DateValue {
        add_years(value, amount)
    }

    fn add_months(&self, value: &DateValue, amount: i64) -> DateValue {
        add_months(value, amount)
    }

    fn add_weeks(&self, value: &DateValue, amount: i64) -> DateValue {
        add_weeks(value, amount)
    }

    fn add_days(&self, value: &DateValue, amount: i64) -> DateValue {
        add_days(value, amount)
    }

    fn add_hours(&self, value: &DateValue, amount: i64) -> DateValue {
        add_hours(value, amount)
    }

    fn add_minutes(&self, value: &DateValue, amount: i64) -> DateValue {
        add_minutes(value, amount)
    }

    fn add_seconds(&self, value: &DateValue, amount: i64) -> DateValue {
        add_seconds(value, amount)
    }

    fn add_milliseconds(&self, value: &DateValue, amount: i64) -> DateValue {
        add_milliseconds(value, amount)
    }

    fn get_year(&self, value: &DateValue) -> i64 {
        i64::from(value.wall().year())
    }

    /// `getMonth` (`:372-374`) — 0-based, January = 0.
    fn get_month(&self, value: &DateValue) -> i64 {
        i64::from(value.wall().month()) - 1
    }

    fn get_date(&self, value: &DateValue) -> i64 {
        i64::from(value.wall().day())
    }

    fn get_hours(&self, value: &DateValue) -> i64 {
        i64::from(value.wall().hour())
    }

    fn get_minutes(&self, value: &DateValue) -> i64 {
        i64::from(value.wall().minute())
    }

    fn get_seconds(&self, value: &DateValue) -> i64 {
        i64::from(value.wall().second())
    }

    fn get_milliseconds(&self, value: &DateValue) -> i64 {
        i64::from(value.wall().millisecond())
    }

    fn get_time(&self, value: &DateValue) -> i64 {
        value.millis()
    }

    /// `setYear` (`:400-402`) — the JS `setFullYear` rollover (Feb 29 → Mar 1).
    fn set_year(&self, value: &DateValue, year: i64) -> DateValue {
        set_year(value, year)
    }

    /// `setMonth` (`:404-406`) — the day clamps to the target month's length.
    fn set_month(&self, value: &DateValue, month: i64) -> DateValue {
        set_month(value, month)
    }

    fn set_date(&self, value: &DateValue, date: i64) -> DateValue {
        set_date(value, date)
    }

    /// `setHours` (`:412-414`) — the JS rollover semantics.
    fn set_hours(&self, value: &DateValue, hours: i64) -> DateValue {
        let wall = value.wall();
        value.set_wall_time(
            hours,
            i64::from(wall.minute()),
            i64::from(wall.second()),
            i64::from(wall.millisecond()),
        )
    }

    fn set_minutes(&self, value: &DateValue, minutes: i64) -> DateValue {
        let wall = value.wall();
        value.set_wall_minutes(
            minutes,
            i64::from(wall.second()),
            i64::from(wall.millisecond()),
        )
    }

    fn set_seconds(&self, value: &DateValue, seconds: i64) -> DateValue {
        let wall = value.wall();
        value.set_wall_seconds(seconds, i64::from(wall.millisecond()))
    }

    fn set_milliseconds(&self, value: &DateValue, milliseconds: i64) -> DateValue {
        value.set_wall_milliseconds(milliseconds)
    }

    fn difference_in_years(&self, value: &DateValue, comparing: &DateValue) -> i64 {
        difference_in_years(value, comparing)
    }

    fn difference_in_months(&self, value: &DateValue, comparing: &DateValue) -> i64 {
        difference_in_months(value, comparing)
    }

    fn difference_in_weeks(&self, value: &DateValue, comparing: &DateValue) -> i64 {
        difference_in_weeks(value, comparing)
    }

    fn difference_in_days(&self, value: &DateValue, comparing: &DateValue) -> i64 {
        difference_in_days(value, comparing)
    }

    fn difference_in_hours(&self, value: &DateValue, comparing: &DateValue) -> i64 {
        difference_in_hours(value, comparing)
    }

    fn difference_in_minutes(&self, value: &DateValue, comparing: &DateValue) -> i64 {
        difference_in_minutes(value, comparing)
    }

    fn get_days_in_month(&self, value: &DateValue) -> i64 {
        get_days_in_month(value)
    }

    /// `getWeekNumber` (`:456-458`) — `getWeek(value, { locale })`.
    fn get_week_number(&self, value: &DateValue) -> i64 {
        get_week(
            value,
            self.locale.week_starts_on,
            self.locale.first_week_contains_date,
        )
    }

    /// `getDayOfWeek` (`:460-463`) — `((getDay + 7 - weekStartsOn) % 7) + 1`, 1-based.
    fn get_day_of_week(&self, value: &DateValue) -> i64 {
        ((get_day(value) + 7 - self.locale.week_starts_on) % 7) + 1
    }
}

// Re-used by the harness-shaped tests below.

/// The current wall-clock instant — `Date.now()`.
pub(crate) fn now_millis() -> i64 {
    Timestamp::now().as_millisecond()
}

/// The wall clock of an instant in UTC.
fn utc_wall(millis: i64) -> civil::DateTime {
    crate::temporal::timestamp(millis)
        .to_zoned(TimeZone::UTC)
        .datetime()
}

/// The JS `new Date(string)` parser — the ES `Date Time String Format` grammar plus
/// V8's space separator and lowercase `t` (see the module docs for the unsupported
/// legacy formats). Returns `None` where JS produces an Invalid Date.
///
/// - `YYYY` | `YYYY-MM` | `YYYY-MM-DD` — UTC (midnight for the date-only forms).
/// - `YYYY-MM-DD[T ]HH:mm(:ss(.fff)?)?(Z|±HH(:mm(:ss)?)?|±HHmm|±HH)?` — local time
///   without an offset, the given instant with one.
/// - Out-of-range components (`2018-42-30`, `11:60:00`) are rejected, as the ISO
///   branch of the JS spec requires (`testComputations.ts:222-229`).
pub(crate) fn parse_js_date_string(value: &str) -> Option<i64> {
    let chars: Vec<char> = value.chars().collect();
    // The date part: `YYYY`, `YYYY-MM`, or `YYYY-MM-DD` (digits and dashes only).
    let date_len = match chars.len() {
        0 => return None,
        n if n >= 10 => 10,
        7 => 7,
        4 => 4,
        _ => return None,
    };
    let date = &chars[..date_len];
    let digit = |c: char| c.is_ascii_digit();
    let number = |slice: &[char]| -> Option<i64> { slice.iter().collect::<String>().parse().ok() };
    if !(date[0].is_ascii_digit()
        && date[1].is_ascii_digit()
        && date[2].is_ascii_digit()
        && date[3].is_ascii_digit())
    {
        return None;
    }
    let year = number(&date[..4])?;
    let month = if date_len >= 7 {
        if date[4] != '-' || !(digit(date[5]) && digit(date[6])) {
            return None;
        }
        number(&date[5..7])?
    } else {
        1
    };
    let day = if date_len >= 10 {
        if date[7] != '-' || !(digit(date[8]) && digit(date[9])) {
            return None;
        }
        number(&date[8..10])?
    } else {
        1
    };
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    if day > days_in_civil_month(year, month) {
        return None;
    }

    // No time part: UTC (midnight for the date-only forms).
    if chars.len() == date_len {
        return Some(
            civil::DateTime::new(year as i16, month as i8, day as i8, 0, 0, 0, 0)
                .ok()?
                .to_zoned(TimeZone::UTC)
                .ok()?
                .timestamp()
                .as_millisecond(),
        );
    }

    // The separator: `T`, `t`, or V8's space.
    if !matches!(chars[date_len], 'T' | 't' | ' ') {
        return None;
    }
    let time = &chars[date_len + 1..];
    if time.len() < 3 || !(digit(time[0]) && digit(time[1]) && time[2] == ':') {
        return None;
    }
    let hour = number(&time[..2])?;
    let mut i = 3;
    let read_two = |time: &[char], i: &mut usize| -> Option<i64> {
        if *i + 1 < time.len() && digit(time[*i]) && digit(time[*i + 1]) {
            let n = number(&time[*i..*i + 2])?;
            *i += 2;
            Some(n)
        } else {
            None
        }
    };
    let minute = read_two(time, &mut i)?;
    let mut second = 0;
    let mut millis = 0;
    if i < time.len() && time[i] == ':' {
        i += 1;
        second = read_two(time, &mut i)?;
        if i < time.len() && time[i] == '.' {
            i += 1;
            let start = i;
            while i < time.len() && digit(time[i]) {
                i += 1;
            }
            if i == start {
                return None;
            }
            let fraction: String = time[start..i.min(start + 3)].iter().collect();
            millis = fraction.parse::<i64>().ok()? * 10i64.pow(3 - fraction.len() as u32);
        }
    }
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }

    // The optional offset.
    let offset_millis = if i >= time.len() {
        None
    } else {
        match time[i] {
            'Z' | 'z' => {
                i += 1;
                Some(0)
            }
            '+' | '-' => {
                let sign: i64 = if time[i] == '+' { 1 } else { -1 };
                i += 1;
                let hours = read_two(time, &mut i)?;
                let mut minutes = 0;
                let mut seconds = 0;
                if i < time.len() && time[i] == ':' {
                    i += 1;
                    minutes = read_two(time, &mut i)?;
                    if i < time.len() && time[i] == ':' {
                        i += 1;
                        seconds = read_two(time, &mut i)?;
                    }
                } else if i + 1 < time.len() && digit(time[i]) && digit(time[i + 1]) {
                    // `±HHmm`
                    minutes = number(&time[i..i + 2])?;
                    i += 2;
                }
                if hours > 23 || minutes > 59 || seconds > 59 {
                    return None;
                }
                Some(sign * (hours * 3_600_000 + minutes * 60_000 + seconds * 1_000))
            }
            _ => return None,
        }
    };
    // Nothing may follow the offset.
    if i != time.len() {
        return None;
    }

    let wall = civil::DateTime::new(
        year as i16,
        month as i8,
        day as i8,
        hour as i8,
        minute as i8,
        second as i8,
        (millis * 1_000_000) as i32,
    )
    .ok()?;
    match offset_millis {
        None => Some(wall_in_zone_millis(wall, &Zone::System)),
        Some(offset) => Some(
            wall.to_zoned(TimeZone::UTC)
                .ok()?
                .timestamp()
                .as_millisecond()
                - offset,
        ),
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::date_fns_locale::{EN_US, FR};
    use crate::temporal::set_system_zone;

    const TEST_DATE_ISO: &str = "2018-10-30T11:44:25.750Z";
    const TEST_DATE_ISO_MILLIS: i64 = 1_540_899_865_750;
    const FIXTURE_ISO: &str = "2020-01-01T15:08:09.000Z";

    fn adapter() -> TemporalAdapterDateFns {
        TemporalAdapterDateFns::new()
    }

    fn with_utc_zone<T>(run: impl FnOnce() -> T) -> T {
        set_system_zone(Some(TimeZone::UTC));
        let out = run();
        set_system_zone(None);
        out
    }

    #[test]
    fn preserves_the_date_for_date_only_strings_in_a_negative_offset() {
        // TemporalAdapterDateFns.test.ts:23-31 — `America/Sao_Paulo` (UTC-3).
        let sao = TimeZone::get("America/Sao_Paulo").expect("zone");
        set_system_zone(Some(sao));
        let result = adapter().date(Some("2026-04-06"), &TemporalTimezone::system_tz()).expect("date");
        assert_eq!(adapter().get_year(&result), 2026);
        assert_eq!(adapter().get_month(&result), 3);
        assert_eq!(adapter().get_date(&result), 6);
        set_system_zone(None);
    }

    #[test]
    fn preserves_the_date_for_date_only_strings_in_a_named_timezone() {
        // TemporalAdapterDateFns.test.ts:33-40.
        let sao = TimeZone::get("America/Sao_Paulo").expect("zone");
        set_system_zone(Some(sao));
        let result = adapter()
            .date(Some("2026-04-06"), &TemporalTimezone::from("America/Sao_Paulo"))
            .expect("date");
        assert_eq!(adapter().get_year(&result), 2026);
        assert_eq!(adapter().get_month(&result), 3);
        assert_eq!(adapter().get_date(&result), 6);
        assert_eq!(adapter().get_hours(&result), 0);
        set_system_zone(None);
    }

    #[test]
    fn still_uses_local_getters_for_datetime_strings() {
        // TemporalAdapterDateFns.test.ts:42-47.
        let sao = TimeZone::get("America/Sao_Paulo").expect("zone");
        set_system_zone(Some(sao));
        let result = adapter().date(Some("2026-04-06T14:30:00"), &TemporalTimezone::system_tz()).expect("date");
        assert_eq!(adapter().get_hours(&result), 14);
        assert_eq!(adapter().get_minutes(&result), 30);
        set_system_zone(None);
    }

    #[test]
    fn parses_iso_strings_into_every_timezone_shape() {
        // testComputations.ts:37-69 — `toEqualDateTime` compares the WALL CLOCK in the
        // value's zone (`addVitestMatchers.ts:29-31`), so every zone arm keeps the
        // string's face-value wall clock.
        with_utc_zone(|| {
            for timezone in ["UTC", "system", "America/New_York", "Europe/Paris"] {
                let value = adapter()
                    .date(Some(TEST_DATE_ISO), &TemporalTimezone::from(timezone))
                    .expect("date");
                let wall = value.wall();
                assert_eq!(
                    format!(
                        "{}-{}-{}T{}:{}:{}.{}",
                        wall.year(),
                        wall.month(),
                        wall.day(),
                        wall.hour(),
                        wall.minute(),
                        wall.second(),
                        wall.millisecond()
                    ),
                    "2018-10-30T11:44:25.750"
                );
                assert_eq!(adapter().get_timezone(&value).as_str(), timezone);
            }
            // The UTC arm additionally keeps the timestamp.
            let utc = adapter().date(Some(TEST_DATE_ISO), &TemporalTimezone::utc()).expect("date");
            assert_eq!(utc.millis(), TEST_DATE_ISO_MILLIS);
        });
    }

    #[test]
    fn rejects_invalid_component_ranges_like_the_js_spec() {
        // testComputations.ts:222-229 — `2018-42-30T11:60:00.000Z` is an Invalid Date.
        let invalid = adapter().date(Some("2018-42-30T11:60:00.000Z"), &TemporalTimezone::default_tz());
        assert!(invalid.is_none());
        assert!(!adapter().is_valid(None));
        assert!(adapter()
            .date(Some(TEST_DATE_ISO), &TemporalTimezone::default_tz())
            .is_some_and(|v| adapter().is_valid(Some(&v))));
    }

    #[test]
    fn converts_timezones_without_impacting_the_timestamp() {
        // testComputations.ts:165-184.
        with_utc_zone(|| {
            let value = adapter().date(Some(TEST_DATE_ISO), &TemporalTimezone::system_tz()).expect("date");
            for timezone in ["America/New_York", "Europe/Paris", "Australia/Sydney", "UTC"] {
                let converted = adapter()
                    .set_timezone(&value, &TemporalTimezone::from(timezone))
                    .expect("resolvable zone");
                assert_eq!(adapter().get_timezone(&converted).as_str(), timezone);
                assert_eq!(converted.millis(), value.millis());
            }
        });
    }

    #[test]
    fn set_timezone_system_strips_to_the_plain_date() {
        with_utc_zone(|| {
            let value = adapter().date(Some(TEST_DATE_ISO), &TemporalTimezone::from("UTC")).expect("date");
            let stripped = adapter().set_timezone(&value, &TemporalTimezone::default_tz()).expect("date");
            assert_eq!(stripped.zone(), &Zone::System);
            assert_eq!(stripped.millis(), value.millis());
            let stripped = adapter().to_js_date(&value);
            assert_eq!(stripped.zone(), &Zone::System);
        });
    }

    #[test]
    fn now_returns_the_current_instant_in_the_requested_zone() {
        with_utc_zone(|| {
            for timezone in ["system", "UTC", "America/New_York"] {
                let value = adapter().now(&TemporalTimezone::from(timezone)).expect("now");
                assert_eq!(adapter().get_timezone(&value).as_str(), timezone);
                assert!((value.millis() - now_millis()).abs() < 5_000);
            }
            // An unresolvable zone is the port's Invalid Date.
            assert!(adapter().now(&TemporalTimezone::from("Not/A_Zone")).is_none());
        });
    }

    #[test]
    fn formats_through_the_adapter_keys_in_the_requested_case() {
        // testFormats.ts:14-31, with the harness's lowercase 'utc' zone.
        with_utc_zone(|| {
            let value = adapter().date(Some(FIXTURE_ISO), &TemporalTimezone::from("utc")).expect("date");
            assert_eq!(adapter().get_timezone(&value).as_str(), "utc");
            let f = |key| adapter().format(&value, key);
            assert_eq!(f(TemporalFormatKey::YearPadded), "2020");
            assert_eq!(f(TemporalFormatKey::Hours24hPadded), "15");
            assert_eq!(f(TemporalFormatKey::Hours12hPadded), "03");
            assert_eq!(f(TemporalFormatKey::Weekday), "Wednesday");
            assert_eq!(f(TemporalFormatKey::Meridiem), "PM");
            assert_eq!(f(TemporalFormatKey::LocalizedNumericDate), "01/01/2020");
        });
    }

    #[test]
    fn computes_the_harness_arithmetic_matrix() {
        with_utc_zone(|| {
            let value = adapter().date(Some(TEST_DATE_ISO), &TemporalTimezone::default_tz()).expect("date");
            let iso = |s: &str| adapter().date(Some(s), &TemporalTimezone::default_tz()).expect("date");
            let eq = |value: &DateValue, expected: &str| {
                assert_eq!(
                    value.millis(),
                    adapter().date(Some(expected), &TemporalTimezone::default_tz()).expect("date").millis(),
                    "{expected}"
                );
            };
            // startOf*/endOf* (testComputations.ts:477-564, runner TZ = UTC).
            eq(&adapter().start_of_year(&value), "2018-01-01T00:00:00.000Z");
            eq(&adapter().start_of_month(&value), "2018-10-01T00:00:00.000Z");
            eq(&adapter().start_of_week(&value), "2018-10-28T00:00:00.000Z");
            eq(&adapter().start_of_day(&value), "2018-10-30T00:00:00.000Z");
            eq(&adapter().start_of_hour(&value), "2018-10-30T11:00:00.000Z");
            eq(&adapter().end_of_year(&value), "2018-12-31T23:59:59.999Z");
            eq(&adapter().end_of_month(&value), "2018-10-31T23:59:59.999Z");
            eq(&adapter().end_of_week(&value), "2018-11-03T23:59:59.999Z");
            // add* (testComputations.ts:566-630).
            eq(&adapter().add_years(&value, 2), "2020-10-30T11:44:25.750Z");
            eq(&adapter().add_months(&value, 3), "2019-01-30T11:44:25.750Z");
            eq(&adapter().add_weeks(&value, 2), "2018-11-13T11:44:25.750Z");
            eq(&adapter().add_days(&value, 2), "2018-11-01T11:44:25.750Z");
            eq(&adapter().add_hours(&value, 15), "2018-10-31T02:44:25.750Z");
            eq(&adapter().add_minutes(&value, 20), "2018-10-30T12:04:25.750Z");
            eq(&adapter().add_seconds(&value, 70), "2018-10-30T11:45:35.750Z");
            eq(&adapter().add_milliseconds(&value, 500), "2018-10-30T11:44:26.250Z");
            // get*/set* (testComputations.ts:632-690).
            assert_eq!(adapter().get_year(&value), 2018);
            assert_eq!(adapter().get_month(&value), 9);
            assert_eq!(adapter().get_time(&value), TEST_DATE_ISO_MILLIS);
            eq(&adapter().set_year(&value, 2011), "2011-10-30T11:44:25.750Z");
            eq(&adapter().set_month(&value, 4), "2018-05-30T11:44:25.750Z");
            eq(&adapter().set_date(&value, 15), "2018-10-15T11:44:25.750Z");
            eq(&adapter().set_hours(&value, 0), "2018-10-30T00:44:25.750Z");
            eq(&adapter().set_milliseconds(&value, 11), "2018-10-30T11:44:25.011Z");
            // differences (testComputations.ts:692-843).
            assert_eq!(adapter().difference_in_years(&iso("2020-04-01"), &iso("2018-04-01")), 2);
            assert_eq!(adapter().difference_in_years(&iso("2020-04-01"), &iso("2018-10-30")), 1);
            assert_eq!(adapter().difference_in_months(&iso("2019-01-30"), &iso("2018-10-30")), 3);
            assert_eq!(adapter().difference_in_months(&iso("2019-01-15"), &iso("2018-10-30")), 2);
            assert_eq!(adapter().difference_in_days(&iso("2018-11-05"), &iso("2018-10-30")), 6);
            assert_eq!(adapter().difference_in_hours(&iso("2018-10-31T15:00"), &iso("2018-10-30T11:00")), 28);
            assert_eq!(adapter().difference_in_minutes(&iso("2018-10-30T12:30"), &iso("2018-10-30T11:00")), 90);
            assert_eq!(adapter().get_days_in_month(&value), 31);
            assert_eq!(adapter().get_days_in_month(&adapter().add_months(&value, 1)), 30);
            // week numbers + day of week (testComputations.ts:851-857).
            assert_eq!(adapter().get_week_number(&value), 44);
            assert_eq!(adapter().get_day_of_week(&value), 3);
        });
    }

    #[test]
    fn computes_the_cross_timezone_matrix_in_the_first_arguments_zone() {
        // testComputations.ts:709-843 — the Paris/NY pins.
        let paris = TemporalTimezone::from("Europe/Paris");
        let ny = TemporalTimezone::from("America/New_York");
        let a = adapter();
        let diff_years = a.difference_in_years(
            &a.date(Some("2020-04-01T12:00"), &paris).expect("date"),
            &a.date(Some("2018-04-01T12:00"), &ny).expect("date"),
        );
        assert_eq!(diff_years, 1);
        let diff_months = a.difference_in_months(
            &a.date(Some("2018-06-30T12:00"), &paris).expect("date"),
            &a.date(Some("2018-04-30T12:00"), &ny).expect("date"),
        );
        assert_eq!(diff_months, 1);
        let diff_days = a.difference_in_days(
            &a.date(Some("2018-10-07T12:00"), &paris).expect("date"),
            &a.date(Some("2018-10-05T12:00"), &ny).expect("date"),
        );
        assert_eq!(diff_days, 1);
        let diff_hours = a.difference_in_hours(
            &a.date(Some("2018-10-30T12:00"), &paris).expect("date"),
            &a.date(Some("2018-10-30T12:00"), &ny).expect("date"),
        );
        assert_eq!(diff_hours, -5);
        let diff_minutes = a.difference_in_minutes(
            &a.date(Some("2018-10-30T12:00"), &paris).expect("date"),
            &a.date(Some("2018-10-30T12:00"), &ny).expect("date"),
        );
        assert_eq!(diff_minutes, -300);
        // Across DST: the CET spring-forward hour.
        let dst_hours = a.difference_in_hours(
            &a.date(Some("2022-03-28"), &paris).expect("date"),
            &a.date(Some("2022-03-27"), &paris).expect("date"),
        );
        assert_eq!(dst_hours, 23);
        assert_eq!(dst_hours * 60, a.difference_in_minutes(
            &a.date(Some("2022-03-28"), &paris).expect("date"),
            &a.date(Some("2022-03-27"), &paris).expect("date"),
        ));
    }

    #[test]
    fn is_same_and_range_and_equality_follow_the_harness() {
        with_utc_zone(|| {
            let a = adapter();
            let value = a.date(Some(TEST_DATE_ISO), &TemporalTimezone::default_tz()).expect("date");
            let iso = |s: &str| a.date(Some(s), &TemporalTimezone::default_tz()).expect("date");
            // isSame* (testComputations.ts:252-358).
            assert!(a.is_same_year(&value, &iso("2018-10-01T00:00:00.000Z")));
            assert!(!a.is_same_year(&value, &iso("2019-10-01T00:00:00.000Z")));
            assert!(a.is_same_month(&value, &iso("2018-10-01T00:00:00.000Z")));
            assert!(a.is_same_day(&value, &iso("2018-10-30T00:00:00.000Z")));
            assert!(!a.is_same_day(&value, &iso("2019-10-30T00:00:00.000Z")));
            assert!(a.is_same_hour(&value, &iso("2018-10-30T11:00:00.000Z")));
            assert!(!a.is_same_hour(&value, &iso("2018-10-30T12:00:00.000Z")));
            // The same instant in different zones counts as the same calendar unit.
            let london = a.end_of_year(&a.set_timezone(&value, &TemporalTimezone::from("Europe/London")).expect("tz"));
            let paris = a.set_timezone(&london, &TemporalTimezone::from("Europe/Paris")).expect("tz");
            assert!(a.is_same_year(&london, &paris));
            assert!(a.is_same_year(&paris, &london));
            // isEqual (testComputations.ts:231-249).
            assert!(a.is_equal(None, None));
            assert!(!a.is_equal(Some(&value), None));
            assert!(a.is_equal(Some(&value), Some(&a.date(Some(TEST_DATE_ISO), &TemporalTimezone::default_tz()).expect("date"))));
            let in_london = a.set_timezone(&value, &TemporalTimezone::from("Europe/London")).expect("tz");
            let in_paris = a.set_timezone(&value, &TemporalTimezone::from("Europe/Paris")).expect("tz");
            assert!(a.is_equal(Some(&in_london), Some(&in_paris)));
            // isWithinRange (testComputations.ts:406-474).
            assert!(a.is_within_range(&iso("2019-10-01"), (&iso("2019-09-01"), &iso("2019-11-01"))));
            assert!(!a.is_within_range(&iso("2019-12-01"), (&iso("2019-09-01"), &iso("2019-11-01"))));
            assert!(a.is_within_range(&iso("2019-09-01"), (&iso("2019-09-01"), &iso("2019-12-01"))));
            assert!(a.is_within_range(&iso("2019-12-01"), (&iso("2019-09-01"), &iso("2019-12-01"))));
            // The fr-locale date in a plain-locale range (testComputations.ts:467-474).
            let fr = TemporalAdapterDateFns::with_locale(&FR);
            assert!(a.is_within_range(
                &iso("2022-04-17"),
                (
                    &fr.date(Some("2022-04-17"), &TemporalTimezone::default_tz()).expect("date"),
                    &fr.date(Some("2022-04-19"), &TemporalTimezone::default_tz()).expect("date"),
                ),
            ));
        });
    }

    #[test]
    fn parses_through_the_adapter_and_rezones() {
        with_utc_zone(|| {
            // testComputations.ts:104-117 — the RFC5545 custom format:
            // `yyyyMMdd'T'HHmmss'Z'` (the harness builds it from the formats table).
            let f = adapter().formats();
            let format = format!(
                "{}{}{}'T'{}{}{}'Z'",
                f.year_padded, f.month_padded, f.day_of_month_padded, f.hours24h_padded,
                f.minutes_padded, f.seconds_padded
            );
            let parsed = adapter()
                .parse("20181030T114400Z", &format, &TemporalTimezone::default_tz())
                .expect("parses");
            assert_eq!(parsed.millis(), 1_540_899_840_000); // 2018-10-30T11:44:00Z in UTC
            // The non-system arms rebuild the parsed wall clock in the target zone.
            let parsed = adapter()
                .parse("2020-06-15 14:30", "yyyy-MM-dd HH:mm", &TemporalTimezone::from("UTC"))
                .expect("parses");
            assert_eq!(adapter().get_timezone(&parsed).as_str(), "UTC");
            assert_eq!(adapter().get_hours(&parsed), 14);
            // A failed parse is the port's Invalid Date.
            assert!(adapter().parse("not-a-date", "yyyy-MM-dd", &TemporalTimezone::default_tz()).is_none());
        });
    }

    #[test]
    fn carries_the_locale_dependent_behavior() {
        with_utc_zone(|| {
            let fr = TemporalAdapterDateFns::with_locale(&FR);
            assert_eq!(fr.get_current_locale_code(), "fr");
            assert_eq!(adapter().get_current_locale_code(), "en-US");
            let value = fr.date(Some(TEST_DATE_ISO), &TemporalTimezone::from("utc")).expect("date");
            assert_eq!(fr.format(&value, TemporalFormatKey::Weekday), "mardi");
            assert_eq!(adapter().format(&value, TemporalFormatKey::Weekday), "Tuesday");
            // The fr week starts Monday: Oct 30 2018 belongs to week 44 either way,
            // but the week boundaries differ — 2018-01-01 is week 1 in en-US and
            // week 1 in fr too, so pin the en-US-only week 1 Sunday vs fr Monday.
            let jan1 = adapter().date(Some("2018-01-01"), &TemporalTimezone::from("utc")).expect("date");
            assert_eq!(adapter().get_day_of_week(&jan1), 2); // Monday, Sunday-start week
            assert_eq!(fr.get_day_of_week(&jan1), 1); // Monday, Monday-start week
        });
    }
}
