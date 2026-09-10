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
//! `en-US/_lib/match.js` / `fr/_lib/match.js`, the locale root `options`), so formatted
//! output is byte-identical for the locales the crate carries; a consumer needing a
//! further locale extends [`DateFnsLocale`] the way a JS consumer supplies a date-fns
//! locale object.

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
    /// (`locale/{en-US,fr}/_lib/match.js` `matchOrdinalNumberPattern`), in the regex
    /// alternation order (the first suffix that matches wins).
    pub ordinal_suffixes: &'static [&'static str],
    /// The optional `localize.preprocessor` (`format.js:364-366`); `None` for most locales.
    pub preprocessor: Option<PreprocessFn>,
    /// `localize.era` (`_lib/localize.js` `eraValues`): `(narrow, abbreviated, wide)`,
    /// index 0 = Before Christ, 1 = Anno Domini.
    pub eras: Eras,
    /// `localize.quarter` (`_lib/localize.js` `quarterValues`): `(narrow, abbreviated,
    /// wide)`, 0-indexed.
    pub quarters: Quarters,
    /// `localize.dayPeriod` (`_lib/localize.js` `dayPeriodValues` +
    /// `formattingDayPeriodValues`): the standalone table (no `formattingValues` locales
    /// share it for both contexts) and the formatting table.
    pub day_periods: DayPeriodTables,
    /// `formatLong.time` widths (`p` = short, `pp` = medium, `ppp` = long, `pppp` = full).
    pub time_formats: TimeFormatLong,
    /// `formatLong.dateTime` widths — `"{{date}}", "{{time}}"` templates.
    pub date_time_formats: DateTimeFormatLong,
    /// The `match` half (`_lib/match.js`) the parse engine runs against.
    pub match_tables: MatchTables,
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

/// `formatLong.time` (`_lib/formatLong.js` `timeFormats`).
pub struct TimeFormatLong {
    /// `p` — short.
    pub short: &'static str,
    /// `pp` — medium.
    pub medium: &'static str,
    /// `ppp` — long.
    pub long: &'static str,
    /// `pppp` — full.
    pub full: &'static str,
}

/// `formatLong.dateTime` (`_lib/formatLong.js` `dateTimeFormats`).
pub struct DateTimeFormatLong {
    pub short: &'static str,
    pub medium: &'static str,
    pub long: &'static str,
    pub full: &'static str,
}

/// `(narrow, abbreviated, wide)` era names, index 0 = BC, 1 = AD
/// (`_lib/localize.js` `eraValues`).
pub struct Eras {
    pub narrow: [&'static str; 2],
    pub abbreviated: [&'static str; 2],
    pub wide: [&'static str; 2],
}

/// `(narrow, abbreviated, wide)` quarter names, 0-indexed (`_lib/localize.js`
/// `quarterValues`).
pub struct Quarters {
    pub narrow: [&'static str; 4],
    pub abbreviated: [&'static str; 4],
    pub wide: [&'static str; 4],
}

/// The `localize.dayPeriod` enum keys, in `_lib/localize.js` `dayPeriodValues` and
/// `parseDayPeriodPatterns.any` insertion order (the first key whose test matches wins).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DayPeriod {
    Am,
    Pm,
    Midnight,
    Noon,
    Morning,
    Afternoon,
    Evening,
    Night,
}

/// One width's `dayPeriod` strings (`_lib/localize.js`).
pub struct DayPeriodStrings {
    pub am: &'static str,
    pub pm: &'static str,
    pub midnight: &'static str,
    pub noon: &'static str,
    pub morning: &'static str,
    pub afternoon: &'static str,
    pub evening: &'static str,
    pub night: &'static str,
}

impl DayPeriodStrings {
    /// The string for a period at this width.
    pub fn get(&self, period: DayPeriod) -> &'static str {
        match period {
            DayPeriod::Am => self.am,
            DayPeriod::Pm => self.pm,
            DayPeriod::Midnight => self.midnight,
            DayPeriod::Noon => self.noon,
            DayPeriod::Morning => self.morning,
            DayPeriod::Afternoon => self.afternoon,
            DayPeriod::Evening => self.evening,
            DayPeriod::Night => self.night,
        }
    }
}

/// `(narrow, abbreviated, wide)` day-period strings.
pub struct DayPeriodWidths {
    pub narrow: DayPeriodStrings,
    pub abbreviated: DayPeriodStrings,
    pub wide: DayPeriodStrings,
}

impl DayPeriodWidths {
    /// `buildLocalizeFn`'s width resolution within one table — the widths are total, so
    /// a direct pick.
    pub fn get(&self, width: Width) -> &DayPeriodStrings {
        match width {
            Width::Narrow => &self.narrow,
            Width::Abbreviated | Width::Short => &self.abbreviated,
            Width::Wide => &self.wide,
        }
    }
}

/// The `localize.dayPeriod` tables: `buildLocalizeFn` uses `formattingValues` for the
/// `'formatting'` context when the locale carries one, `values` otherwise.
pub struct DayPeriodTables {
    pub standalone: DayPeriodWidths,
    pub formatting: DayPeriodWidths,
}

impl DayPeriodTables {
    /// The table for a context.
    pub fn get(&self, context: Context) -> &DayPeriodWidths {
        match context {
            Context::Formatting => &self.formatting,
            Context::Standalone => &self.standalone,
        }
    }
}

/// The `localize`/`match` width argument (`buildLocalizeFn`/`buildMatchFn` `width`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Width {
    Narrow,
    Short,
    Abbreviated,
    Wide,
}

/// The `localize` context argument (`buildLocalizeFn` `context`, default `'standalone'`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    Formatting,
    Standalone,
}

/// One piece of a transcribed `match` pattern (`_lib/match.js` regexes). The patterns
/// are anchored at the start of the input, case-insensitive, and their alternatives are
/// tried in declaration order (JS leftmost-first alternation).
#[derive(Clone, Copy, Debug)]
pub enum Pat {
    /// A required literal (case-insensitive) — `b`, `quarter`, `av.J.-C`.
    Lit(&'static str),
    /// A greedy optional literal — `\.?` (`.` is literal here), `T?`.
    OptLit(&'static str),
    /// A greedy optional choice among literals — `(th|st|nd|rd)?`, `(er|ème|e)?`.
    OptAlts(&'static [&'static str]),
    /// One whitespace character — `\s` (the `[-\s]` class flattens into alternatives).
    Space,
    /// A greedy optional whitespace character — `\s?`.
    OptSpace,
    /// One character from the set (case-insensitive) — `[jfmasond]`, `[1234]`.
    AnyOf(&'static str),
}

/// An ordered list of alternatives; each alternative is a pattern sequence.
pub type Alternatives = &'static [&'static [Pat]];

/// A width-keyed `matchPatterns` table (`buildMatchFn` `matchPatterns`). A `None` slot
/// means the regex is absent for that width, so `buildMatchFn`'s
/// `(width && matchPatterns[width]) || matchPatterns[defaultMatchWidth]` falls back to
/// the unit's `defaultMatchWidth` (the caller encodes it).
pub struct WidthMatch {
    pub narrow: Option<Alternatives>,
    pub short: Option<Alternatives>,
    pub abbreviated: Option<Alternatives>,
    pub wide: Option<Alternatives>,
}

impl WidthMatch {
    /// `buildMatchFn`'s resolution, with the unit's `defaultMatchWidth` fallback.
    pub fn get(&self, width: Option<Width>, default: Width) -> Alternatives {
        self.get_opt(width).or(self.get_opt(Some(default))).expect("default width is filled")
    }

    fn get_opt(&self, width: Option<Width>) -> Option<Alternatives> {
        match width {
            Some(Width::Narrow) => self.narrow,
            Some(Width::Short) => self.short,
            Some(Width::Abbreviated) => self.abbreviated,
            Some(Width::Wide) => self.wide,
            None => None,
        }
    }
}

/// The `match` half of a locale (`_lib/match.js`), transcribed per locale.
pub struct MatchTables {
    /// `match.era` — `defaultMatchWidth: 'wide'`.
    pub era: WidthMatch,
    /// `parseEraPatterns.any` — ordered anchored-prefix sets; the first set whose prefix
    /// matches the matched string decides 0 = BC / 1 = AD.
    pub era_parse_any: [&'static [&'static str]; 2],
    /// `match.quarter`.
    pub quarter: WidthMatch,
    /// `match.month`.
    pub month: WidthMatch,
    /// `parseMonthPatterns.narrow` — anchored single-letter prefixes.
    pub month_parse_narrow: &'static [&'static str],
    /// `parseMonthPatterns.any` — anchored prefixes tested against the matched string.
    pub month_parse_any: &'static [&'static str],
    /// `match.day`.
    pub day: WidthMatch,
    /// `parseDayPatterns.narrow`.
    pub day_parse_narrow: &'static [&'static str],
    /// `parseDayPatterns.any`.
    pub day_parse_any: &'static [&'static str],
    /// `match.dayPeriod` — `matchPatterns` only has `narrow` + `any`
    /// (`defaultMatchWidth: 'any'`); the `None` short/abbreviated/wide slots fall back.
    pub day_period_narrow: Alternatives,
    pub day_period_any: Alternatives,
    /// `parseDayPeriodPatterns.any` — the key set tested in insertion order; the flag is
    /// whether the test is anchored (`^`) or a substring search.
    pub day_period_parse_any: [(&'static str, bool); 8],
}

// en-US `match` pattern sequences (`locale/en-US/_lib/match.js`), as [`Pat`] data. The
// `(in the|at) (morning|…)` groups flatten into full alternatives; preference order is
// preserved because the time words are mutually exclusive prefixes.
const EN_A_DOT_ALT: &[Pat] = &[Pat::Lit("b"), Pat::OptLit("."), Pat::OptSpace, Pat::Lit("c"), Pat::OptLit(".")];
const EN_BCE_DOT_ALT: &[Pat] = &[
    Pat::Lit("b"),
    Pat::OptLit("."),
    Pat::OptSpace,
    Pat::Lit("c"),
    Pat::OptLit("."),
    Pat::OptSpace,
    Pat::Lit("e"),
    Pat::OptLit("."),
];
const EN_AD_DOT_ALT: &[Pat] = &[Pat::Lit("a"), Pat::OptLit("."), Pat::OptSpace, Pat::Lit("d"), Pat::OptLit(".")];
const EN_CE_DOT_ALT: &[Pat] = &[Pat::Lit("c"), Pat::OptLit("."), Pat::OptSpace, Pat::Lit("e"), Pat::OptLit(".")];
const EN_AM_DOT_ALT: &[Pat] = &[Pat::Lit("a"), Pat::OptLit("."), Pat::OptSpace, Pat::Lit("m"), Pat::OptLit(".")];
const EN_PM_DOT_ALT: &[Pat] = &[Pat::Lit("p"), Pat::OptLit("."), Pat::OptSpace, Pat::Lit("m"), Pat::OptLit(".")];

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
    eras: Eras {
        narrow: ["B", "A"],
        abbreviated: ["BC", "AD"],
        wide: ["Before Christ", "Anno Domini"],
    },
    quarters: Quarters {
        narrow: ["1", "2", "3", "4"],
        abbreviated: ["Q1", "Q2", "Q3", "Q4"],
        wide: ["1st quarter", "2nd quarter", "3rd quarter", "4th quarter"],
    },
    day_periods: DayPeriodTables {
        standalone: DayPeriodWidths {
            narrow: DayPeriodStrings {
                am: "a",
                pm: "p",
                midnight: "mi",
                noon: "n",
                morning: "morning",
                afternoon: "afternoon",
                evening: "evening",
                night: "night",
            },
            abbreviated: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "midnight",
                noon: "noon",
                morning: "morning",
                afternoon: "afternoon",
                evening: "evening",
                night: "night",
            },
            wide: DayPeriodStrings {
                am: "a.m.",
                pm: "p.m.",
                midnight: "midnight",
                noon: "noon",
                morning: "morning",
                afternoon: "afternoon",
                evening: "evening",
                night: "night",
            },
        },
        formatting: DayPeriodWidths {
            narrow: DayPeriodStrings {
                am: "a",
                pm: "p",
                midnight: "mi",
                noon: "n",
                morning: "in the morning",
                afternoon: "in the afternoon",
                evening: "in the evening",
                night: "at night",
            },
            abbreviated: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "midnight",
                noon: "noon",
                morning: "in the morning",
                afternoon: "in the afternoon",
                evening: "in the evening",
                night: "at night",
            },
            wide: DayPeriodStrings {
                am: "a.m.",
                pm: "p.m.",
                midnight: "midnight",
                noon: "noon",
                morning: "in the morning",
                afternoon: "in the afternoon",
                evening: "in the evening",
                night: "at night",
            },
        },
    },
    time_formats: TimeFormatLong {
        short: "h:mm a",
        medium: "h:mm:ss a",
        long: "h:mm:ss a z",
        full: "h:mm:ss a zzzz",
    },
    date_time_formats: DateTimeFormatLong {
        short: "{{date}}, {{time}}",
        medium: "{{date}}, {{time}}",
        long: "{{date}} 'at' {{time}}",
        full: "{{date}} 'at' {{time}}",
    },
    match_tables: MatchTables {
        era: WidthMatch {
            narrow: Some(&[&[Pat::Lit("b")], &[Pat::Lit("a")]]),
            short: None,
            abbreviated: Some(&[EN_A_DOT_ALT, EN_BCE_DOT_ALT, EN_AD_DOT_ALT, EN_CE_DOT_ALT]),
            wide: Some(&[
                &[Pat::Lit("before christ")],
                &[Pat::Lit("before common era")],
                &[Pat::Lit("anno domini")],
                &[Pat::Lit("common era")],
            ]),
        },
        era_parse_any: [&["b"], &["a", "c"]],
        quarter: WidthMatch {
            narrow: Some(&[&[Pat::AnyOf("1234")]]),
            short: None,
            abbreviated: Some(&[&[Pat::Lit("q"), Pat::AnyOf("1234")]]),
            wide: Some(&[&[
                Pat::AnyOf("1234"),
                Pat::OptAlts(&["th", "st", "nd", "rd"]),
                Pat::Lit(" quarter"),
            ]]),
        },
        month: WidthMatch {
            narrow: Some(&[&[Pat::AnyOf("jfmasond")]]),
            short: None,
            abbreviated: Some(&[
                &[Pat::Lit("jan")],
                &[Pat::Lit("feb")],
                &[Pat::Lit("mar")],
                &[Pat::Lit("apr")],
                &[Pat::Lit("may")],
                &[Pat::Lit("jun")],
                &[Pat::Lit("jul")],
                &[Pat::Lit("aug")],
                &[Pat::Lit("sep")],
                &[Pat::Lit("oct")],
                &[Pat::Lit("nov")],
                &[Pat::Lit("dec")],
            ]),
            wide: Some(&[
                &[Pat::Lit("january")],
                &[Pat::Lit("february")],
                &[Pat::Lit("march")],
                &[Pat::Lit("april")],
                &[Pat::Lit("may")],
                &[Pat::Lit("june")],
                &[Pat::Lit("july")],
                &[Pat::Lit("august")],
                &[Pat::Lit("september")],
                &[Pat::Lit("october")],
                &[Pat::Lit("november")],
                &[Pat::Lit("december")],
            ]),
        },
        month_parse_narrow: &["j", "f", "m", "a", "m", "j", "j", "a", "s", "o", "n", "d"],
        month_parse_any: &["ja", "f", "mar", "ap", "may", "jun", "jul", "au", "s", "o", "n", "d"],
        day: WidthMatch {
            narrow: Some(&[&[Pat::AnyOf("smtwf")]]),
            short: Some(&[
                &[Pat::Lit("su")],
                &[Pat::Lit("mo")],
                &[Pat::Lit("tu")],
                &[Pat::Lit("we")],
                &[Pat::Lit("th")],
                &[Pat::Lit("fr")],
                &[Pat::Lit("sa")],
            ]),
            abbreviated: Some(&[
                &[Pat::Lit("sun")],
                &[Pat::Lit("mon")],
                &[Pat::Lit("tue")],
                &[Pat::Lit("wed")],
                &[Pat::Lit("thu")],
                &[Pat::Lit("fri")],
                &[Pat::Lit("sat")],
            ]),
            wide: Some(&[
                &[Pat::Lit("sunday")],
                &[Pat::Lit("monday")],
                &[Pat::Lit("tuesday")],
                &[Pat::Lit("wednesday")],
                &[Pat::Lit("thursday")],
                &[Pat::Lit("friday")],
                &[Pat::Lit("saturday")],
            ]),
        },
        day_parse_narrow: &["s", "m", "t", "w", "t", "f", "s"],
        day_parse_any: &["su", "m", "tu", "w", "th", "f", "sa"],
        day_period_narrow: &[
            &[Pat::Lit("a")] as &[Pat],
            &[Pat::Lit("p")] as &[Pat],
            &[Pat::Lit("mi")] as &[Pat],
            &[Pat::Lit("n")] as &[Pat],
            &[Pat::Lit("in the morning")] as &[Pat],
            &[Pat::Lit("in the afternoon")] as &[Pat],
            &[Pat::Lit("in the evening")] as &[Pat],
            &[Pat::Lit("at night")] as &[Pat],
        ] as Alternatives,
        day_period_any: &[
            EN_AM_DOT_ALT,
            EN_PM_DOT_ALT,
            &[Pat::Lit("midnight")] as &[Pat],
            &[Pat::Lit("noon")] as &[Pat],
            &[Pat::Lit("in the morning")] as &[Pat],
            &[Pat::Lit("in the afternoon")] as &[Pat],
            &[Pat::Lit("in the evening")] as &[Pat],
            &[Pat::Lit("at night")] as &[Pat],
        ],
        day_period_parse_any: [
            ("a", true),
            ("p", true),
            ("mi", true),
            ("no", true),
            ("morning", false),
            ("afternoon", false),
            ("evening", false),
            ("night", false),
        ],
    },
};

// fr `match` pattern sequences (`locale/fr/_lib/match.js`). The regex meta-dots are
// literal characters here (`av\.J\.C` matches `av.J.C`); the day-period alternatives
// keep the regex's ordered preference, so `ap.m.` narrow-matches as `a` (am) exactly as
// upstream does.
const FR_A_DOT_ALT: &[Pat] = &[Pat::Lit("a"), Pat::OptLit("."), Pat::OptSpace, Pat::Lit("m"), Pat::OptLit(".")];
const FR_P_DOT_ALT: &[Pat] = &[Pat::Lit("p"), Pat::OptLit("."), Pat::OptSpace, Pat::Lit("m"), Pat::OptLit(".")];

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
    eras: Eras {
        narrow: ["av. J.-C", "ap. J.-C"],
        abbreviated: ["av. J.-C", "ap. J.-C"],
        wide: ["avant Jésus-Christ", "après Jésus-Christ"],
    },
    quarters: Quarters {
        narrow: ["T1", "T2", "T3", "T4"],
        abbreviated: ["1er trim.", "2ème trim.", "3ème trim.", "4ème trim."],
        wide: ["1er trimestre", "2ème trimestre", "3ème trimestre", "4ème trimestre"],
    },
    day_periods: DayPeriodTables {
        standalone: DayPeriodWidths {
            narrow: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "minuit",
                noon: "midi",
                morning: "mat.",
                afternoon: "ap.m.",
                evening: "soir",
                night: "mat.",
            },
            abbreviated: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "minuit",
                noon: "midi",
                morning: "matin",
                afternoon: "après-midi",
                evening: "soir",
                night: "matin",
            },
            wide: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "minuit",
                noon: "midi",
                morning: "du matin",
                afternoon: "de l’après-midi",
                evening: "du soir",
                night: "du matin",
            },
        },
        // fr's `dayPeriod` carries no `formattingValues` — both contexts read the same
        // table (`buildLocalizeFn`'s `args.formattingValues || args.values`).
        formatting: DayPeriodWidths {
            narrow: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "minuit",
                noon: "midi",
                morning: "mat.",
                afternoon: "ap.m.",
                evening: "soir",
                night: "mat.",
            },
            abbreviated: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "minuit",
                noon: "midi",
                morning: "matin",
                afternoon: "après-midi",
                evening: "soir",
                night: "matin",
            },
            wide: DayPeriodStrings {
                am: "AM",
                pm: "PM",
                midnight: "minuit",
                noon: "midi",
                morning: "du matin",
                afternoon: "de l’après-midi",
                evening: "du soir",
                night: "du matin",
            },
        },
    },
    time_formats: TimeFormatLong {
        short: "HH:mm",
        medium: "HH:mm:ss",
        long: "HH:mm:ss z",
        full: "HH:mm:ss zzzz",
    },
    date_time_formats: DateTimeFormatLong {
        short: "{{date}} 'à' {{time}}",
        medium: "{{date}} 'à' {{time}}",
        long: "{{date}} 'à' {{time}}",
        full: "{{date}} 'à' {{time}}",
    },
    match_tables: MatchTables {
        era: WidthMatch {
            narrow: Some(&[
                &[Pat::Lit("av.J.C")],
                &[Pat::Lit("ap.J.C")],
                &[Pat::Lit("ap.J.-C")],
            ]),
            short: None,
            abbreviated: Some(&[
                &[Pat::Lit("av.J.-C")],
                &[Pat::Lit("av.J-C")],
                &[Pat::Lit("apr.J.-C")],
                &[Pat::Lit("apr.J-C")],
                &[Pat::Lit("ap.J-C")],
            ]),
            wide: Some(&[
                &[Pat::Lit("avant Jésus-Christ")],
                &[Pat::Lit("après Jésus-Christ")],
            ]),
        },
        era_parse_any: [&["av"], &["ap"]],
        quarter: WidthMatch {
            narrow: Some(&[&[Pat::OptLit("T"), Pat::AnyOf("1234")]]),
            short: None,
            abbreviated: Some(&[&[
                Pat::AnyOf("1234"),
                Pat::OptAlts(&["er", "ème", "e"]),
                Pat::Lit(" trim"),
                Pat::OptLit("."),
            ]]),
            wide: Some(&[&[
                Pat::AnyOf("1234"),
                Pat::OptAlts(&["er", "ème", "e"]),
                Pat::Lit(" trimestre"),
            ]]),
        },
        month: WidthMatch {
            narrow: Some(&[&[Pat::AnyOf("jfmasond")]]),
            short: None,
            abbreviated: Some(&[
                &[Pat::Lit("janv"), Pat::OptLit(".")],
                &[Pat::Lit("févr"), Pat::OptLit(".")],
                &[Pat::Lit("mars"), Pat::OptLit(".")],
                &[Pat::Lit("avr"), Pat::OptLit(".")],
                &[Pat::Lit("mai"), Pat::OptLit(".")],
                &[Pat::Lit("juin"), Pat::OptLit(".")],
                &[Pat::Lit("juill"), Pat::OptLit(".")],
                &[Pat::Lit("juil"), Pat::OptLit(".")],
                &[Pat::Lit("août"), Pat::OptLit(".")],
                &[Pat::Lit("sept"), Pat::OptLit(".")],
                &[Pat::Lit("oct"), Pat::OptLit(".")],
                &[Pat::Lit("nov"), Pat::OptLit(".")],
                &[Pat::Lit("déc"), Pat::OptLit(".")],
            ]),
            wide: Some(&[
                &[Pat::Lit("janvier")],
                &[Pat::Lit("février")],
                &[Pat::Lit("mars")],
                &[Pat::Lit("avril")],
                &[Pat::Lit("mai")],
                &[Pat::Lit("juin")],
                &[Pat::Lit("juillet")],
                &[Pat::Lit("août")],
                &[Pat::Lit("septembre")],
                &[Pat::Lit("octobre")],
                &[Pat::Lit("novembre")],
                &[Pat::Lit("décembre")],
            ]),
        },
        month_parse_narrow: &["j", "f", "m", "a", "m", "j", "j", "a", "s", "o", "n", "d"],
        month_parse_any: &["ja", "f", "mar", "av", "ma", "juin", "juil", "ao", "s", "o", "n", "d"],
        day: WidthMatch {
            narrow: Some(&[&[Pat::AnyOf("lmjvsd")]]),
            short: Some(&[
                &[Pat::Lit("di")],
                &[Pat::Lit("lu")],
                &[Pat::Lit("ma")],
                &[Pat::Lit("me")],
                &[Pat::Lit("je")],
                &[Pat::Lit("ve")],
                &[Pat::Lit("sa")],
            ]),
            abbreviated: Some(&[
                &[Pat::Lit("dim"), Pat::OptLit(".")],
                &[Pat::Lit("lun"), Pat::OptLit(".")],
                &[Pat::Lit("mar"), Pat::OptLit(".")],
                &[Pat::Lit("mer"), Pat::OptLit(".")],
                &[Pat::Lit("jeu"), Pat::OptLit(".")],
                &[Pat::Lit("ven"), Pat::OptLit(".")],
                &[Pat::Lit("sam"), Pat::OptLit(".")],
            ]),
            wide: Some(&[
                &[Pat::Lit("dimanche")],
                &[Pat::Lit("lundi")],
                &[Pat::Lit("mardi")],
                &[Pat::Lit("mercredi")],
                &[Pat::Lit("jeudi")],
                &[Pat::Lit("vendredi")],
                &[Pat::Lit("samedi")],
            ]),
        },
        day_parse_narrow: &["d", "l", "m", "m", "j", "v", "s"],
        day_parse_any: &["di", "lu", "ma", "me", "je", "ve", "sa"],
        day_period_narrow: &[
            &[Pat::Lit("a")],
            &[Pat::Lit("p")],
            &[Pat::Lit("minuit")],
            &[Pat::Lit("midi")],
            &[Pat::Lit("mat"), Pat::OptLit(".")],
            &[Pat::Lit("ap"), Pat::OptLit("."), Pat::Lit("m"), Pat::OptLit(".")],
            &[Pat::Lit("soir")],
            &[Pat::Lit("nuit")],
        ],
        // `de l'après[-\s]midi` flattens to the dash and space variants.
        day_period_any: &[
            FR_A_DOT_ALT,
            FR_P_DOT_ALT,
            &[Pat::Lit("du matin")],
            &[Pat::Lit("de l'après-midi")],
            &[Pat::Lit("de l'après midi")],
            &[Pat::Lit("du soir")],
            &[Pat::Lit("de la nuit")],
        ],
        day_period_parse_any: [
            ("a", true),
            ("p", true),
            ("min", true),
            ("mid", true),
            ("mat", false),
            ("ap", false),
            ("soir", false),
            ("nuit", false),
        ],
    },
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

// ─── pattern matcher ─────────────────────────────────────────────────────────────────
//
// The `match` patterns are case-insensitive, anchored at the input start, and their
// alternatives are tried in declaration order (JS leftmost-first alternation). Greedy
// optional pieces try the present branch first and fall back to the absent branch —
// the same backtracking order a Perl-family regex engine uses.

fn chars_ci_eq(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

/// The JS `\s` class (the patterns only ever meet plain spaces in practice; the class is
/// spelled out so `[-\s]`-style alternatives behave identically).
fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\n' | '\r' | '\u{b}' | '\u{c}' | '\u{85}' | '\u{a0}')
}

fn match_seq(pats: &[Pat], input: &[char]) -> Option<usize> {
    let (first, rest_pats) = pats.split_first()?;
    match first {
        Pat::Lit(s) => {
            let n = s.chars().count();
            if input.len() >= n && s.chars().zip(input).all(|(a, b)| chars_ci_eq(a, *b)) {
                match_seq(rest_pats, &input[n..]).map(|rest| rest + n)
            } else {
                None
            }
        }
        Pat::OptLit(s) => {
            let n = s.chars().count();
            if input.len() >= n && s.chars().zip(input).all(|(a, b)| chars_ci_eq(a, *b)) {
                if let Some(total) = match_seq(rest_pats, &input[n..]) {
                    return Some(total + n);
                }
            }
            match_seq(rest_pats, input)
        }
        Pat::OptAlts(alts) => {
            for alt in *alts {
                let n = alt.chars().count();
                if input.len() >= n && alt.chars().zip(input).all(|(a, b)| chars_ci_eq(a, *b)) {
                    if let Some(total) = match_seq(rest_pats, &input[n..]) {
                        return Some(total + n);
                    }
                }
            }
            match_seq(rest_pats, input)
        }
        Pat::Space => {
            if input.first().copied().is_some_and(is_space) {
                match_seq(rest_pats, &input[1..]).map(|rest| rest + 1)
            } else {
                None
            }
        }
        Pat::OptSpace => {
            if input.first().copied().is_some_and(is_space) {
                if let Some(total) = match_seq(rest_pats, &input[1..]) {
                    return Some(total + 1);
                }
            }
            match_seq(rest_pats, input)
        }
        Pat::AnyOf(set) => {
            if let Some(c) = input.first() {
                if set.chars().any(|s| chars_ci_eq(s, *c)) {
                    return match_seq(rest_pats, &input[1..]).map(|rest| rest + 1);
                }
            }
            None
        }
    }
}

/// The first alternative that matches, with its consumed length.
fn match_alternatives(alts: Alternatives, input: &[char]) -> Option<usize> {
    alts.iter().find_map(|alt| match_seq(alt, input))
}

impl DateFnsLocale {
    /// `match.ordinalNumber` (`buildMatchPatternFn` over `/^(\d+)(th|st|nd|rd)?/i` and
    /// the fr suffixes): the digits are the value; the optional suffix is consumed.
    pub(crate) fn match_ordinal_number(&self, input: &str) -> Option<(i64, String)> {
        let chars: Vec<char> = input.chars().collect();
        let digit_end = chars.iter().position(|c| !c.is_ascii_digit()).unwrap_or(chars.len());
        if digit_end == 0 {
            return None;
        }
        let mut consumed = digit_end;
        let digits: String = chars[..digit_end].iter().collect();
        // Greedy digit run first, then the optional first-matching suffix.
        for suffix in self.ordinal_suffixes {
            let s: Vec<char> = suffix.chars().collect();
            if chars.len() >= digit_end + s.len()
                && s.iter().zip(&chars[digit_end..]).all(|(a, b)| chars_ci_eq(*a, *b))
            {
                consumed = digit_end + s.len();
                break;
            }
        }
        Some((digits.parse().ok()?, chars[consumed..].iter().collect()))
    }

    /// `match.era` (`buildMatchFn`, `defaultMatchWidth: 'wide'`): returns 0 = BC,
    /// 1 = AD per `parseEraPatterns.any`.
    pub(crate) fn match_era(&self, input: &str, width: Option<Width>) -> Option<(i64, String)> {
        let matched = self.match_with_width(input, width, &self.match_tables.era, Width::Wide)?;
        let chars: Vec<char> = matched.chars().collect();
        let era = self
            .match_tables
            .era_parse_any
            .iter()
            .position(|prefixes| prefixes.iter().any(|p| starts_with_prefix(&chars, p)))?;
        Some((era as i64, input[matched.len()..].to_string()))
    }

    /// `match.quarter` (`parseQuarterPatterns.any`: the first digit contained in the
    /// matched string, `valueCallback: index + 1`).
    pub(crate) fn match_quarter(
        &self,
        input: &str,
        width: Option<Width>,
    ) -> Option<(i64, String)> {
        let matched = self.match_with_width(input, width, &self.match_tables.quarter, Width::Wide)?;
        let chars: Vec<char> = matched.chars().collect();
        let index = chars
            .iter()
            .position(|c| matches!(c, '1' | '2' | '3' | '4'))?;
        Some((index as i64 + 1, input[matched.len()..].to_string()))
    }

    /// `match.month` (`parsePatterns`: narrow width → the narrow prefixes, every other
    /// width → the `any` prefixes).
    pub(crate) fn match_month(&self, input: &str, width: Option<Width>) -> Option<(i64, String)> {
        let matched = self.match_with_width(input, width, &self.match_tables.month, Width::Wide)?;
        let chars: Vec<char> = matched.chars().collect();
        let table = if width == Some(Width::Narrow) {
            self.match_tables.month_parse_narrow
        } else {
            self.match_tables.month_parse_any
        };
        let index = table.iter().position(|p| starts_with_prefix(&chars, p))?;
        Some((index as i64, input[matched.len()..].to_string()))
    }

    /// `match.day` — same width→parse-table rule as [`Self::match_month`]; 0 = Sunday.
    pub(crate) fn match_day(&self, input: &str, width: Option<Width>) -> Option<(i64, String)> {
        let matched = self.match_with_width(input, width, &self.match_tables.day, Width::Wide)?;
        let chars: Vec<char> = matched.chars().collect();
        let table = if width == Some(Width::Narrow) {
            self.match_tables.day_parse_narrow
        } else {
            self.match_tables.day_parse_any
        };
        let index = table.iter().position(|p| starts_with_prefix(&chars, p))?;
        Some((index as i64, input[matched.len()..].to_string()))
    }

    /// `match.dayPeriod` (`matchPatterns` has narrow + any, `defaultMatchWidth: 'any'`;
    /// `parseDayPeriodPatterns.any` has the single `any` key set).
    pub(crate) fn match_day_period(
        &self,
        input: &str,
        width: Option<Width>,
    ) -> Option<(DayPeriod, String)> {
        let chars: Vec<char> = input.chars().collect();
        let alts = if width == Some(Width::Narrow) {
            self.match_tables.day_period_narrow
        } else {
            self.match_tables.day_period_any
        };
        let matched_len = match_alternatives(alts, &chars)?;
        let matched: String = chars[..matched_len].iter().collect();
        let period = self
            .match_tables
            .day_period_parse_any
            .iter()
            .position(|(needle, anchored)| {
                if *anchored {
                    starts_with_prefix(&chars[..matched_len], needle)
                } else {
                    contains_ci(&matched, needle)
                }
            })?;
        Some((day_period_from_index(period), input[matched_len..].to_string()))
    }

    /// `buildMatchFn`'s `matchPatterns[width] || matchPatterns[defaultMatchWidth]`, then
    /// the matched string of the first matching alternative.
    fn match_with_width(
        &self,
        input: &str,
        width: Option<Width>,
        table: &WidthMatch,
        default: Width,
    ) -> Option<String> {
        let chars: Vec<char> = input.chars().collect();
        let alts = table.get(width, default);
        let matched_len = match_alternatives(alts, &chars)?;
        Some(chars[..matched_len].iter().collect())
    }
}

fn starts_with_prefix(chars: &[char], prefix: &str) -> bool {
    let p: Vec<char> = prefix.chars().collect();
    chars.len() >= p.len() && p.iter().zip(chars).all(|(a, b)| chars_ci_eq(*a, *b))
}

fn contains_ci(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

fn day_period_from_index(index: usize) -> DayPeriod {
    match index {
        0 => DayPeriod::Am,
        1 => DayPeriod::Pm,
        2 => DayPeriod::Midnight,
        3 => DayPeriod::Noon,
        4 => DayPeriod::Morning,
        5 => DayPeriod::Afternoon,
        6 => DayPeriod::Evening,
        _ => DayPeriod::Night,
    }
}
