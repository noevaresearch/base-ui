//! The date-fns locale tables the `TemporalAdapterDateFns` port reads — the Rust
//! counterpart of the upstream `date-fns/locale` objects the adapter is constructed with
//! (`packages/react/src/internals/temporal-adapter-date-fns/TemporalAdapterDateFns.ts:43-44,113-115`,
//! consumed at `startOfWeek`/`endOfWeek` `:288-291`/`:316-318`, `getWeekNumber` `:456-458`,
//! `getDayOfWeek` `:460-463`, and the parse/format `locale` plumbing `:167-171`/`:236-238`).
//!
//! A JS date-fns locale is an open-ended bag (`code`, `formatDistance`, `formatLong`,
//! `formatRelative`, `localize`, `match`, `options`). The adapter reads exactly five
//! things off it:
//!
//! - `code` (`getCurrentLocaleCode`, `:220-222`),
//! - `options.weekStartsOn` / `options.firstWeekContainsDate` (week math, `:460-463` /
//!   `getWeek`'s chain in `date-fns/getWeekYear.js:57-63` / `startOfWeekYear.js:45-51`),
//! - the month/weekday name tables the `match` half of parse uses,
//! - `formatLong.date` (the `P`→`PPPP` long-format expansion, `date-fns/_lib/format/longFormatters.js:1-16`),
//! - `localize.ordinalNumber` (the `do` token) and the fr-only `localize.preprocessor`
//!   (`format.js:364-366`).
//!
//! Rust adaptation: the open-ended locale registry collapses to the embedded tables
//! (`EN_US`, `FR`) — the default locale (`enUS`, `:114`) plus the localization-suite
//! locale the shared harness parameterizes with (`TemporalAdapterDateFns.test.ts:9`,
//! `adapterFr: new TemporalAdapterDateFns({ locale: fr })`). The tables are transcribed
//! from `node_modules/date-fns/locale/{en-US,fr}` (`_lib/localize.js`, `_lib/formatLong.js`,
//! the locale root `options`), so formatted output is byte-identical for the locales the
//! crate carries; a consumer needing a further locale extends [`DateFnsLocale`] the way a
//! JS consumer supplies a date-fns locale object.

/// The ordinal-number renderer (`localize.ordinalNumber`): `unit` is the
/// `localize.ordinalNumber(n, { unit })` argument (`'year'`, `'date'`, …).
pub type OrdinalFn = fn(i64, &str) -> String;

/// The fr-only `localize.preprocessor` hook
/// (`locale/fr/_lib/localize.js` — replaces `do` with `d` when the day of the month is
/// greater than one and a long month token is present: "1er août", "29 août"). The first
/// argument is the date's day of the month — the upstream closure captures the date.
pub type PreprocessFn = fn(i64, &[(bool, String)]) -> Vec<(bool, String)>;

/// One embedded date-fns locale (see the module docs for what the adapter reads).
pub struct DateFnsLocale {
    /// `locale.code` (`getCurrentLocaleCode`).
    pub code: &'static str,
    /// `locale.options.weekStartsOn` — 0 = Sunday, 1 = Monday.
    pub week_starts_on: i64,
    /// `locale.options.firstWeekContainsDate` — the day of January always in the first
    /// week of the week-numbering year.
    pub first_week_contains_date: i64,
    /// Month names by width: `(narrow, abbreviated, wide)`, January-first.
    pub months: Months,
    /// Weekday names by width: `(narrow, short, abbreviated, wide)`, Sunday-first (the
    /// JS `getDay` order the name lists use upstream).
    pub days: Days,
    /// `formatLong.date` widths (`P` = short, `PP` = medium, `PPP` = long, `PPPP` = full).
    pub date_formats: DateFormatLong,
    /// `localize.dayPeriod` am/pm strings by width — the `a` token
    /// (`locale/{en-US,fr}/_lib/localize.js` `dayPeriodValues`:
    /// `(abbreviated, narrow, wide)`).
    pub meridiem: Meridiem,
    /// `localize.ordinalNumber`.
    pub ordinal_number: OrdinalFn,
    /// The `match.ordinalNumber` suffix alternatives
    /// (`locale/{en-US,fr}/_lib/match.js` `matchOrdinalNumberPattern`), longest-first for
    /// prefix matching.
    pub ordinal_suffixes: &'static [&'static str],
    /// The optional `localize.preprocessor` (`format.js:364-366`); `None` for most locales.
    pub preprocessor: Option<PreprocessFn>,
}

/// `(narrow, abbreviated, wide)` month names, January-first
/// (`locale/en-US/_lib/localize.js` `monthValues` / `locale/fr/_lib/localize.js`).
pub struct Months {
    pub narrow: [&'static str; 12],
    pub abbreviated: [&'static str; 12],
    pub wide: [&'static str; 12],
}

/// `localize.dayPeriod` am/pm strings by width (`locale/_lib/localize.js`
/// `dayPeriodValues`): `(abbreviated, narrow, wide)`. fr renders `AM`/`PM` at every
/// width; en-US narrows to `a`/`p` and widens to `a.m.`/`p.m.`
pub struct Meridiem {
    pub abbreviated: (&'static str, &'static str),
    pub narrow: (&'static str, &'static str),
    pub wide: (&'static str, &'static str),
}

/// `(narrow, short, abbreviated, wide)` weekday names, Sunday-first
/// (`dayValues` in both locales' `_lib/localize.js`).
pub struct Days {
    pub narrow: [&'static str; 7],
    pub short: [&'static str; 7],
    pub abbreviated: [&'static str; 7],
    pub wide: [&'static str; 7],
}

/// `formatLong.date` (`locale/{en-US,fr}/_lib/formatLong.js` `dateFormats`).
pub struct DateFormatLong {
    /// `P` — `formatLong.date({ width: 'short' })`.
    pub short: &'static str,
    /// `PP` — medium.
    pub medium: &'static str,
    /// `PPP` — long.
    pub long: &'static str,
    /// `PPPP` — full.
    pub full: &'static str,
}

/// The `enUS` locale (`date-fns/locale/en-US`; the adapter default, `:114`).
pub static EN_US: DateFnsLocale = DateFnsLocale {
    code: "en-US",
    week_starts_on: 0, /* Sunday */
    first_week_contains_date: 1,
    months: Months {
        narrow: ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"],
        abbreviated: [
            "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
        ],
        wide: [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ],
    },
    days: Days {
        narrow: ["S", "M", "T", "W", "T", "F", "S"],
        short: ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"],
        abbreviated: ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"],
        wide: [
            "Sunday",
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
        ],
    },
    date_formats: DateFormatLong {
        short: "MM/dd/yyyy",
        medium: "MMM d, y",
        long: "MMMM do, y",
        full: "EEEE, MMMM do, y",
    },
    meridiem: Meridiem {
        abbreviated: ("AM", "PM"),
        narrow: ("a", "p"),
        wide: ("a.m.", "p.m."),
    },
    ordinal_number: en_us_ordinal,
    ordinal_suffixes: &["th", "st", "nd", "rd"],
    preprocessor: None,
};

/// The `fr` locale (`date-fns/locale/fr`; the harness's `adapterFr`).
pub static FR: DateFnsLocale = DateFnsLocale {
    code: "fr",
    week_starts_on: 1, /* Monday */
    first_week_contains_date: 4,
    months: Months {
        narrow: ["J", "F", "M", "A", "M", "J", "J", "A", "S", "O", "N", "D"],
        abbreviated: [
            "janv.", "févr.", "mars", "avr.", "mai", "juin", "juil.", "août", "sept.", "oct.",
            "nov.", "déc.",
        ],
        wide: [
            "janvier",
            "février",
            "mars",
            "avril",
            "mai",
            "juin",
            "juillet",
            "août",
            "septembre",
            "octobre",
            "novembre",
            "décembre",
        ],
    },
    days: Days {
        narrow: ["D", "L", "M", "M", "J", "V", "S"],
        short: ["di", "lu", "ma", "me", "je", "ve", "sa"],
        abbreviated: ["dim.", "lun.", "mar.", "mer.", "jeu.", "ven.", "sam."],
        wide: [
            "dimanche",
            "lundi",
            "mardi",
            "mercredi",
            "jeudi",
            "vendredi",
            "samedi",
        ],
    },
    date_formats: DateFormatLong {
        short: "dd/MM/y",
        medium: "d MMM y",
        long: "d MMMM y",
        full: "EEEE d MMMM y",
    },
    meridiem: Meridiem {
        abbreviated: ("AM", "PM"),
        narrow: ("AM", "PM"),
        wide: ("AM", "PM"),
    },
    ordinal_number: fr_ordinal,
    ordinal_suffixes: &["ième", "ère", "ème", "er", "e"],
    preprocessor: Some(fr_preprocess),
};

/// `en-US` `ordinalNumber` (`locale/en-US/_lib/localize.js`): `1st/2nd/3rd`, else `th`
/// (`11th`-`13th` included).
fn en_us_ordinal(number: i64, _unit: &str) -> String {
    let rem100 = number % 100;
    if rem100 > 20 || rem100 < 10 {
        match rem100 % 10 {
            1 => return format!("{number}st"),
            2 => return format!("{number}nd"),
            3 => return format!("{number}rd"),
            _ => {}
        }
    }
    format!("{number}th")
}

/// `fr` `ordinalNumber` (`locale/fr/_lib/localize.js`): `1er` (or `1ère` for the feminine
/// units year/week/hour/minute/second), else `ème`; `0` stays bare.
fn fr_ordinal(number: i64, unit: &str) -> String {
    if number == 0 {
        return "0".to_string();
    }
    const FEMININE_UNITS: [&str; 5] = ["year", "week", "hour", "minute", "second"];
    if number == 1 {
        let suffix = if FEMININE_UNITS.contains(&unit) {
            "ère"
        } else {
            "er"
        };
        return format!("{number}{suffix}");
    }
    format!("{number}ème")
}

/// The fr `preprocessor`: replaces the `do` token with `d` when the day of the month is
/// greater than one and a long month token (`MMM`/`MMMM`) is present — "1er août" but
/// "29 août" (`locale/fr/_lib/localize.js`, the `LONG_MONTHS_TOKENS` note).
fn fr_preprocess(day_of_month: i64, parts: &[(bool, String)]) -> Vec<(bool, String)> {
    if day_of_month == 1 {
        return parts.to_vec();
    }
    let has_long_month_token = parts
        .iter()
        .any(|(is_token, value)| *is_token && (value == "MMM" || value == "MMMM"));
    if !has_long_month_token {
        return parts.to_vec();
    }
    parts
        .iter()
        .map(|(is_token, value)| {
            if *is_token && value == "do" {
                (true, "d".to_string())
            } else {
                (*is_token, value.clone())
            }
        })
        .collect()
}
