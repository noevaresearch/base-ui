//! The date-fns calendar functions the temporal adapter, formatter, and parser share —
//! direct ports of the `date-fns@4.4.0` sources the upstream adapter calls
//! (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:2-59`
//! imports). Each function cites its upstream file.
//!
//! Rust adaptation: the two-argument date-fns functions take an options bag carrying the
//! locale's `weekStartsOn`/`firstWeekContainsDate`; here they are plain parameters the
//! adapter passes from its [`DateFnsLocale`](crate::date_fns_locale::DateFnsLocale).
//! `normalizeDates` (`date-fns/_lib/normalizeDates.js`) re-constructs every argument in
//! the first date's own zone (the `constructDateFrom` TZDate bridge,
//! `@date-fns/tz/date/mini.js:104-106`); in the port that projection is a plain
//! zone re-attachment of the same instant — [`DateValue::millis`] with the first
//! argument's [`Zone`] — since the port's setters compute wall clocks in the value's
//! zone directly.

use jiff::civil;

use crate::temporal::{civil_rollover, days_from_civil, wall_in_zone_millis, DateValue, Zone};

const MS_IN_MINUTE: i64 = 60_000;
const MS_IN_HOUR: i64 = 3_600_000;
const MS_IN_DAY: i64 = 86_400_000;
const MS_IN_WEEK: i64 = 604_800_000;

/// Re-attaches `other`'s instant to `value`'s zone — the `normalizeDates` context rule
/// (the first object argument's zone wins).
pub(crate) fn project_to_value_zone(value: &DateValue, other: &DateValue) -> DateValue {
    DateValue {
        millis: other.millis,
        zone: value.zone.clone(),
    }
}

/// `date-fns/startOfDay.js` — the wall clock at local midnight.
pub(crate) fn start_of_day(value: &DateValue) -> DateValue {
    value.set_wall_time(0, 0, 0, 0)
}

/// `date-fns/endOfDay.js`.
pub(crate) fn end_of_day(value: &DateValue) -> DateValue {
    value.set_wall_time(23, 59, 59, 999)
}

/// `date-fns/startOfHour.js`.
pub(crate) fn start_of_hour(value: &DateValue) -> DateValue {
    let wall = value.wall();
    value.set_wall_time(i64::from(wall.hour()), 0, 0, 0)
}

/// `date-fns/endOfHour.js`.
pub(crate) fn end_of_hour(value: &DateValue) -> DateValue {
    let wall = value.wall();
    value.set_wall_time(i64::from(wall.hour()), 59, 59, 999)
}

/// `date-fns/startOfMinute.js`.
pub(crate) fn start_of_minute(value: &DateValue) -> DateValue {
    let wall = value.wall();
    value.set_wall_time(i64::from(wall.hour()), i64::from(wall.minute()), 0, 0)
}

/// `date-fns/endOfMinute.js`.
pub(crate) fn end_of_minute(value: &DateValue) -> DateValue {
    let wall = value.wall();
    value.set_wall_time(i64::from(wall.hour()), i64::from(wall.minute()), 59, 999)
}

/// `date-fns/startOfSecond.js`.
pub(crate) fn start_of_second(value: &DateValue) -> DateValue {
    let wall = value.wall();
    value.set_wall_time(
        i64::from(wall.hour()),
        i64::from(wall.minute()),
        i64::from(wall.second()),
        0,
    )
}

/// `date-fns/endOfSecond.js`.
pub(crate) fn end_of_second(value: &DateValue) -> DateValue {
    let wall = value.wall();
    value.set_wall_time(
        i64::from(wall.hour()),
        i64::from(wall.minute()),
        i64::from(wall.second()),
        999,
    )
}

/// `date-fns/startOfYear.js`.
pub(crate) fn start_of_year(value: &DateValue) -> DateValue {
    value.set_wall_ymd(i64::from(value.wall().year()), 0, 1)
}

/// `date-fns/endOfYear.js` — `endOfMonth(startOfMonth? …)`: Dec 31 23:59:59.999.
pub(crate) fn end_of_year(value: &DateValue) -> DateValue {
    let year = i64::from(value.wall().year());
    let last = last_day_of_month_wall(year, 11);
    value
        .set_wall_ymd(year, 11, last)
        .set_wall_time(23, 59, 59, 999)
}

/// `date-fns/startOfMonth.js`.
pub(crate) fn start_of_month(value: &DateValue) -> DateValue {
    let wall = value.wall();
    value.set_wall_ymd(i64::from(wall.year()), i64::from(wall.month()) - 1, 1)
}

/// `date-fns/endOfMonth.js`.
pub(crate) fn end_of_month(value: &DateValue) -> DateValue {
    let wall = value.wall();
    let last = last_day_of_month_wall(i64::from(wall.year()), i64::from(wall.month()) - 1);
    value
        .set_wall_ymd(i64::from(wall.year()), i64::from(wall.month()) - 1, last)
        .set_wall_time(23, 59, 59, 999)
}

/// `date-fns/startOfWeek.js` — `diff = (day < weekStartsOn ? 7 : 0) + day -
/// weekStartsOn`, then `setDate(getDate() - diff)` + midnight.
pub(crate) fn start_of_week(value: &DateValue, week_starts_on: i64) -> DateValue {
    let day = get_day(value);
    let diff = if day < week_starts_on { 7 } else { 0 } + day - week_starts_on;
    let wall = value.wall();
    value
        .set_wall_day(i64::from(wall.day()) - diff)
        .set_wall_time(0, 0, 0, 0)
}

/// `date-fns/endOfWeek.js` — `diff = (day < weekStartsOn ? -7 : 0) + 6 -
/// (day - weekStartsOn)`, then `setDate(getDate() + diff)` + 23:59:59.999.
pub(crate) fn end_of_week(value: &DateValue, week_starts_on: i64) -> DateValue {
    let day = get_day(value);
    let diff = if day < week_starts_on { -7 } else { 0 } + 6 - (day - week_starts_on);
    let wall = value.wall();
    value
        .set_wall_day(i64::from(wall.day()) + diff)
        .set_wall_time(23, 59, 59, 999)
}

/// `date-fns/startOfISOWeek.js` — `startOfWeek(date, { weekStartsOn: 1 })`.
pub(crate) fn start_of_iso_week(value: &DateValue) -> DateValue {
    start_of_week(value, 1)
}

/// The JS `Date.prototype.getDay` — 0 = Sunday … 6 = Saturday, wall clock in the
/// value's zone.
pub(crate) fn get_day(value: &DateValue) -> i64 {
    (i64::from(value.wall().date().weekday().to_monday_zero_offset()) + 1) % 7
}

/// The wall clock as `(year, monthIndex, day)`.
fn wall_ymd(value: &DateValue) -> (i64, i64, i64) {
    let wall = value.wall();
    (i64::from(wall.year()), i64::from(wall.month()) - 1, i64::from(wall.day()))
}

/// The last wall-clock day of `(year, monthIndex)` — `constructFrom(…, 0)` +
/// `setFullYear(year, month + 1, 0)` (`date-fns/getDaysInMonth.js`).
fn last_day_of_month_wall(year: i64, month_index: i64) -> i64 {
    civil_rollover(year, month_index + 1, 0).2
}

/// `date-fns/getDaysInMonth.js` — `getDaysInMonth(date)`.
pub(crate) fn get_days_in_month(value: &DateValue) -> i64 {
    let (year, month_index, _) = wall_ymd(value);
    last_day_of_month_wall(year, month_index)
}

/// The number of days in the JS-rolled `(year, monthIndex)` month (out-of-range months
/// roll over first — the callers normalize through the same `constructFrom` + setter
/// dance upstream).
pub(crate) fn days_in_month_rolled(year: i64, month_index: i64) -> i64 {
    last_day_of_month_wall(year, month_index)
}

/// A wall-clock rebuild from raw components (the `constructFrom(…, 0)` + setters dance
/// upstream uses to build "the same wall time in the value's zone").
pub(crate) fn wall_value(value: &DateValue, year: i64, month_index: i64, day: i64) -> DateValue {
    value.set_wall_ymd(year, month_index, day)
}

/// `date-fns/getDayOfYear.js` — the civil day difference from the year start, + 1.
pub(crate) fn get_day_of_year(value: &DateValue) -> i64 {
    let (year, month_index, day) = wall_ymd(value);
    days_from_civil(year, month_index + 1, day) - days_from_civil(year, 1, 1) + 1
}

/// `date-fns/getWeekYear.js` — the local week-numbering year.
pub(crate) fn get_week_year(
    value: &DateValue,
    week_starts_on: i64,
    first_week_contains_date: i64,
) -> i64 {
    let year = i64::from(value.wall().year());
    let start_of_next_year = start_of_week(
        &wall_value(value, year + 1, 0, first_week_contains_date).set_wall_time(0, 0, 0, 0),
        week_starts_on,
    );
    if value.millis >= start_of_next_year.millis {
        return year + 1;
    }
    let start_of_this_year = start_of_week(
        &wall_value(value, year, 0, first_week_contains_date).set_wall_time(0, 0, 0, 0),
        week_starts_on,
    );
    if value.millis >= start_of_this_year.millis {
        year
    } else {
        year - 1
    }
}

/// `date-fns/startOfWeekYear.js`.
pub(crate) fn start_of_week_year(
    value: &DateValue,
    week_starts_on: i64,
    first_week_contains_date: i64,
) -> DateValue {
    let year = get_week_year(value, week_starts_on, first_week_contains_date);
    start_of_week(
        &wall_value(value, year, 0, first_week_contains_date).set_wall_time(0, 0, 0, 0),
        week_starts_on,
    )
}

/// `date-fns/getWeek.js` — `Math.round((+startOfWeek - +startOfWeekYear) / msInWeek) + 1`
/// (the round absorbs DST-hour week edges).
pub(crate) fn get_week(value: &DateValue, week_starts_on: i64, first_week_contains_date: i64) -> i64 {
    let diff = start_of_week(value, week_starts_on).millis
        - start_of_week_year(value, week_starts_on, first_week_contains_date).millis;
    (diff as f64 / MS_IN_WEEK as f64).round() as i64 + 1
}

/// `date-fns/getISOWeekYear.js` — the ISO week-numbering year (the week-year Jan 4
/// belongs to).
pub(crate) fn get_iso_week_year(value: &DateValue) -> i64 {
    let year = i64::from(value.wall().year());
    let start_of_next_year = start_of_iso_week(
        &wall_value(value, year + 1, 0, 4).set_wall_time(0, 0, 0, 0),
    );
    if value.millis >= start_of_next_year.millis {
        return year + 1;
    }
    let start_of_this_year =
        start_of_iso_week(&wall_value(value, year, 0, 4).set_wall_time(0, 0, 0, 0));
    if value.millis >= start_of_this_year.millis {
        year
    } else {
        year - 1
    }
}

/// `date-fns/startOfISOWeekYear.js` — the ISO week-year's Monday.
pub(crate) fn start_of_iso_week_year(value: &DateValue) -> DateValue {
    let year = get_iso_week_year(value);
    start_of_iso_week(&wall_value(value, year, 0, 4).set_wall_time(0, 0, 0, 0))
}

/// `date-fns/getISOWeek.js`.
pub(crate) fn get_iso_week(value: &DateValue) -> i64 {
    let diff =
        start_of_iso_week(value).millis - start_of_iso_week_year(value).millis;
    (diff as f64 / MS_IN_WEEK as f64).round() as i64 + 1
}

/// `date-fns/getISODay.js` — 1 = Monday … 7 = Sunday.
pub(crate) fn get_iso_day(value: &DateValue) -> i64 {
    match get_day(value) {
        0 => 7,
        day => day,
    }
}

/// `date-fns/compareAsc.js` — instant compare, -1/0/1.
pub(crate) fn compare_asc(value: &DateValue, comparing: &DateValue) -> i64 {
    value.millis.cmp(&comparing.millis) as i64
}

/// The `compareLocalAsc` helper in `date-fns/differenceInDays.js` — a wall-clock
/// field-by-field compare of two dates already projected into the same zone (its
/// documented reason: UTC timestamps sharing a local representation compare equal by
/// wall clock, not by instant).
pub(crate) fn compare_local_asc(later: &DateValue, earlier: &DateValue) -> i64 {
    let a = later.wall();
    let b = earlier.wall();
    let ordering = a.cmp(&b);
    match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Greater => 1,
        std::cmp::Ordering::Equal => 0,
    }
}

/// `date-fns/differenceInCalendarDays.js` — both dates projected into the first
/// argument's zone, then the civil-day difference (the
/// `getTimezoneOffsetInMilliseconds` dance reduces to exactly this inside one zone).
pub(crate) fn difference_in_calendar_days(later: &DateValue, earlier: &DateValue) -> i64 {
    let later_ = project_to_value_zone(later, later);
    let earlier_ = project_to_value_zone(later, earlier);
    let (ly, lm, ld) = wall_ymd(&later_);
    let (ey, em, ed) = wall_ymd(&earlier_);
    days_from_civil(ly, lm + 1, ld) - days_from_civil(ey, em + 1, ed)
}

/// `date-fns/differenceInCalendarMonths.js`.
pub(crate) fn difference_in_calendar_months(later: &DateValue, earlier: &DateValue) -> i64 {
    let (ly, lm, _) = wall_ymd(&project_to_value_zone(later, later));
    let (ey, em, _) = wall_ymd(&project_to_value_zone(later, earlier));
    (ly - ey) * 12 + (lm - em)
}

/// `date-fns/differenceInCalendarYears.js`.
pub(crate) fn difference_in_calendar_years(later: &DateValue, earlier: &DateValue) -> i64 {
    let ly = project_to_value_zone(later, later).wall().year();
    let ey = project_to_value_zone(later, earlier).wall().year();
    i64::from(ly) - i64::from(ey)
}

/// `date-fns/isLastDayOfMonth.js` — `+endOfDay === +endOfMonth`.
pub(crate) fn is_last_day_of_month(value: &DateValue) -> bool {
    end_of_day(value).millis == end_of_month(value).millis
}

/// `date-fns/differenceInDays.js` — the full-days algorithm with its
/// `isLastDayNotFull` correction.
pub(crate) fn difference_in_days(later: &DateValue, earlier: &DateValue) -> i64 {
    let later_ = project_to_value_zone(later, later);
    let earlier_ = project_to_value_zone(later, earlier);
    let sign = compare_local_asc(&later_, &earlier_);
    let difference = difference_in_calendar_days(&later_, &earlier_).abs();
    let (_, _, day) = wall_ymd(&later_);
    let mutated = later_.set_wall_day(day - sign * difference);
    let is_last_day_not_full = i64::from(compare_local_asc(&mutated, &earlier_) == -sign);
    let result = sign * (difference - is_last_day_not_full);
    if result == 0 {
        0
    } else {
        result
    }
}

/// `date-fns/differenceInMonths.js` — the calendar-months algorithm with the Feb-29
/// `setDate(30)` quirk and the last-day-of-month correction.
pub(crate) fn difference_in_months(later: &DateValue, earlier: &DateValue) -> i64 {
    let later_ = project_to_value_zone(later, later);
    let earlier_ = project_to_value_zone(later, earlier);
    let mut working = later_.clone();
    let sign = compare_asc(&working, &earlier_);
    let difference = difference_in_calendar_months(&working, &earlier_).abs();
    if difference < 1 {
        return 0;
    }
    let (_, month_index, day) = wall_ymd(&working);
    if month_index == 1 && day > 27 {
        working = working.set_wall_day(30);
    }
    let (_, wmonth_index, _) = wall_ymd(&working);
    working = working.set_wall_month(wmonth_index - sign * difference, {
        let (_, _, wday) = wall_ymd(&working);
        wday
    });
    let mut is_last_month_not_full = compare_asc(&working, &earlier_) == -sign;
    if is_last_day_of_month(&later_)
        && difference == 1
        && compare_asc(&later_, &earlier_) == 1
    {
        is_last_month_not_full = false;
    }
    let result = sign * (difference - i64::from(is_last_month_not_full));
    if result == 0 {
        0
    } else {
        result
    }
}

/// `date-fns/differenceInYears.js` — both dates re-set to the pivot year 1584, then the
/// full/partial correction.
pub(crate) fn difference_in_years(later: &DateValue, earlier: &DateValue) -> i64 {
    let mut later_ = project_to_value_zone(later, later);
    let mut earlier_ = project_to_value_zone(later, earlier);
    let sign = compare_asc(&later_, &earlier_);
    let diff = difference_in_calendar_years(&later_, &earlier_).abs();
    later_ = later_.set_wall_year(1584);
    earlier_ = earlier_.set_wall_year(1584);
    let partial = compare_asc(&later_, &earlier_) == -sign;
    let result = sign * (diff - i64::from(partial));
    if result == 0 {
        0
    } else {
        result
    }
}

/// `date-fns/differenceInWeeks.js` — `Math.trunc(differenceInDays / 7)` with the
/// negative-zero guard.
pub(crate) fn difference_in_weeks(later: &DateValue, earlier: &DateValue) -> i64 {
    let diff = difference_in_days(later, earlier);
    let truncated = diff.div_euclid(7);
    if truncated == 0 {
        0
    } else {
        truncated
    }
}

/// `date-fns/differenceInHours.js` — instant difference, truncated.
pub(crate) fn difference_in_hours(later: &DateValue, earlier: &DateValue) -> i64 {
    (later.millis - earlier.millis).div_euclid(MS_IN_HOUR)
}

/// `date-fns/differenceInMinutes.js`.
pub(crate) fn difference_in_minutes(later: &DateValue, earlier: &DateValue) -> i64 {
    (later.millis - earlier.millis).div_euclid(MS_IN_MINUTE)
}

/// `date-fns/addMonths.js` — the end-of-month clamp (`new Date(2020, 13, 31)` must be
/// Feb 28, not Mar 3).
pub(crate) fn add_months(value: &DateValue, amount: i64) -> DateValue {
    if amount == 0 {
        return value.clone();
    }
    let (_, month_index, day_of_month) = wall_ymd(value);
    // `setMonth(month + amount + 1, 0)` — the last day of the desired month.
    let end_of_desired_month = value.set_wall_month(month_index + amount + 1, 0);
    let days_in_month = i64::from(end_of_desired_month.wall().day());
    if day_of_month >= days_in_month {
        end_of_desired_month
    } else {
        let (ey, em, _) = wall_ymd(&end_of_desired_month);
        value.set_wall_ymd(ey, em, day_of_month)
    }
}

/// `date-fns/addYears.js` — `addMonths(date, amount * 12)`.
pub(crate) fn add_years(value: &DateValue, amount: i64) -> DateValue {
    add_months(value, amount * 12)
}

/// `date-fns/addDays.js` — wall-clock `setDate(getDate() + amount)` (DST keeps the
/// local time of day).
pub(crate) fn add_days(value: &DateValue, amount: i64) -> DateValue {
    if amount == 0 {
        return value.clone();
    }
    let (_, _, day) = wall_ymd(value);
    value.set_wall_day(day + amount)
}

/// `date-fns/addWeeks.js`.
pub(crate) fn add_weeks(value: &DateValue, amount: i64) -> DateValue {
    add_days(value, amount * 7)
}

/// `date-fns/addMilliseconds.js` — instant arithmetic.
pub(crate) fn add_milliseconds(value: &DateValue, amount: i64) -> DateValue {
    DateValue {
        millis: value.millis + amount,
        zone: value.zone.clone(),
    }
}

/// `date-fns/addHours.js`.
pub(crate) fn add_hours(value: &DateValue, amount: i64) -> DateValue {
    add_milliseconds(value, amount * MS_IN_HOUR)
}

/// `date-fns/addMinutes.js`.
pub(crate) fn add_minutes(value: &DateValue, amount: i64) -> DateValue {
    add_milliseconds(value, amount * MS_IN_MINUTE)
}

/// `date-fns/addSeconds.js`.
pub(crate) fn add_seconds(value: &DateValue, amount: i64) -> DateValue {
    add_milliseconds(value, amount * 1_000)
}

/// `date-fns/setDay.js` — the `weekStartsOn` delta formula over `addDays`.
pub(crate) fn set_day(value: &DateValue, day: i64, week_starts_on: i64) -> DateValue {
    let current_day = get_day(value);
    let remainder = day.rem_euclid(7);
    let day_index = remainder;
    let delta = 7 - week_starts_on;
    let diff = if day < 0 || day > 6 {
        day - (current_day + delta).rem_euclid(7)
    } else {
        (day_index + delta).rem_euclid(7) - (current_day + delta).rem_euclid(7)
    };
    add_days(value, diff)
}

/// `date-fns/setISODay.js`.
pub(crate) fn set_iso_day(value: &DateValue, day: i64) -> DateValue {
    let current = get_iso_day(value);
    add_days(value, day - current)
}

/// `date-fns/setWeek.js` — `setDate(getDate() - (getWeek - week) * 7)`.
pub(crate) fn set_week(
    value: &DateValue,
    week: i64,
    week_starts_on: i64,
    first_week_contains_date: i64,
) -> DateValue {
    let diff = get_week(value, week_starts_on, first_week_contains_date) - week;
    let (_, _, day) = wall_ymd(value);
    value.set_wall_day(day - diff * 7)
}

/// `date-fns/setISOWeek.js`.
pub(crate) fn set_iso_week(value: &DateValue, week: i64) -> DateValue {
    let diff = get_iso_week(value) - week;
    let (_, _, day) = wall_ymd(value);
    value.set_wall_day(day - diff * 7)
}

/// `date-fns/setYear.js` — plain `setFullYear` (the Feb 29 → Mar 1 rollover).
pub(crate) fn set_year(value: &DateValue, year: i64) -> DateValue {
    value.set_wall_year(year)
}

/// `date-fns/setMonth.js` — the day clamps to the target month's length ("allows to
/// wrap Jan 31 to Feb 28").
pub(crate) fn set_month(value: &DateValue, month: i64) -> DateValue {
    let wall = value.wall();
    let year = i64::from(wall.year());
    let day = i64::from(wall.day());
    let days_in_month = days_in_month_rolled(year, month);
    value.set_wall_month(month, day.min(days_in_month))
}

/// `date-fns/setDate.js`.
pub(crate) fn set_date(value: &DateValue, day: i64) -> DateValue {
    value.set_wall_day(day)
}

/// The zone-offset in minutes west of UTC at the value's instant — the JS
/// `getTimezoneOffset()` (the TZDateMini floor/ceil toward zero for historical
/// seconds-offsets, `date/mini.js:80-85`).
pub(crate) fn get_timezone_offset_minutes_west(value: &DateValue) -> i64 {
    let minutes = get_timezone_offset_millis(value) as f64 / MS_IN_MINUTE as f64;
    if minutes > 0.0 {
        minutes.floor() as i64
    } else {
        minutes.ceil() as i64
    }
}

/// The date-fns-signature offset at the instant (`getTimezoneOffsetInMilliseconds`):
/// `+date - +utcDate`, negative east of UTC.
pub(crate) fn get_timezone_offset_millis(value: &DateValue) -> i64 {
    crate::temporal::offset_millis(value.zone(), value.millis())
}

/// A civil `DateTime` at `(year, monthIndex, day)` midnight — the
/// `constructFrom(…, 0)` + `setHours(0,0,0,0)` pattern.
pub(crate) fn civil_midnight(year: i64, month_index: i64, day: i64) -> civil::DateTime {
    let (y, m, d) = civil_rollover(year, month_index, day);
    civil::DateTime::new(y as i16, m as i8, d as i8, 0, 0, 0, 0)
        .expect("rolled civil date in range")
}

/// Resolves a wall clock into an instant for an already-resolved [`Zone`] — exposed for
/// the parse engine's offset-token setter.
pub(crate) fn wall_in_zone(wall: civil::DateTime, zone: &Zone) -> i64 {
    wall_in_zone_millis(wall, zone)
}
