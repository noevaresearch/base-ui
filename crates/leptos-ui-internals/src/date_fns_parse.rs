//! Port of `date-fns/parse.js` (v4.4.0) — the token engine behind the adapter's `parse`
//! (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:167-188`,
//! which passes the adapter's locale through `parse(value, format, new Date(), { locale })`).
//! The parser table transcribes `date-fns/parse/_lib/parsers/*` + `utils.js`; the
//! two-pass tokenizer and the setter machine transcribe `parse.js` itself.
//!
//! Rust adaptations (behavior-preserving where the upstream contract is defined):
//! - The regex numeric/timezone patterns become ordered slot sequences with the same
//!   greedy-then-backtrack preference (`parse.js` matches them with `String.match`).
//! - `RangeError`s (incompatible tokens, unescaped latin characters) become panics with
//!   the same messages; a failed match anywhere yields the port's Invalid Date `None`.
//! - The `DateTimezoneSetter` head of the setter list (`Setter.js:28-45`) is a no-op in
//!   the port: the wall-clock setters already operate in the value's zone, so the
//!   `transpose` re-construction has nothing to change. Its priority/subpriority slot is
//!   kept because it decides which setter wins the priority-10 group when no timezone
//!   token parsed.
//! - `DayOfYearParser`'s upstream `subpriority = 1` typo (lowercase field, so the
//!   subPriority stays 0) is preserved — it decides the `d`+`D` group winner.

use crate::date_fns_calendar::{
    get_iso_week, get_timezone_offset_millis, get_week_year, set_day, set_iso_day, set_iso_week,
    set_week, start_of_iso_week, start_of_week,
};
use crate::date_fns_format::{clean_escaped_string, tokenize_long_formatters, tokenize_tokens};
use crate::date_fns_locale::{DateFnsLocale, DayPeriod, Width};
use crate::temporal::DateValue;

/// `parse(dateStr, formatStr, referenceDate, { locale })` (`date-fns/parse.js`).
pub(crate) fn parse(
    value: &str,
    format_str: &str,
    reference: &DateValue,
    locale: &'static DateFnsLocale,
) -> Option<DateValue> {
    if format_str.is_empty() {
        // `if (!formatStr) return dateStr ? invalidDate() : toDate(referenceDate)`.
        return if value.is_empty() { Some(reference.clone()) } else { None };
    }
    let week_starts_on = locale.week_starts_on;
    let first_week_contains_date = locale.first_week_contains_date;

    let expanded = tokenize_long_formatters(format_str, locale).join("");
    let tokens = tokenize_tokens(&expanded);

    let mut rest = value.to_string();
    let mut setters: Vec<Setter> = vec![Setter::ContextZone];
    let mut used_tokens: Vec<(char, String)> = Vec::new();

    for token in &tokens {
        let first = token.chars().next().unwrap_or('\0');
        if let Some(descriptor) = parser_for(first) {
            // The incompatible-token guards (`parse.js`).
            if let Some(incompatible) = descriptor.incompatible {
                if incompatible == "*" {
                    if !used_tokens.is_empty() {
                        panic!(
                            "The format string mustn't contain `{token}` and any other token at the same time"
                        );
                    }
                } else if let Some((_, earlier)) = used_tokens.iter().rev().find(|(used, full)| {
                    incompatible.contains(*used) || *used == first
                }) {
                    panic!(
                        "The format string mustn't contain `{earlier}` and `{token}` at the same time"
                    );
                }
            }
            used_tokens.push((first, token.clone()));
            let (parsed, remainder) = (descriptor.run)(value_or_empty(&rest), token, locale)?;
            setters.push(parsed);
            rest = remainder;
        } else {
            if first.is_ascii_alphabetic() {
                panic!("Format string contains an unescaped latin alphabet character `{first}`");
            }
            let literal = if token == "''" {
                "'".to_string()
            } else if first == '\'' {
                clean_escaped_string(token)
            } else {
                token.clone()
            };
            if rest.starts_with(&literal) {
                rest = rest[literal.len()..].to_string();
            } else {
                return None;
            }
        }
    }

    // The remaining input must be whitespace.
    if rest.chars().any(|c| !is_js_space(c)) {
        return None;
    }

    // `uniquePrioritySetters` — one setter per priority group, the highest subPriority
    // (stable insertion order on ties).
    let mut priorities: Vec<i64> = setters.iter().map(|s| s.priority()).collect();
    priorities.sort_unstable_by(|a, b| b.cmp(a));
    priorities.dedup();
    let mut date = reference.clone();
    let mut flags = Flags::default();
    for priority in priorities {
        let winner = setters
            .iter()
            .filter(|s| s.priority() == priority)
            .enumerate()
            .max_by_key(|(index, s)| (s.sub_priority(), std::cmp::Reverse(*index)))
            .map(|(_, s)| s)
            .expect("group is non-empty");
        if !winner.validate(&date, week_starts_on) {
            return None;
        }
        date = winner.apply(&date, &mut flags, week_starts_on, first_week_contains_date);
    }
    Some(date)
}

fn value_or_empty(s: &str) -> &str {
    s
}

/// The JS `\s` test (the subset the inputs realistically carry; `parse.js` uses `/\S/`).
fn is_js_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{85}' | '\u{a0}')
}

#[derive(Default)]
struct Flags {
    timestamp_is_set: bool,
    era: Option<i64>,
}

// ─── parser descriptors ──────────────────────────────────────────────────────────────

type RunFn = fn(&str, &str, &'static DateFnsLocale) -> Option<(Setter, String)>;

struct Descriptor {
    run: RunFn,
    /// `incompatibleTokens`; `Some("*")` for the timestamp parsers.
    incompatible: Option<&'static str>,
}

fn parser_for(first: char) -> Option<Descriptor> {
    let descriptor = |run: RunFn, incompatible: Option<&'static str>| Descriptor { run, incompatible };
    match first {
        'G' => Some(descriptor(run_era, Some("RutT"))),
        'y' => Some(descriptor(run_year, Some("YRuwIiectT"))),
        'Y' => Some(descriptor(run_local_week_year, Some("yRuQqMLIdDitT"))),
        'R' => Some(descriptor(run_iso_week_year, Some("GyYuQqMLwdDectT"))),
        'u' => Some(descriptor(run_extended_year, Some("GyYRwIiectT"))),
        'Q' => Some(descriptor(run_quarter, Some("YRqMLwIdDectT"))),
        'q' => Some(descriptor(run_quarter, Some("YRQMLwIdDectT"))),
        'M' => Some(descriptor(run_month, Some("YRqQLwIDiectT"))),
        'L' => Some(descriptor(run_month, Some("YRqQMwIDiectT"))),
        'w' => Some(descriptor(run_local_week, Some("yRuqQMLIdDitT"))),
        'I' => Some(descriptor(run_iso_week, Some("yYuqQMLwdDectT"))),
        'd' => Some(descriptor(run_date, Some("YRqQwIDiectT"))),
        'D' => Some(descriptor(run_day_of_year, Some("YRqQMLwIdEiectT"))),
        'E' => Some(descriptor(run_day, Some("DiectT"))),
        'e' => Some(descriptor(run_local_day, Some("yRuqQMLIdEictT"))),
        'c' => Some(descriptor(run_local_day, Some("yRuqQMLIdEietT"))),
        'i' => Some(descriptor(run_iso_day, Some("yYuqQMLwdDEectT"))),
        'a' => Some(descriptor(run_day_period, Some("bBHktT"))),
        'b' => Some(descriptor(run_day_period, Some("aBHktT"))),
        'B' => Some(descriptor(run_day_period, Some("abtT"))),
        'h' => Some(descriptor(run_hour12, Some("HKktT"))),
        'H' => Some(descriptor(run_hour23, Some("abhKktT"))),
        'K' => Some(descriptor(run_hour11, Some("hHktT"))),
        'k' => Some(descriptor(run_hour24, Some("abhHKtT"))),
        'm' => Some(descriptor(run_minute, Some("tT"))),
        's' => Some(descriptor(run_second, Some("tT"))),
        'S' => Some(descriptor(run_fraction, Some("tT"))),
        'X' => Some(descriptor(run_iso_tz_with_z, Some("tTx"))),
        'x' => Some(descriptor(run_iso_tz, Some("tTX"))),
        't' => Some(descriptor(run_timestamp_seconds, Some("*"))),
        'T' => Some(descriptor(run_timestamp_millis, Some("*"))),
        _ => None,
    }
}

// ─── setters ─────────────────────────────────────────────────────────────────────────

/// The `ValueSetter` variants (`parsers` table) with their `priority`/`subPriority`.
#[derive(Clone, Debug)]
enum Setter {
    /// The head `DateTimezoneSetter` (priority 10, subPriority -1): a no-op in the port
    /// beyond the `timestampIsSet` guard (see the module docs).
    ContextZone,
    Era { value: i64 },
    Year { year: i64, two_digit: bool },
    LocalWeekYear { year: i64, two_digit: bool },
    IsoWeekYear { year: i64 },
    ExtendedYear { year: i64 },
    Quarter { value: i64 },
    Month { value: i64 },
    LocalWeek { value: i64 },
    IsoWeek { value: i64 },
    Date { value: i64 },
    DayOfYear { value: i64 },
    Day { value: i64 },
    IsoDay { value: i64 },
    DayPeriod { value: DayPeriod },
    Hour12 { value: i64 },
    Hour23 { value: i64 },
    Hour11 { value: i64 },
    Hour24 { value: i64 },
    Minute { value: i64 },
    Second { value: i64 },
    TimestampSeconds { value: i64 },
    Fraction { value: i64 },
    TimestampMillis { value: i64 },
    IsoTz { value: i64 },
}

impl Setter {
    fn priority(&self) -> i64 {
        match self {
            Setter::ContextZone => 10,
            Setter::Era { .. } => 140,
            Setter::Year { .. } | Setter::LocalWeekYear { .. } | Setter::IsoWeekYear { .. }
            | Setter::ExtendedYear { .. } => 130,
            Setter::Quarter { .. } => 120,
            Setter::Month { .. } => 110,
            Setter::LocalWeek { .. } | Setter::IsoWeek { .. } => 100,
            Setter::Date { .. } | Setter::DayOfYear { .. } | Setter::Day { .. }
            | Setter::IsoDay { .. } => 90,
            Setter::DayPeriod { .. } => 80,
            Setter::Hour12 { .. } | Setter::Hour23 { .. } | Setter::Hour11 { .. }
            | Setter::Hour24 { .. } => 70,
            Setter::Minute { .. } => 60,
            Setter::Second { .. } => 50,
            Setter::TimestampSeconds { .. } => 40,
            Setter::Fraction { .. } => 30,
            Setter::TimestampMillis { .. } => 20,
            Setter::IsoTz { .. } => 10,
        }
    }

    fn sub_priority(&self) -> i64 {
        match self {
            // `DateParser` declares `subPriority = 1`; `DayOfYearParser`'s lowercase
            // `subpriority` typo leaves it at 0.
            Setter::Date { .. } => 1,
            Setter::ContextZone => -1,
            _ => 0,
        }
    }

    fn validate(&self, date: &DateValue, _week_starts_on: i64) -> bool {
        match self {
            Setter::Year { year, two_digit } | Setter::LocalWeekYear { year, two_digit } => {
                *two_digit || *year > 0
            }
            Setter::Quarter { value } => (1..=4).contains(value),
            Setter::Month { value } => (0..=11).contains(value),
            Setter::LocalWeek { value } | Setter::IsoWeek { value } => (1..=53).contains(value),
            Setter::Date { value } => {
                let year = date.wall().year();
                let leap = year % 400 == 0 || (year % 4 == 0 && year % 100 != 0);
                let month0 = i64::from(date.wall().month()) - 1;
                let days_in_month = crate::date_fns_calendar::days_in_month_rolled(i64::from(year), month0);
                *value >= 1 && *value <= days_in_month
            }
            Setter::DayOfYear { value } => {
                let year = date.wall().year();
                let leap = year % 400 == 0 || (year % 4 == 0 && year % 100 != 0);
                if leap {
                    (1..=366).contains(value)
                } else {
                    (1..=365).contains(value)
                }
            }
            Setter::Day { value } => (0..=6).contains(value),
            Setter::IsoDay { value } => (1..=7).contains(value),
            Setter::Hour12 { value } => (1..=12).contains(value),
            Setter::Hour23 { value } => (0..=23).contains(value),
            Setter::Hour11 { value } => (0..=11).contains(value),
            Setter::Hour24 { value } => (1..=24).contains(value),
            Setter::Minute { value } | Setter::Second { value } => (0..=59).contains(value),
            _ => true,
        }
    }

    fn apply(
        &self,
        date: &DateValue,
        flags: &mut Flags,
        week_starts_on: i64,
        first_week_contains_date: i64,
    ) -> DateValue {
        let midnight = |d: DateValue| d.set_wall_time(0, 0, 0, 0);
        match self {
            Setter::ContextZone => {
                // `if (flags.timestampIsSet) return date;` — the transpose is a no-op.
                date.clone()
            }
            Setter::Era { value } => {
                flags.era = Some(*value);
                midnight(date.set_wall_ymd(*value, 0, 1))
            }
            Setter::Year { year, two_digit } => {
                let year = if *two_digit {
                    normalize_two_digit_year(*year, i64::from(date.wall().year()))
                } else if flags.era.is_none_or(|era| era == 1) {
                    *year
                } else {
                    1 - *year
                };
                midnight(date.set_wall_ymd(year, 0, 1))
            }
            Setter::LocalWeekYear { year, two_digit } => {
                let current = get_week_year(date, week_starts_on, first_week_contains_date);
                let year = if *two_digit {
                    normalize_two_digit_year(*year, current)
                } else if flags.era.is_none_or(|era| era == 1) {
                    *year
                } else {
                    1 - *year
                };
                start_of_week(
                    &midnight(date.set_wall_ymd(year, 0, first_week_contains_date)),
                    week_starts_on,
                )
            }
            Setter::IsoWeekYear { year } => start_of_iso_week(&midnight(date.set_wall_ymd(
                *year,
                0,
                4,
            ))),
            Setter::ExtendedYear { year } => midnight(date.set_wall_ymd(*year, 0, 1)),
            Setter::Quarter { value } => {
                midnight(date.set_wall_month((*value - 1) * 3, 1))
            }
            Setter::Month { value } => midnight(date.set_wall_month(*value, 1)),
            Setter::LocalWeek { value } => start_of_week(
                &set_week(date, *value, week_starts_on, first_week_contains_date),
                week_starts_on,
            ),
            Setter::IsoWeek { value } => start_of_iso_week(&set_iso_week(date, *value)),
            Setter::Date { value } => midnight(date.set_wall_day(*value)),
            Setter::DayOfYear { value } => midnight(date.set_wall_month(0, *value)),
            Setter::Day { value } => midnight(set_day(date, *value, week_starts_on)),
            Setter::IsoDay { value } => midnight(set_iso_day(date, *value)),
            Setter::DayPeriod { value } => {
                date.set_wall_time(day_period_enum_to_hours(*value), 0, 0, 0)
            }
            Setter::Hour12 { value } => {
                let is_pm = date.wall().hour() >= 12;
                let hours = if is_pm && *value < 12 {
                    *value + 12
                } else if !is_pm && *value == 12 {
                    0
                } else {
                    *value
                };
                date.set_wall_time(hours, 0, 0, 0)
            }
            Setter::Hour23 { value } => date.set_wall_time(*value, 0, 0, 0),
            Setter::Hour11 { value } => {
                let is_pm = date.wall().hour() >= 12;
                let hours = if is_pm && *value < 12 { *value + 12 } else { *value };
                date.set_wall_time(hours, 0, 0, 0)
            }
            Setter::Hour24 { value } => {
                let hours = if *value <= 24 { *value % 24 } else { *value };
                date.set_wall_time(hours, 0, 0, 0)
            }
            Setter::Minute { value } => date.set_wall_minutes(*value, 0, 0),
            Setter::Second { value } => date.set_wall_seconds(*value, 0),
            Setter::TimestampSeconds { value } => DateValue {
                millis: value * 1_000,
                zone: date.zone().clone(),
            },
            Setter::Fraction { value } => date.set_wall_milliseconds(*value),
            Setter::TimestampMillis { value } => DateValue {
                millis: *value,
                zone: date.zone().clone(),
            },
            Setter::IsoTz { value } => {
                if flags.timestamp_is_set {
                    date.clone()
                } else {
                    DateValue {
                        millis: date.millis() - get_timezone_offset_millis(date) - *value,
                        zone: date.zone().clone(),
                    }
                }
            }
        }
    }
}

// ─── numeric pattern machinery ───────────────────────────────────────────────────────

/// One slot of a transcribed numeric regex: a required or optional single-character
/// class. Optional slots are greedy and backtrack (the regex `?`).
#[derive(Clone, Copy)]
enum Slot {
    Required(Class),
    Optional(Class),
}

#[derive(Clone, Copy)]
enum Class {
    Digit,
    Char(char),
    Range(char, char),
}

impl Class {
    fn matches(self, c: char) -> bool {
        match self {
            Class::Digit => c.is_ascii_digit(),
            Class::Char(expected) => c == expected,
            Class::Range(lo, hi) => c >= lo && c <= hi,
        }
    }
}

const DIGIT: Class = Class::Digit;

/// Matches the slots at the input start with regex backtracking (optional slots present
/// first), returning the consumed length.
fn match_slots(slots: &[Slot], chars: &[char]) -> Option<usize> {
    let Some((first, rest)) = slots.split_first() else {
        return Some(0);
    };
    let (class, optional) = match first {
        Slot::Required(class) => (*class, false),
        Slot::Optional(class) => (*class, true),
    };
    if let Some(&c) = chars.first() {
        if class.matches(c) {
            if let Some(n) = match_slots(rest, &chars[1..]) {
                return Some(n + 1);
            }
        }
    }
    if optional {
        match_slots(rest, chars)
    } else {
        None
    }
}

fn try_alternatives(alternatives: &[&[Slot]], chars: &[char]) -> Option<usize> {
    alternatives.iter().find_map(|slots| match_slots(slots, chars))
}

/// `parseNumericPattern` (`date-fns/parse/_lib/utils.js` over `numericPatterns`).
fn parse_numeric_pattern(pattern: NumericPattern, input: &str) -> Option<(i64, String)> {
    let chars: Vec<char> = input.chars().collect();
    let consumed = match pattern {
        // ^(1[0-2]|0?\d)
        NumericPattern::Month => try_alternatives(
            &[
                &[Slot::Required(Class::Char('1')), Slot::Required(Class::Range('0', '2'))],
                &[Slot::Optional(Class::Char('0')), Slot::Required(DIGIT)],
            ],
            &chars,
        ),
        // ^(3[0-1]|[0-2]?\d)
        NumericPattern::Date => try_alternatives(
            &[
                &[Slot::Required(Class::Char('3')), Slot::Required(Class::Range('0', '1'))],
                &[Slot::Optional(Class::Range('0', '2')), Slot::Required(DIGIT)],
            ],
            &chars,
        ),
        // ^(36[0-6]|3[0-5]\d|[0-2]?\d?\d)
        NumericPattern::DayOfYear => try_alternatives(
            &[
                &[
                    Slot::Required(Class::Char('3')),
                    Slot::Required(Class::Char('6')),
                    Slot::Required(Class::Range('0', '6')),
                ],
                &[
                    Slot::Required(Class::Char('3')),
                    Slot::Required(Class::Range('0', '5')),
                    Slot::Required(DIGIT),
                ],
                &[
                    Slot::Optional(Class::Range('0', '2')),
                    Slot::Optional(DIGIT),
                    Slot::Required(DIGIT),
                ],
            ],
            &chars,
        ),
        // ^(5[0-3]|[0-4]?\d)
        NumericPattern::Week => try_alternatives(
            &[
                &[Slot::Required(Class::Char('5')), Slot::Required(Class::Range('0', '3'))],
                &[Slot::Optional(Class::Range('0', '4')), Slot::Required(DIGIT)],
            ],
            &chars,
        ),
        // ^(2[0-3]|[0-1]?\d)
        NumericPattern::Hour23h => try_alternatives(
            &[
                &[Slot::Required(Class::Char('2')), Slot::Required(Class::Range('0', '3'))],
                &[Slot::Optional(Class::Range('0', '1')), Slot::Required(DIGIT)],
            ],
            &chars,
        ),
        // ^(2[0-4]|[0-1]?\d)
        NumericPattern::Hour24h => try_alternatives(
            &[
                &[Slot::Required(Class::Char('2')), Slot::Required(Class::Range('0', '4'))],
                &[Slot::Optional(Class::Range('0', '1')), Slot::Required(DIGIT)],
            ],
            &chars,
        ),
        // ^(1[0-1]|0?\d)
        NumericPattern::Hour11h => try_alternatives(
            &[
                &[Slot::Required(Class::Char('1')), Slot::Required(Class::Range('0', '1'))],
                &[Slot::Optional(Class::Char('0')), Slot::Required(DIGIT)],
            ],
            &chars,
        ),
        // ^(1[0-2]|0?\d)
        NumericPattern::Hour12h => try_alternatives(
            &[
                &[Slot::Required(Class::Char('1')), Slot::Required(Class::Range('0', '2'))],
                &[Slot::Optional(Class::Char('0')), Slot::Required(DIGIT)],
            ],
            &chars,
        ),
        // ^[0-5]?\d
        NumericPattern::Minute | NumericPattern::Second => try_alternatives(
            &[&[Slot::Optional(Class::Range('0', '5')), Slot::Required(DIGIT)]],
            &chars,
        ),
    }?;
    let matched: String = chars[..consumed].iter().collect();
    Some((matched.parse().ok()?, chars[consumed..].iter().collect()))
}

#[derive(Clone, Copy)]
enum NumericPattern {
    Month,
    Date,
    DayOfYear,
    Week,
    Hour23h,
    Hour24h,
    Hour11h,
    Hour12h,
    Minute,
    Second,
}

/// `parseNDigits(n)` — `/^\d{1,n}/`.
fn parse_n_digits(n: usize, input: &str) -> Option<(i64, String)> {
    let chars: Vec<char> = input.chars().collect();
    let count = chars.iter().take(n).take_while(|c| c.is_ascii_digit()).count();
    if count == 0 {
        return None;
    }
    let matched: String = chars[..count].iter().collect();
    Some((matched.parse().ok()?, chars[count..].iter().collect()))
}

/// `parseAnyDigitsSigned` — `/^-?\d+/`.
fn parse_any_digits_signed(input: &str) -> Option<(i64, String)> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    if chars.first() == Some(&'-') {
        i = 1;
    }
    let digits = chars[i..].iter().take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let matched: String = chars[..i + digits].iter().collect();
    Some((matched.parse().ok()?, chars[i + digits..].iter().collect()))
}

/// `parseNDigitsSigned(n)` — `/^-?\d{1,n}/`.
fn parse_n_digits_signed(n: usize, input: &str) -> Option<(i64, String)> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;
    if chars.first() == Some(&'-') {
        i = 1;
    }
    let digits = chars[i..].iter().take(n).take_while(|c| c.is_ascii_digit()).count();
    if digits == 0 {
        return None;
    }
    let matched: String = chars[..i + digits].iter().collect();
    Some((matched.parse().ok()?, chars[i + digits..].iter().collect()))
}

/// The timezone patterns (`timezonePatterns` in `utils.js`): `Z` or a signed offset.
fn parse_timezone_pattern(pattern: TzPattern, input: &str) -> Option<(i64, String)> {
    let chars: Vec<char> = input.chars().collect();
    if chars.first() == Some(&'Z') {
        return Some((0, chars[1..].iter().collect()));
    }
    let sign = match chars.first()? {
        '+' => 1,
        '-' => -1,
        _ => return None,
    };
    let digits_at = |from: usize, count: usize| -> Option<i64> {
        let slice: String = chars.get(from..from + count)?.iter().collect();
        if slice.chars().count() == count && slice.chars().all(|c| c.is_ascii_digit()) {
            slice.parse().ok()
        } else {
            None
        }
    };
    let hours = digits_at(1, 2)?;
    // `([+-])(\d{2})(:?\d{2})...` — the minutes follow directly (`basic*`) or after a
    // colon (`extended*`); only `basicOptionalMinutes` makes them optional.
    let colon = matches!(pattern, TzPattern::Extended | TzPattern::ExtendedOptionalSeconds);
    if colon && chars.get(3) != Some(&':') {
        return None;
    }
    let minutes_start = if colon { 4 } else { 3 };
    let minutes_optional = matches!(pattern, TzPattern::BasicOptionalMinutes);
    let (minutes, mut consumed) = match digits_at(minutes_start, 2) {
        Some(minutes) => (minutes, minutes_start + 2),
        None if minutes_optional => (0, minutes_start),
        None => return None,
    };
    // Optional seconds: `((\d{2}))?` directly after the minutes, or `:(\d{2})`.
    let seconds = match pattern {
        TzPattern::BasicOptionalSeconds if consumed == 5 => match digits_at(5, 2) {
            Some(seconds) => {
                consumed = 7;
                seconds
            }
            None => 0,
        },
        TzPattern::ExtendedOptionalSeconds if consumed == 6 && chars.get(6) == Some(&':') => {
            match digits_at(7, 2) {
                Some(seconds) => {
                    consumed = 9;
                    seconds
                }
                None => 0,
            }
        }
        _ => 0,
    };
    Some((
        sign * (hours * 3_600_000 + minutes * 60_000 + seconds * 1_000),
        chars[consumed..].iter().collect(),
    ))
}

#[derive(Clone, Copy)]
enum TzPattern {
    BasicOptionalMinutes,
    Basic,
    BasicOptionalSeconds,
    Extended,
    ExtendedOptionalSeconds,
}

/// `dayPeriodEnumToHours` (`utils.js`).
fn day_period_enum_to_hours(period: DayPeriod) -> i64 {
    match period {
        DayPeriod::Morning => 4,
        DayPeriod::Evening => 17,
        DayPeriod::Pm | DayPeriod::Noon | DayPeriod::Afternoon => 12,
        DayPeriod::Am | DayPeriod::Midnight | DayPeriod::Night => 0,
    }
}

/// `normalizeTwoDigitYear` (`utils.js`).
fn normalize_two_digit_year(two_digit_year: i64, current_year: i64) -> i64 {
    let is_common_era = current_year > 0;
    let abs_current_year = if is_common_era { current_year } else { 1 - current_year };
    let result = if abs_current_year <= 50 {
        if two_digit_year == 0 { 100 } else { two_digit_year }
    } else {
        let range_end = abs_current_year + 50;
        let range_end_century = (range_end / 100) * 100;
        let is_previous_century = two_digit_year >= range_end % 100;
        two_digit_year + range_end_century - i64::from(is_previous_century) * 100
    };
    if is_common_era {
        result
    } else {
        1 - result
    }
}

// ─── parser runs ─────────────────────────────────────────────────────────────────────
//
// One `run` per parser: `(dateString, token, locale) -> (setter, rest)` — the
// `Parser.run` contract (`parse/_lib/Parser.js:4-20`), with the locale-carrying
// `match` half inlined (the `subFnOptions` bag upstream).

fn run_era(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let era = |width| locale.match_era(value, Some(width));
    let parsed = match token {
        "G" | "GG" | "GGG" => era(Width::Abbreviated).or_else(|| era(Width::Narrow)),
        "GGGGG" => era(Width::Narrow),
        _ => era(Width::Wide).or_else(|| era(Width::Abbreviated)).or_else(|| era(Width::Narrow)),
    }?;
    Some((Setter::Era { value: parsed.0 }, parsed.1))
}

fn run_year(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (year, rest) = match token {
        "y" => parse_n_digits(4, value)?,
        "yo" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Year { year, two_digit: token == "yy" }, rest))
}

fn run_local_week_year(
    value: &str,
    token: &str,
    locale: &'static DateFnsLocale,
) -> Option<(Setter, String)> {
    let (year, rest) = match token {
        "Y" => parse_n_digits(4, value)?,
        "Yo" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::LocalWeekYear { year, two_digit: token == "YY" }, rest))
}

fn run_iso_week_year(
    value: &str,
    token: &str,
    _locale: &'static DateFnsLocale,
) -> Option<(Setter, String)> {
    let (year, rest) = if token == "R" {
        parse_n_digits_signed(4, value)?
    } else {
        parse_n_digits_signed(token.chars().count(), value)?
    };
    Some((Setter::IsoWeekYear { year }, rest))
}

fn run_extended_year(
    value: &str,
    token: &str,
    _locale: &'static DateFnsLocale,
) -> Option<(Setter, String)> {
    let (year, rest) = if token == "u" {
        parse_n_digits_signed(4, value)?
    } else {
        parse_n_digits_signed(token.chars().count(), value)?
    };
    Some((Setter::ExtendedYear { year }, rest))
}

fn run_quarter(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (quarter, rest) = match token {
        "Q" | "QQ" | "q" | "qq" => parse_n_digits(token.chars().count(), value)?,
        "Qo" | "qo" => locale.match_ordinal_number(value)?,
        "QQQ" | "qqq" => locale
            .match_quarter(value, Some(Width::Abbreviated))
            .or_else(|| locale.match_quarter(value, Some(Width::Narrow)))?,
        "QQQQQ" | "qqqqq" => locale.match_quarter(value, Some(Width::Narrow))?,
        _ => locale
            .match_quarter(value, Some(Width::Wide))
            .or_else(|| locale.match_quarter(value, Some(Width::Abbreviated)))
            .or_else(|| locale.match_quarter(value, Some(Width::Narrow)))?,
    };
    Some((Setter::Quarter { value: quarter }, rest))
}

fn run_month(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    // The `value - 1` callback applies only to the numeric/ordinal arms (which parse
    // 1-12); the name matchers return the 0-based month index directly.
    let (value, rest) = match token {
        "M" | "L" => {
            let (value, rest) = parse_numeric_pattern(NumericPattern::Month, value)?;
            (value - 1, rest)
        }
        "MM" | "LL" => {
            let (value, rest) = parse_n_digits(2, value)?;
            (value - 1, rest)
        }
        "Mo" | "Lo" => {
            let (value, rest) = locale.match_ordinal_number(value)?;
            (value - 1, rest)
        }
        "MMM" | "LLL" => locale
            .match_month(value, Some(Width::Abbreviated))
            .or_else(|| locale.match_month(value, Some(Width::Narrow)))?,
        "MMMMM" | "LLLLL" => locale.match_month(value, Some(Width::Narrow))?,
        _ => locale
            .match_month(value, Some(Width::Wide))
            .or_else(|| locale.match_month(value, Some(Width::Abbreviated)))
            .or_else(|| locale.match_month(value, Some(Width::Narrow)))?,
    };
    Some((Setter::Month { value }, rest))
}

fn run_local_week(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (week, rest) = match token {
        "w" => parse_numeric_pattern(NumericPattern::Week, value)?,
        "wo" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::LocalWeek { value: week }, rest))
}

fn run_iso_week(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (week, rest) = match token {
        "I" => parse_numeric_pattern(NumericPattern::Week, value)?,
        "Io" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::IsoWeek { value: week }, rest))
}

fn run_date(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (date, rest) = match token {
        "d" => parse_numeric_pattern(NumericPattern::Date, value)?,
        "do" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Date { value: date }, rest))
}

fn run_day_of_year(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (day, rest) = match token {
        "D" | "DD" => parse_numeric_pattern(NumericPattern::DayOfYear, value)?,
        // The upstream `unit: "date"` for `Do` (DayOfYearParser).
        "Do" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::DayOfYear { value: day }, rest))
}

fn run_day(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let day = |width| locale.match_day(value, Some(width));
    let (day, rest) = match token {
        "E" | "EE" | "EEE" => {
            day(Width::Abbreviated)
                .or_else(|| day(Width::Short))
                .or_else(|| day(Width::Narrow))?
        }
        "EEEEE" => day(Width::Narrow)?,
        "EEEEEE" => day(Width::Short).or_else(|| day(Width::Narrow))?,
        _ => day(Width::Wide)
            .or_else(|| day(Width::Abbreviated))
            .or_else(|| day(Width::Short))
            .or_else(|| day(Width::Narrow))?,
    };
    Some((Setter::Day { value: day }, rest))
}

fn run_local_day(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    // The valueCallback folds the locale day-of-week into the JS getDay range:
    // `(value + weekStartsOn + 6) % 7 + floor((value - 1) / 7) * 7`.
    let callback = |value: i64| {
        let whole_week_days = ((value - 1).div_euclid(7)) * 7;
        (value + locale.week_starts_on + 6).rem_euclid(7) + whole_week_days
    };
    let (day, rest) = match token {
        "e" | "ee" | "c" | "cc" => parse_n_digits(token.chars().count(), value)?,
        "eo" | "co" => locale.match_ordinal_number(value)?,
        "eee" | "ccc" => locale
            .match_day(value, Some(Width::Abbreviated))
            .or_else(|| locale.match_day(value, Some(Width::Short)))
            .or_else(|| locale.match_day(value, Some(Width::Narrow)))?,
        "eeeee" | "ccccc" => locale.match_day(value, Some(Width::Narrow))?,
        "eeeeee" | "cccccc" => locale
            .match_day(value, Some(Width::Short))
            .or_else(|| locale.match_day(value, Some(Width::Narrow)))?,
        _ => locale
            .match_day(value, Some(Width::Wide))
            .or_else(|| locale.match_day(value, Some(Width::Abbreviated)))
            .or_else(|| locale.match_day(value, Some(Width::Short)))
            .or_else(|| locale.match_day(value, Some(Width::Narrow)))?,
    };
    Some((Setter::Day { value: callback(day) }, rest))
}

fn run_iso_day(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    // `valueCallback: (value) => value === 0 ? 7 : value` — the name matchers only; the
    // numeric `i`/`ii`/`io` parse paths return the raw value (a parsed `0` then fails
    // the 1..=7 validation, exactly as upstream).
    let (day, rest) = match token {
        "i" | "ii" => parse_n_digits(token.chars().count(), value)?,
        "io" => locale.match_ordinal_number(value)?,
        "iii" => locale
            .match_day(value, Some(Width::Abbreviated))
            .or_else(|| locale.match_day(value, Some(Width::Short)))
            .or_else(|| locale.match_day(value, Some(Width::Narrow)))?,
        "iiiii" => locale.match_day(value, Some(Width::Narrow))?,
        "iiiiii" => locale
            .match_day(value, Some(Width::Short))
            .or_else(|| locale.match_day(value, Some(Width::Narrow)))?,
        _ => locale
            .match_day(value, Some(Width::Wide))
            .or_else(|| locale.match_day(value, Some(Width::Abbreviated)))
            .or_else(|| locale.match_day(value, Some(Width::Short)))
            .or_else(|| locale.match_day(value, Some(Width::Narrow)))?,
    };
    let day = if matches!(token, "i" | "ii" | "io") { day } else if day == 0 { 7 } else { day };
    Some((Setter::IsoDay { value: day }, rest))
}

fn run_day_period(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let period = |width| locale.match_day_period(value, Some(width));
    let (period, rest) = match token {
        "a" | "aa" | "aaa" | "b" | "bb" | "bbb" | "B" | "BB" | "BBB" => period(Width::Abbreviated)
            .or_else(|| period(Width::Narrow))?,
        "aaaaa" | "bbbbb" | "BBBBB" => period(Width::Narrow)?,
        _ => period(Width::Wide)
            .or_else(|| period(Width::Abbreviated))
            .or_else(|| period(Width::Narrow))?,
    };
    Some((Setter::DayPeriod { value: period }, rest))
}

fn run_hour12(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (hour, rest) = match token {
        "h" => parse_numeric_pattern(NumericPattern::Hour12h, value)?,
        "ho" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Hour12 { value: hour }, rest))
}

fn run_hour23(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (hour, rest) = match token {
        "H" => parse_numeric_pattern(NumericPattern::Hour23h, value)?,
        "Ho" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Hour23 { value: hour }, rest))
}

fn run_hour11(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (hour, rest) = match token {
        "K" => parse_numeric_pattern(NumericPattern::Hour11h, value)?,
        "Ko" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Hour11 { value: hour }, rest))
}

fn run_hour24(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (hour, rest) = match token {
        "k" => parse_numeric_pattern(NumericPattern::Hour24h, value)?,
        "ko" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Hour24 { value: hour }, rest))
}

fn run_minute(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (minute, rest) = match token {
        "m" => parse_numeric_pattern(NumericPattern::Minute, value)?,
        "mo" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Minute { value: minute }, rest))
}

fn run_second(value: &str, token: &str, locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let (second, rest) = match token {
        "s" => parse_numeric_pattern(NumericPattern::Second, value)?,
        "so" => locale.match_ordinal_number(value)?,
        _ => parse_n_digits(token.chars().count(), value)?,
    };
    Some((Setter::Second { value: second }, rest))
}

fn run_fraction(value: &str, token: &str, _locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let token_length = token.chars().count();
    let (value, rest) = parse_n_digits(token_length, value)?;
    // `valueCallback: (value) => Math.trunc(value * Math.pow(10, -token.length + 3))`.
    let millis = (value as f64 * 10f64.powi(-(token_length as i32) + 3)).trunc() as i64;
    Some((Setter::Fraction { value: millis }, rest))
}

fn run_iso_tz_with_z(value: &str, token: &str, _locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let pattern = match token {
        "X" => TzPattern::BasicOptionalMinutes,
        "XX" => TzPattern::Basic,
        "XXXX" => TzPattern::BasicOptionalSeconds,
        "XXXXX" => TzPattern::ExtendedOptionalSeconds,
        _ => TzPattern::Extended,
    };
    let (offset, rest) = parse_timezone_pattern(pattern, value)?;
    Some((Setter::IsoTz { value: offset }, rest))
}

fn run_iso_tz(value: &str, token: &str, _locale: &'static DateFnsLocale) -> Option<(Setter, String)> {
    let pattern = match token {
        "x" => TzPattern::BasicOptionalMinutes,
        "xx" => TzPattern::Basic,
        "xxxx" => TzPattern::BasicOptionalSeconds,
        "xxxxx" => TzPattern::ExtendedOptionalSeconds,
        _ => TzPattern::Extended,
    };
    let (offset, rest) = parse_timezone_pattern(pattern, value)?;
    Some((Setter::IsoTz { value: offset }, rest))
}

fn run_timestamp_seconds(
    value: &str,
    _token: &str,
    _locale: &'static DateFnsLocale,
) -> Option<(Setter, String)> {
    let (timestamp, rest) = parse_any_digits_signed(value)?;
    Some((Setter::TimestampSeconds { value: timestamp }, rest))
}

fn run_timestamp_millis(
    value: &str,
    _token: &str,
    _locale: &'static DateFnsLocale,
) -> Option<(Setter, String)> {
    let (timestamp, rest) = parse_any_digits_signed(value)?;
    Some((Setter::TimestampMillis { value: timestamp }, rest))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::date_fns_locale::{EN_US, FR};
    use crate::temporal::{set_system_zone, Zone};

    fn reference() -> DateValue {
        DateValue::from_millis(1_577_891_289_000) // 2020-01-01T15:08:09Z, arbitrary fixed "now"
    }

    /// The wall clock of a parsed value as `YYYY-MM-DD HH:mm:ss.SSS` in its zone.
    fn wall_string(value: &DateValue) -> String {
        let wall = value.wall();
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            wall.year(),
            wall.month(),
            wall.day(),
            wall.hour(),
            wall.minute(),
            wall.second(),
            wall.millisecond()
        )
    }

    fn with_utc_zone<T>(run: impl FnOnce() -> T) -> T {
        set_system_zone(Some(jiff::tz::TimeZone::UTC));
        let out = run();
        set_system_zone(None);
        out
    }

    #[test]
    fn parses_the_harness_custom_string() {
        // testComputations.ts:104-117 — RFC5545 `yyyyMMdd'T'HHmmss'Z'` with quoted
        // literals; the trailing `Z` is a literal character, so the result is a system-
        // zone wall time.
        with_utc_zone(|| {
            let parsed = parse(
                "20181030T114400Z",
                "yyyyMMdd'T'HHmmss'Z'",
                &reference(),
                &EN_US,
            )
            .expect("parses");
            assert_eq!(wall_string(&parsed), "2018-10-30 11:44:00.000");
            assert_eq!(parsed.zone(), &Zone::System);
        });
    }

    #[test]
    fn parses_numeric_dates() {
        with_utc_zone(|| {
            let parsed = parse("2020-06-15", "yyyy-MM-dd", &reference(), &EN_US).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-06-15 00:00:00.000");
            let parsed = parse("06/15/2020", "MM/dd/yyyy", &reference(), &EN_US).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-06-15 00:00:00.000");
            let parsed = parse("2020-06-15 14:30:05.123", "yyyy-MM-dd HH:mm:ss.SSS", &reference(), &EN_US)
                .expect("parses");
            assert_eq!(wall_string(&parsed), "2020-06-15 14:30:05.123");
        });
    }

    #[test]
    fn parses_localized_names_and_ordinals() {
        with_utc_zone(|| {
            let parsed = parse("January 1st, 2020", "MMMM do, y", &reference(), &EN_US)
                .expect("parses");
            assert_eq!(wall_string(&parsed), "2020-01-01 00:00:00.000");
            let parsed = parse(
                "Wednesday, January 1st, 2020",
                "EEEE, MMMM do, y",
                &reference(),
                &EN_US,
            )
            .expect("parses");
            assert_eq!(wall_string(&parsed), "2020-01-01 00:00:00.000");
            // The abbreviated month with the optional fr trailing dot; "1er" is the
            // ordinal form, so the format needs the `do` token.
            let parsed = parse("15 juin 2020", "d MMMM y", &reference(), &FR).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-06-15 00:00:00.000");
            let parsed = parse("1er janvier 2020", "do MMMM y", &reference(), &FR).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-01-01 00:00:00.000");
        });
    }

    #[test]
    fn parses_twelve_hour_clock_with_day_periods() {
        with_utc_zone(|| {
            // The `a` setter (priority 80) runs before the hour setter (70).
            let parsed = parse("3:08 PM", "h:mm a", &reference(), &EN_US).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-01-01 15:08:00.000");
            let parsed = parse("12:00 AM", "h:mm a", &reference(), &EN_US).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-01-01 00:00:00.000");
            let parsed = parse("12:30 PM", "h:mm a", &reference(), &EN_US).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-01-01 12:30:00.000");
        });
    }

    #[test]
    fn parses_two_digit_years_via_the_current_year_window() {
        with_utc_zone(|| {
            let parsed = parse("6/15/20", "M/dd/yy", &reference(), &EN_US).expect("parses");
            assert_eq!(wall_string(&parsed), "2020-06-15 00:00:00.000");
        });
    }

    #[test]
    fn parses_iso_timezone_tokens() {
        with_utc_zone(|| {
            let parsed = parse(
                "2020-06-15 14:30+02:00",
                "yyyy-MM-dd HH:mmXXX",
                &reference(),
                &EN_US,
            )
            .expect("parses");
            // The wall fields are read as UTC and shifted by the parsed offset.
            assert_eq!(parsed.millis(), 1_592_224_200_000); // 2020-06-15T12:30Z
            let parsed = parse("2020-06-15 14:30Z", "yyyy-MM-dd HH:mmXX", &reference(), &EN_US)
                .expect("parses");
            assert_eq!(parsed.millis(), 1_592_231_400_000); // 14:30 as UTC
        });
    }

    #[test]
    fn returns_none_for_mismatches_and_out_of_range_days() {
        with_utc_zone(|| {
            assert!(parse("2020-02-30", "yyyy-MM-dd", &reference(), &EN_US).is_none());
            assert!(parse("2020-06-15x", "yyyy-MM-dd", &reference(), &EN_US).is_none());
            assert!(parse("Foo 2020", "MMMM yyyy", &reference(), &EN_US).is_none());
            assert!(parse("2020-06-15", "yyyy/MM/dd", &reference(), &EN_US).is_none());
            assert!(parse("25:00", "HH:mm", &reference(), &EN_US).is_none());
        });
    }

    #[test]
    fn handles_the_empty_format_rule() {
        with_utc_zone(|| {
            let parsed = parse("", "", &reference(), &EN_US).expect("the reference");
            assert_eq!(parsed.millis(), reference().millis());
            assert!(parse("2020", "", &reference(), &EN_US).is_none());
        });
    }

    #[test]
    fn parses_timestamp_tokens() {
        with_utc_zone(|| {
            let parsed = parse("1592220900", "t", &reference(), &EN_US).expect("parses");
            assert_eq!(parsed.millis(), 1_592_220_900_000);
            let parsed = parse("1592220900000", "T", &reference(), &EN_US).expect("parses");
            assert_eq!(parsed.millis(), 1_592_220_900_000);
        });
    }

    #[test]
    #[should_panic(expected = "The format string mustn't contain `y` and `Y`")]
    fn panics_on_incompatible_tokens() {
        with_utc_zone(|| {
            let _ = parse("2020 2020", "y Y", &reference(), &EN_US);
        });
    }



}
