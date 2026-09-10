//! Port of `date-fns/format.js` (v4.4.0) — the token engine behind the adapter's
//! `format`/`formatByString`
//! (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:232-238`,
//! which passes the adapter's locale through `dateFnsFormat(value, format, { locale })`).
//! Formatters transcribe `date-fns/_lib/format/formatters.js` +
//! `lightFormatters.js`; the `P`/`p` pre-pass transcribes `longFormatters.js` over the
//! locale's `formatLong` tables (embedded in [`DateFnsLocale`]).
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The regex tokenizers become hand-rolled scanners with the same alternation order:
//!   ordinal tokens `[yYQqMLwIdDecihHKkms]o` first, same-`\w`-character runs, the `''`
//!   escape, quoted strings `'(''|[^'])+('|$)`, then any single character.
//! - Upstream throws `RangeError` for unescaped latin alphabet characters and for the
//!   protected `D`/`DD`/`YY`/`YYYY` tokens; the port panics with the same messages (the
//!   error-guideline mapping for developer-facing invariants). The warn-only protected
//!   tokens (`Y`, `YYY`, `DDD`, …) go through the crate's `warn` logger exactly as
//!   upstream's `console.warn`.
//! - `String(date)` inside the protected-token message has no JS `Date` toString here;
//!   the value's epoch milliseconds stand in.
//! - `isValid` throws `RangeError('Invalid time value')` for invalid dates; the port's
//!   `DateValue` cannot be invalid (the trait boundary uses `None`), so the check is
//!   structural.

use leptos_ui_utils::warn as warn_log;

use crate::date_fns_calendar::{
    get_day, get_day_of_year, get_iso_day, get_iso_week, get_iso_week_year,
    get_timezone_offset_minutes_west, get_week, get_week_year,
};
use crate::date_fns_locale::{Context, DateFnsLocale, DayPeriod, Width};
use crate::temporal::DateValue;

/// The formatter token chars (`formatters` table keys).
const FORMATTER_CHARS: &str = "GyYRuQqMLwIdDEeciabBHKkhmsSXxOztT";

/// `format(date, formatStr, { locale })` (`date-fns/format.js`).
pub(crate) fn format(value: &DateValue, format_str: &str, locale: &'static DateFnsLocale) -> String {
    let week_starts_on = locale.week_starts_on;
    let first_week_contains_date = locale.first_week_contains_date;

    // Long-formatter pre-pass: expand `P`/`p` runs through `locale.formatLong`.
    let expanded = tokenize_long_formatters(format_str, locale).join("");
    // Token pass.
    let tokens = tokenize_tokens(&expanded);
    let mut parts: Vec<(bool, String)> = tokens
        .into_iter()
        .map(|token| {
            if token == "''" {
                (false, "'".to_string())
            } else if token.starts_with('\'') {
                (false, clean_escaped_string(&token))
            } else if FORMATTER_CHARS.contains(token.chars().next().unwrap_or('\0')) {
                (true, token)
            } else {
                let first = token.chars().next().unwrap();
                if first.is_ascii_alphabetic() {
                    panic!(
                        "Format string contains an unescaped latin alphabet character `{first}`"
                    );
                }
                (false, token)
            }
        })
        .collect();

    // `localize.preprocessor` (only fr at the moment): the `do` → `d` substitution.
    if let Some(preprocess) = locale.preprocessor {
        parts = preprocess(i64::from(value.wall().day()), &parts);
    }

    let mut out = String::new();
    for (is_token, token) in &parts {
        if !*is_token {
            out.push_str(token);
            continue;
        }
        check_protected_token(token, format_str, value);
        out.push_str(&run_formatter(
            value,
            token,
            locale,
            week_starts_on,
            first_week_contains_date,
        ));
    }
    out
}

/// `warnOrThrowProtectedError` (`date-fns/_lib/protectedTokens.js`): warn for every
/// `^Y+$`/`^D+$` token, throw for `D`/`DD`/`YY`/`YYYY`.
fn check_protected_token(token: &str, format_str: &str, value: &DateValue) {
    let all_same = |c: char| token.chars().all(|t| t == c);
    let protected_week_year = token.starts_with('Y') && all_same('Y');
    let protected_day_of_year = token.starts_with('D') && all_same('D');
    if !protected_week_year && !protected_day_of_year {
        return;
    }
    let subject = if protected_week_year { "years" } else { "days of the month" };
    let message = format!(
        "Use `{}` instead of `{}` (in `{}`) for formatting {} to the input `{}`; see: \
         https://github.com/date-fns/date-fns/blob/master/docs/unicodeTokens.md",
        token.to_lowercase(),
        token,
        format_str,
        subject,
        value.millis(),
    );
    warn_log().log(&[&message]);
    if matches!(token, "D" | "DD" | "YY" | "YYYY") {
        panic!("{message}");
    }
}

// ─── tokenizers ──────────────────────────────────────────────────────────────────────

/// `/P+p+|P+|p+|''|'(''|[^'])+('|$)|./g` with the `P`/`p` runs expanded through
/// `longFormatters`.
pub(crate) fn tokenize_long_formatters(input: &str, locale: &DateFnsLocale) -> Vec<String> {
    let chars: Vec<char> = input.chars().collect();
    let mut pieces = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == 'P' || c == 'p' {
            let p_run = chars[i..].iter().take_while(|&&ch| ch == 'P').count();
            let q_run = chars[i + p_run..].iter().take_while(|&&ch| ch == 'p').count();
            let piece_len = if p_run > 0 && q_run > 0 {
                p_run + q_run // `P+p+`
            } else {
                p_run.max(q_run) // `P+` or `p+`
            };
            let piece: String = chars[i..i + piece_len].iter().collect();
            pieces.push(long_formatter(&piece, locale));
            i += piece_len;
        } else if c == '\'' {
            let (piece, consumed) = quote_or_char(&chars, i);
            pieces.push(piece);
            i += consumed;
        } else {
            pieces.push(c.to_string());
            i += 1;
        }
    }
    pieces
}

/// `dateTimeLongFormatter`/`dateLongFormatter`/`timeLongFormatter`
/// (`date-fns/_lib/format/longFormatters.js`).
fn long_formatter(pattern: &str, locale: &DateFnsLocale) -> String {
    let first = pattern.chars().next().unwrap();
    if first == 'P' {
        // `/(P+)(p+)?/` — the date part with an optional time part.
        let p_run = pattern.chars().take_while(|&c| c == 'P').count();
        let has_time = pattern.chars().skip(p_run).take_while(|&c| c == 'p').count() > 0;
        let date_pattern = &pattern[..p_run];
        let date = date_long_formatter(date_pattern, locale);
        if !has_time {
            return date.to_string();
        }
        let time_pattern = &pattern[p_run..];
        let date_time = match date_pattern.len() {
            1 => locale.date_time_formats.short,
            2 => locale.date_time_formats.medium,
            3 => locale.date_time_formats.long,
            _ => locale.date_time_formats.full,
        };
        date_time
            .replace("{{date}}", date)
            .replace("{{time}}", time_long_formatter(time_pattern, locale))
    } else {
        time_long_formatter(pattern, locale).to_string()
    }
}

fn date_long_formatter(pattern: &str, locale: &DateFnsLocale) -> &'static str {
    match pattern.len() {
        1 => locale.date_formats.short,
        2 => locale.date_formats.medium,
        3 => locale.date_formats.long,
        _ => locale.date_formats.full,
    }
}

fn time_long_formatter(pattern: &str, locale: &DateFnsLocale) -> &'static str {
    match pattern.len() {
        1 => locale.time_formats.short,
        2 => locale.time_formats.medium,
        3 => locale.time_formats.long,
        _ => locale.time_formats.full,
    }
}

/// `/[yYQqMLwIdDecihHKkms]o|(\w)\1*|''|'(''|[^'])+('|$)|./g`.
pub(crate) fn tokenize_tokens(input: &str) -> Vec<String> {
    const ORDINAL_CLASS: &str = "yYQqMLwIdDecihHKkms";
    let chars: Vec<char> = input.chars().collect();
    let mut pieces = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if ORDINAL_CLASS.contains(c) && chars.get(i + 1) == Some(&'o') {
            pieces.push(format!("{c}o"));
            i += 2;
        } else if c.is_ascii_alphanumeric() || c == '_' {
            let run = chars[i..].iter().take_while(|&&ch| ch == c).count();
            pieces.push(chars[i..i + run].iter().collect());
            i += run;
        } else if c == '\'' {
            let (piece, consumed) = quote_or_char(&chars, i);
            pieces.push(piece);
            i += consumed;
        } else {
            pieces.push(c.to_string());
            i += 1;
        }
    }
    pieces
}

/// At a `'` position: the `''` pair as its own piece, else the quoted string
/// `'(''|[^'])+('|$)` (at least one unit required; the closing quote optional at end of
/// string), else the single character.
fn quote_or_char(chars: &[char], start: usize) -> (String, usize) {
    if chars.get(start + 1) == Some(&'\'') {
        return ("''".to_string(), 2);
    }
    // `'(''|[^'])+('|$)` — greedy unit scan.
    let mut j = start + 1;
    let mut units = 0;
    while j < chars.len() {
        if chars[j] == '\'' {
            if chars.get(j + 1) == Some(&'\'') {
                j += 2; // one `''` unit
                units += 1;
            } else {
                break; // the closing quote
            }
        } else {
            j += 1;
            units += 1;
        }
    }
    if units == 0 {
        return ("'".to_string(), 1);
    }
    let mut end = j;
    if chars.get(j) == Some(&'\'') {
        end = j + 1; // the `('|$)` closing group
    }
    (chars[start..end].iter().collect(), end - start)
}

/// `cleanEscapedString` (`date-fns/format.js`): `/^'([^]*?)'?$/` (lazy) then
/// `''` → `'`. The lazy inner with the optional trailing quote always strips the last
/// character when one exists, so an unterminated quote loses everything after it —
/// reproduced exactly.
pub(crate) fn clean_escaped_string(token: &str) -> String {
    let chars: Vec<char> = token.chars().collect();
    let inner: String = if chars.len() >= 2 {
        chars[1..chars.len() - 1].iter().collect()
    } else {
        String::new()
    };
    inner.replace("''", "'")
}

// ─── formatters ──────────────────────────────────────────────────────────────────────

/// The `formatters[token[0]]` dispatch (`date-fns/_lib/format/formatters.js` +
/// `lightFormatters.js`).
fn run_formatter(
    value: &DateValue,
    token: &str,
    locale: &'static DateFnsLocale,
    week_starts_on: i64,
    first_week_contains_date: i64,
) -> String {
    match token.chars().next().unwrap() {
        // Era
        'G' => {
            let era = if value.wall().year() > 0 { 1 } else { 0 };
            match token {
                "G" | "GG" | "GGG" => localize_era(locale, era, Width::Abbreviated),
                "GGGGG" => localize_era(locale, era, Width::Narrow),
                _ => localize_era(locale, era, Width::Wide),
            }
        }
        // Year
        'y' => {
            if token == "yo" {
                let signed = i64::from(value.wall().year());
                let year = if signed > 0 { signed } else { 1 - signed };
                return (locale.ordinal_number)(year, "year");
            }
            let signed = i64::from(value.wall().year());
            let year = if signed > 0 { signed } else { 1 - signed };
            add_leading_zeros(if token == "yy" { year % 100 } else { year }, token.len())
        }
        // Local week-numbering year
        'Y' => {
            let week_year =
                get_week_year(value, week_starts_on, first_week_contains_date);
            let signed = week_year;
            let year = if signed > 0 { signed } else { 1 - signed };
            if token == "YY" {
                add_leading_zeros(year % 100, 2)
            } else if token == "Yo" {
                (locale.ordinal_number)(year, "year")
            } else {
                add_leading_zeros(year, token.len())
            }
        }
        // ISO week-numbering year
        'R' => add_leading_zeros(get_iso_week_year(value), token.len()),
        // Extended year
        'u' => add_leading_zeros(i64::from(value.wall().year()), token.len()),
        // Quarter
        'Q' | 'q' => {
            let month0 = i64::from(value.wall().month()) - 1;
            let quarter = (month0 + 3) / 3; // `Math.ceil((month + 1) / 3)`
            match token {
                "Q" | "q" => quarter.to_string(),
                "QQ" | "qq" => add_leading_zeros(quarter, 2),
                "Qo" | "qo" => (locale.ordinal_number)(quarter, "quarter"),
                "QQQ" | "qqq" => localize_quarter(locale, quarter, Width::Abbreviated),
                "QQQQQ" | "qqqqq" => localize_quarter(locale, quarter, Width::Narrow),
                _ => localize_quarter(locale, quarter, Width::Wide),
            }
        }
        // Month
        'M' | 'L' => {
            let month0 = i64::from(value.wall().month()) - 1;
            match token {
                "M" => (month0 + 1).to_string(),
                "MM" => add_leading_zeros(month0 + 1, 2),
                "Mo" | "Lo" => (locale.ordinal_number)(month0 + 1, "month"),
                "MMM" | "LLL" => localize_month(locale, month0, Width::Abbreviated),
                "MMMMM" | "LLLLL" => localize_month(locale, month0, Width::Narrow),
                "L" => (month0 + 1).to_string(),
                "LL" => add_leading_zeros(month0 + 1, 2),
                _ => localize_month(locale, month0, Width::Wide),
            }
        }
        // Local week of year
        'w' => {
            let week = get_week(value, week_starts_on, first_week_contains_date);
            if token == "wo" {
                (locale.ordinal_number)(week, "week")
            } else {
                add_leading_zeros(week, token.len())
            }
        }
        // ISO week of year
        'I' => {
            let week = get_iso_week(value);
            if token == "Io" {
                (locale.ordinal_number)(week, "week")
            } else {
                add_leading_zeros(week, token.len())
            }
        }
        // Day of the month
        'd' => {
            if token == "do" {
                return (locale.ordinal_number)(i64::from(value.wall().day()), "date");
            }
            add_leading_zeros(i64::from(value.wall().day()), token.len())
        }
        // Day of year
        'D' => {
            let day_of_year = get_day_of_year(value);
            if token == "Do" {
                (locale.ordinal_number)(day_of_year, "dayOfYear")
            } else {
                add_leading_zeros(day_of_year, token.len())
            }
        }
        // Day of week
        'E' => {
            let day = get_day(value);
            match token {
                "E" | "EE" | "EEE" => localize_day(locale, day, Width::Abbreviated),
                "EEEEE" => localize_day(locale, day, Width::Narrow),
                "EEEEEE" => localize_day(locale, day, Width::Short),
                _ => localize_day(locale, day, Width::Wide),
            }
        }
        // Local day of week
        'e' | 'c' => {
            let day = get_day(value);
            let local_day = {
                let value = (day - week_starts_on + 8) % 7;
                if value == 0 {
                    7
                } else {
                    value
                }
            };
            match token {
                "e" | "c" => local_day.to_string(),
                "ee" => add_leading_zeros(local_day, 2),
                "cc" => add_leading_zeros(local_day, token.len()),
                "eo" | "co" => (locale.ordinal_number)(local_day, "day"),
                "eee" | "ccc" => localize_day(locale, day, Width::Abbreviated),
                "eeeee" | "ccccc" => localize_day(locale, day, Width::Narrow),
                "eeeeee" | "cccccc" => localize_day(locale, day, Width::Short),
                _ => localize_day(locale, day, Width::Wide),
            }
        }
        // ISO day of week
        'i' => {
            let day = get_iso_day(value);
            match token {
                "i" => day.to_string(),
                "ii" => add_leading_zeros(day, token.len()),
                "io" => (locale.ordinal_number)(day, "day"),
                "iii" => localize_day(locale, get_day(value), Width::Abbreviated),
                "iiiii" => localize_day(locale, get_day(value), Width::Narrow),
                "iiiiii" => localize_day(locale, get_day(value), Width::Short),
                _ => localize_day(locale, get_day(value), Width::Wide),
            }
        }
        // AM or PM
        'a' => {
            let period = day_period_hours_division(value);
            match token {
                "a" | "aa" => localize_day_period(locale, period, Width::Abbreviated, Context::Formatting),
                "aaa" => {
                    localize_day_period(locale, period, Width::Abbreviated, Context::Formatting)
                        .to_lowercase()
                }
                "aaaaa" => localize_day_period(locale, period, Width::Narrow, Context::Formatting),
                _ => localize_day_period(locale, period, Width::Wide, Context::Formatting),
            }
        }
        // AM, PM, midnight, noon
        'b' => {
            let hours = i64::from(value.wall().hour());
            let period = if hours == 12 {
                DayPeriod::Noon
            } else if hours == 0 {
                DayPeriod::Midnight
            } else {
                day_period_hours_division(value)
            };
            match token {
                "b" | "bb" => localize_day_period(locale, period, Width::Abbreviated, Context::Formatting),
                "bbb" => {
                    localize_day_period(locale, period, Width::Abbreviated, Context::Formatting)
                        .to_lowercase()
                }
                "bbbbb" => localize_day_period(locale, period, Width::Narrow, Context::Formatting),
                _ => localize_day_period(locale, period, Width::Wide, Context::Formatting),
            }
        }
        // in the morning, in the afternoon, in the evening, at night
        'B' => {
            let hours = i64::from(value.wall().hour());
            let period = if hours >= 17 {
                DayPeriod::Evening
            } else if hours >= 12 {
                DayPeriod::Afternoon
            } else if hours >= 4 {
                DayPeriod::Morning
            } else {
                DayPeriod::Night
            };
            match token {
                "B" | "BB" | "BBB" => {
                    localize_day_period(locale, period, Width::Abbreviated, Context::Formatting)
                }
                "BBBBB" => localize_day_period(locale, period, Width::Narrow, Context::Formatting),
                _ => localize_day_period(locale, period, Width::Wide, Context::Formatting),
            }
        }
        // Hour [1-12]
        'h' => {
            if token == "ho" {
                let hours = i64::from(value.wall().hour()) % 12;
                let hours = if hours == 0 { 12 } else { hours };
                return (locale.ordinal_number)(hours, "hour");
            }
            let hours = i64::from(value.wall().hour()) % 12;
            add_leading_zeros(if hours == 0 { 12 } else { hours }, token.len())
        }
        // Hour [0-23]
        'H' => {
            if token == "Ho" {
                return (locale.ordinal_number)(i64::from(value.wall().hour()), "hour");
            }
            add_leading_zeros(i64::from(value.wall().hour()), token.len())
        }
        // Hour [0-11]
        'K' => {
            let hours = i64::from(value.wall().hour()) % 12;
            if token == "Ko" {
                return (locale.ordinal_number)(hours, "hour");
            }
            add_leading_zeros(hours, token.len())
        }
        // Hour [1-24]
        'k' => {
            let hours = i64::from(value.wall().hour());
            let hours = if hours == 0 { 24 } else { hours };
            if token == "ko" {
                return (locale.ordinal_number)(hours, "hour");
            }
            add_leading_zeros(hours, token.len())
        }
        // Minute
        'm' => {
            if token == "mo" {
                return (locale.ordinal_number)(i64::from(value.wall().minute()), "minute");
            }
            add_leading_zeros(i64::from(value.wall().minute()), token.len())
        }
        // Second
        's' => {
            if token == "so" {
                return (locale.ordinal_number)(i64::from(value.wall().second()), "second");
            }
            add_leading_zeros(i64::from(value.wall().second()), token.len())
        }
        // Fraction of second
        'S' => {
            let digits = token.len();
            let ms = i64::from(value.wall().millisecond());
            let fractional = (ms as f64 * 10f64.powi(digits as i32 - 3)).trunc() as i64;
            add_leading_zeros(fractional, digits)
        }
        // Timezone (ISO-8601, `Z` at zero offset)
        'X' => {
            let offset = get_timezone_offset_minutes_west(value);
            if offset == 0 {
                return "Z".to_string();
            }
            match token {
                "X" => format_timezone_with_optional_minutes(offset, ""),
                "XX" | "XXXX" => format_timezone(offset, ""),
                _ => format_timezone(offset, ":"),
            }
        }
        // Timezone (ISO-8601)
        'x' => {
            let offset = get_timezone_offset_minutes_west(value);
            match token {
                "x" => format_timezone_with_optional_minutes(offset, ""),
                "xx" | "xxxx" => format_timezone(offset, ""),
                _ => format_timezone(offset, ":"),
            }
        }
        // Timezone (GMT)
        'O' => {
            let offset = get_timezone_offset_minutes_west(value);
            match token {
                "O" | "OO" | "OOO" => format!("GMT{}", format_timezone_short(offset, ":")),
                _ => format!("GMT{}", format_timezone(offset, ":")),
            }
        }
        // Timezone (specific non-location)
        'z' => {
            let offset = get_timezone_offset_minutes_west(value);
            match token {
                "z" | "zz" | "zzz" => format!("GMT{}", format_timezone_short(offset, ":")),
                _ => format!("GMT{}", format_timezone(offset, ":")),
            }
        }
        // Seconds timestamp
        't' => add_leading_zeros(value.millis().div_euclid(1_000), token.len()),
        // Milliseconds timestamp
        'T' => add_leading_zeros(value.millis(), token.len()),
        _ => String::new(),
    }
}

/// The `am`/`pm` half of the day (`hours / 12 >= 1 ? 'pm' : 'am'`).
fn day_period_hours_division(value: &DateValue) -> DayPeriod {
    if i64::from(value.wall().hour()) / 12 >= 1 {
        DayPeriod::Pm
    } else {
        DayPeriod::Am
    }
}

fn add_leading_zeros(number: i64, target_length: usize) -> String {
    let sign = if number < 0 { "-" } else { "" };
    format!("{sign}{:0>width$}", number.abs(), width = target_length)
}

fn format_timezone_short(offset: i64, delimiter: &str) -> String {
    let sign = if offset > 0 { "-" } else { "+" };
    let abs = offset.abs();
    let hours = abs / 60;
    let minutes = abs % 60;
    if minutes == 0 {
        format!("{sign}{hours}")
    } else {
        format!("{sign}{hours}{delimiter}{}", add_leading_zeros(minutes, 2))
    }
}

fn format_timezone(offset: i64, delimiter: &str) -> String {
    let sign = if offset > 0 { "-" } else { "+" };
    let abs = offset.abs();
    format!(
        "{sign}{}{delimiter}{}",
        add_leading_zeros(abs / 60, 2),
        add_leading_zeros(abs % 60, 2)
    )
}

fn format_timezone_with_optional_minutes(offset: i64, delimiter: &str) -> String {
    if offset % 60 == 0 {
        let sign = if offset > 0 { "-" } else { "+" };
        format!("{sign}{}", add_leading_zeros(offset.abs() / 60, 2))
    } else {
        format_timezone(offset, delimiter)
    }
}

// ─── localize ────────────────────────────────────────────────────────────────────────

/// `localize.era` — `values[width] || values[defaultWidth: 'wide']`.
fn localize_era(locale: &DateFnsLocale, era: i64, width: Width) -> String {
    let eras = &locale.eras;
    let value = match width {
        Width::Narrow => eras.narrow[era as usize],
        Width::Abbreviated => eras.abbreviated[era as usize],
        Width::Wide | Width::Short => eras.wide[era as usize],
    };
    value.to_string()
}

/// `localize.quarter` (`argumentCallback: quarter - 1`).
fn localize_quarter(locale: &DateFnsLocale, quarter: i64, width: Width) -> String {
    let quarters = &locale.quarters;
    let index = (quarter - 1).max(0) as usize;
    let value = match width {
        Width::Narrow => quarters.narrow[index],
        Width::Abbreviated => quarters.abbreviated[index],
        Width::Wide | Width::Short => quarters.wide[index],
    };
    value.to_string()
}

/// `localize.month`.
fn localize_month(locale: &DateFnsLocale, month0: i64, width: Width) -> String {
    let months = &locale.months;
    let value = match width {
        Width::Narrow => months.narrow[month0 as usize],
        Width::Abbreviated => months.abbreviated[month0 as usize],
        Width::Wide | Width::Short => months.wide[month0 as usize],
    };
    value.to_string()
}

/// `localize.day`.
fn localize_day(locale: &DateFnsLocale, day0: i64, width: Width) -> String {
    let days = &locale.days;
    let value = match width {
        Width::Narrow => days.narrow[day0 as usize],
        Width::Short => days.short[day0 as usize],
        Width::Abbreviated => days.abbreviated[day0 as usize],
        Width::Wide => days.wide[day0 as usize],
    };
    value.to_string()
}

/// `localize.dayPeriod` — `formattingValues` for the formatting context when the locale
/// carries one.
fn localize_day_period(
    locale: &DateFnsLocale,
    period: DayPeriod,
    width: Width,
    context: Context,
) -> String {
    locale
        .day_periods
        .get(context)
        .get(width)
        .get(period)
        .to_string()
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::date_fns_locale::{EN_US, FR};

    // 2020-01-01T15:08:09.000Z — the testFormats.ts fixture date.
    const FIXTURE: i64 = 1_577_891_289_000;

    fn fixture_utc() -> DateValue {
        DateValue::from_millis_in_zone(FIXTURE, "utc").expect("valid zone")
    }

    #[test]
    fn formats_the_harness_standalone_keys() {
        // testFormats.ts:14-31, in 'utc'.
        let value = fixture_utc();
        let f = |key| format(&value, key, &EN_US);
        assert_eq!(f("yyyy"), "2020");
        assert_eq!(f("MM"), "01");
        assert_eq!(f("dd"), "01");
        assert_eq!(f("HH"), "15");
        assert_eq!(f("hh"), "03");
        assert_eq!(f("mm"), "08");
        assert_eq!(f("ss"), "09");
        assert_eq!(f("d"), "1");
        assert_eq!(f("EEEE"), "Wednesday");
        assert_eq!(f("EEE"), "Wed");
        assert_eq!(f("a"), "PM");
    }

    #[test]
    fn formats_long_forms_and_ordinals() {
        let value = fixture_utc();
        // P → short ("MM/dd/yyyy"); PPPP → full ("EEEE, MMMM do, y").
        assert_eq!(format(&value, "P", &EN_US), "01/01/2020");
        assert_eq!(format(&value, "PPPP", &EN_US), "Wednesday, January 1st, 2020");
        assert_eq!(format(&value, "do", &EN_US), "1st");
        assert_eq!(format(&value, "MMMM", &EN_US), "January");
        assert_eq!(format(&value, "MMMMM", &EN_US), "J");
        // fr: the preprocessor leaves "1 janvier" (no `do` token in the fr formatLong).
        assert_eq!(format(&value, "PPPP", &FR), "mercredi 1 janvier 2020");
        assert_eq!(format(&value, "EEEE", &FR), "mercredi");
        assert_eq!(format(&value, "a", &FR), "PM");
    }

    #[test]
    fn formats_quoting_and_literals() {
        let value = fixture_utc();
        // The harness's custom-parse format family: quoted 'T'/'Z' literals.
        assert_eq!(format(&value, "yyyy MM dd'T'HH mm ss'Z'", &EN_US), "2020 01 01T15 08 09Z");
        // `''` → a literal quote (digits pass through as literal runs; latin letters
        // outside the formatter set would panic — pinned below).
        assert_eq!(format(&value, "d''11''MM", &EN_US), "1'11'01");
        // Non-token punctuation passes through; digits are literal runs.
        assert_eq!(format(&value, "yyyy-MM-dd HH:mm:ss.SSS", &EN_US), "2020-01-01 15:08:09.000");
    }

    #[test]
    fn formats_the_wider_token_zoo() {
        let value = fixture_utc();
        assert_eq!(format(&value, "G GG GGGG GGGGG", &EN_US), "AD AD Anno Domini A");
        assert_eq!(format(&value, "yy yyyy", &EN_US), "20 2020");
        assert_eq!(format(&value, "Q QQ QQQ QQQQ", &EN_US), "1 01 Q1 1st quarter");
        assert_eq!(format(&value, "w ww", &EN_US), "1 01"); // Jan 1 is week 1 (en-US)
        assert_eq!(format(&value, "DDDD", &EN_US), "0001"); // `D`/`DD` are protected (panic); `DDD`+ are warn-only
        assert_eq!(format(&value, "e ee eee", &EN_US), "4 04 Wed"); // Wed in a Sunday-start week
        assert_eq!(format(&value, "i ii iiii", &EN_US), "3 03 Wednesday");
        assert_eq!(format(&value, "h H K k", &EN_US), "3 15 3 15");
        assert_eq!(format(&value, "b B", &EN_US), "PM in the afternoon");
        assert_eq!(format(&value, "S SS SSS", &EN_US), "0 00 000");
        assert_eq!(format(&value, "t T", &EN_US), "1577891289 1577891289000");
    }

    #[test]
    fn formats_offsets_per_zone() {
        let ny = DateValue::from_millis_in_zone(FIXTURE, "America/New_York").expect("zone");
        // Jan 1: EST = UTC-5 → minutes west = 300.
        assert_eq!(format(&ny, "XXX", &EN_US), "-05:00");
        assert_eq!(format(&ny, "XX", &EN_US), "-0500");
        assert_eq!(format(&ny, "X", &EN_US), "-05");
        assert_eq!(format(&ny, "x", &EN_US), "-05");
        assert_eq!(format(&ny, "O", &EN_US), "GMT-5");
        assert_eq!(format(&ny, "OOOO", &EN_US), "GMT-05:00");
        let utc = fixture_utc();
        assert_eq!(format(&utc, "XXX", &EN_US), "Z");
        assert_eq!(format(&utc, "x", &EN_US), "+00");
    }

    #[test]
    fn computes_the_harness_week_numbers() {
        // testComputations.ts:855-857 — 2018-10-30T11:44:25.750Z is ISO/local week 44.
        let value = DateValue::from_millis_in_zone(1_540_899_865_750, "utc").expect("zone");
        assert_eq!(format(&value, "w", &EN_US), "44");
        assert_eq!(format(&value, "I", &EN_US), "44");
        assert_eq!(format(&value, "Y", &EN_US), "2018");
        assert_eq!(format(&value, "R", &EN_US), "2018");
    }

    #[test]
    #[should_panic(expected = "unescaped latin alphabet character `v`")]
    fn panics_on_unescaped_latin_characters() {
        format(&fixture_utc(), "yyyyv", &EN_US);
    }

    #[test]
    #[should_panic(expected = "Use `yyyy` instead of `YYYY`")]
    fn panics_on_protected_year_tokens() {
        format(&fixture_utc(), "YYYY", &EN_US);
    }
}
