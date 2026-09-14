//! Pure-utility half of the OTP Field port — the line-for-line translation of
//! `packages/react/src/otp-field/utils/otp.ts` (behavior.md "Utils layer behavior",
//! implementation.md "Utils": "pure functions — no hooks").
//!
//! The normalization pipeline (implementation.md, "Utils" section):
//! strip whitespace → apply `validationType` regexp → apply user `normalizeValue` →
//! re-apply validation → clamp by Unicode code point
//! (`normalizeOTPValueWithDetails`, otp.ts:51-77), threading a
//! `didRejectCharacters` flag through every shrinking step to drive `onValueInvalid`.

/// `OTPValidationType` (`otp.ts:5`) — the built-in charset filters.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum OtpValidationType {
    /// `numeric` — the upstream prop default (`OTPFieldRoot.tsx:60`).
    #[default]
    Numeric,
    Alpha,
    Alphanumeric,
    /// `'none'` — disables built-in filtering; custom `normalizeValue` only.
    None,
}

/// `OTPValidationConfig` (`otp.ts:1-9`) — per-type slot/root patterns, the
/// rejection regexp, and the derived input mode.
pub struct OtpValidationConfig {
    /// `slotPattern` — the visible slots' single-character `pattern` attribute
    /// (e.g. `[a-zA-Z0-9]{1}`).
    pub slot_pattern: &'static str,
    /// `getRootPattern(length)` — the hidden input's length-sized pattern
    /// (e.g. `\d{6}`).
    pub root_pattern: fn(usize) -> String,
    /// `inputMode` — `numeric` or `text`.
    pub input_mode: &'static str,
    /// Which validation arm this config drives (`regexp`, otp.ts:13/18/23) — the
    /// JS `RegExp` ports to an explicit charset predicate per arm.
    pub charset: OtpCharset,
}

/// The per-arm charset predicate standing in for the JS `RegExp` — a character
/// is KEPT iff it passes (upstream strips the *negated* class: `/[^\d]/g` etc).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OtpCharset {
    /// `/[^\d]/g` — keep digits.
    Digits,
    /// `/[^a-zA-Z]/g` — keep ASCII letters.
    Letters,
    /// `/[^a-zA-Z0-9]/g` — keep ASCII alphanumerics.
    Alphanumeric,
}

impl OtpCharset {
    /// The regexp's keep-arm (`value.replace(validation.regexp, '')`, otp.ts:43):
    /// every character failing the charset is removed.
    fn keeps(self, c: char) -> bool {
        match self {
            OtpCharset::Digits => c.is_ascii_digit(),
            OtpCharset::Letters => c.is_ascii_alphabetic(),
            OtpCharset::Alphanumeric => c.is_ascii_alphanumeric(),
        }
    }
}

/// `OTP_VALIDATION_CONFIG` (`otp.ts:11-27`).
pub const OTP_VALIDATION_CONFIG: &[(OtpValidationType, OtpValidationConfig)] = &[
    (
        OtpValidationType::Numeric,
        OtpValidationConfig {
            slot_pattern: r"\d{1}",
            root_pattern: |length| format!(r"\d{{{length}}}"),
            input_mode: "numeric",
            charset: OtpCharset::Digits,
        },
    ),
    (
        OtpValidationType::Alpha,
        OtpValidationConfig {
            slot_pattern: "[a-zA-Z]{1}",
            root_pattern: |length| format!("[a-zA-Z]{{{length}}}"),
            input_mode: "text",
            charset: OtpCharset::Letters,
        },
    ),
    (
        OtpValidationType::Alphanumeric,
        OtpValidationConfig {
            slot_pattern: "[a-zA-Z0-9]{1}",
            root_pattern: |length| format!("[a-zA-Z0-9]{{{length}}}"),
            input_mode: "text",
            charset: OtpCharset::Alphanumeric,
        },
    ),
];

/// `getOTPValidationConfig` (`otp.ts:29-38`): `None` for `'none'`, the config
/// otherwise. Upstream's `Record<Exclude<OTPValidationType, 'none'>, …>` lookup
/// is total for the three real arms, so the port returns a reference into the
/// const table.
pub fn get_otp_validation_config(
    validation_type: OtpValidationType,
) -> Option<&'static OtpValidationConfig> {
    match validation_type {
        OtpValidationType::None => None,
        other => OTP_VALIDATION_CONFIG
            .iter()
            .find(|(key, _)| *key == other)
            .map(|(_, config)| config),
    }
}

/// `stripOTPWhitespace` (`otp.ts:40-42`): removes every `\s` character. The
/// JS `/\s/g` matches Unicode whitespace; Rust's `char::is_whitespace` is the
/// same shape for the whitespace ranges the tests exercise (spaces, tabs).
pub fn strip_otp_whitespace(value: Option<&str>) -> String {
    value
        .unwrap_or("")
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}

/// `applyOTPValidation` (`otp.ts:44-46`).
fn apply_otp_validation(value: &str, validation: Option<&OtpValidationConfig>) -> String {
    match validation {
        Some(config) => value.chars().filter(|c| config.charset.keeps(*c)).collect(),
        None => value.to_string(),
    }
}

/// `normalizeOTPValueWithDetails` (`otp.ts:51-77`): the full pipeline, returning
/// `(normalizedValue, didRejectCharacters)` — the thread that drives
/// `onValueInvalid` (behavior.md "Events").
pub fn normalize_otp_value_with_details(
    value: Option<&str>,
    length: usize,
    validation_type: OtpValidationType,
    normalize_value: Option<&dyn Fn(&str) -> String>,
) -> (String, bool) {
    let stripped_value = strip_otp_whitespace(value);
    let validation = get_otp_validation_config(validation_type);
    let mut normalized_value = apply_otp_validation(&stripped_value, validation);
    // `didRejectCharacters = strippedValue.length > normalizedValue.length`
    // (`otp.ts:56`) — string length, so Rust `chars().count()` (the code-point
    // count the JS string length is).
    let mut did_reject_characters =
        stripped_value.chars().count() > normalized_value.chars().count();

    if let Some(normalize_value) = normalize_value {
        let custom_normalized_value = normalize_value(&normalized_value);
        did_reject_characters |=
            normalized_value.chars().count() > custom_normalized_value.chars().count();
        normalized_value = apply_otp_validation(&custom_normalized_value, validation);
        did_reject_characters |=
            custom_normalized_value.chars().count() > normalized_value.chars().count();
    }

    // Clamp by Unicode code point (`otp.ts:63-70`) so multi-byte characters do
    // not split across OTP slots. Negative length cannot occur in the Rust
    // signature (`usize`) — the `length < 0 ? 0` arm is subsumed by the type.
    let max_length = length;
    let characters: Vec<char> = normalized_value.chars().collect();
    let clamped: String = characters.iter().take(max_length).collect();

    (
        clamped,
        did_reject_characters || characters.len() > max_length,
    )
}

/// `normalizeOTPValue` (`otp.ts:79-85`) — the details-free wrapper, the write
/// paths' normalizer.
pub fn normalize_otp_value(
    value: Option<&str>,
    length: usize,
    validation_type: OtpValidationType,
    normalize_value: Option<&dyn Fn(&str) -> String>,
) -> String {
    normalize_otp_value_with_details(value, length, validation_type, normalize_value).0
}

/// `replaceOTPValue` (`otp.ts:92-110`): splices normalized chars at a slot
/// index and re-normalizes the concatenation — what preserves the suffix when a
/// middle replacement shrinks (behavior.md "Utils layer behavior").
pub fn replace_otp_value(
    current_value: &str,
    index: usize,
    next_value: &str,
    length: usize,
    validation_type: OtpValidationType,
    normalize_value: Option<&dyn Fn(&str) -> String>,
) -> String {
    let normalized_value =
        normalize_otp_value(Some(next_value), length, validation_type, normalize_value);

    // `currentValue.slice(0, index)` / `.slice(index + normalizedValue.length)`
    // (`otp.ts:96-97`) — JS string slice indexes are code-point offsets, so the
    // port walks chars, saturating past the end like `String.slice` does.
    let chars: Vec<char> = current_value.chars().collect();
    let end = (index + normalized_value.chars().count()).min(chars.len());
    let prefix: String = chars.iter().take(index).collect();
    let suffix: String = chars.iter().skip(end).collect();

    normalize_otp_value(
        Some(&format!("{prefix}{normalized_value}{suffix}")),
        length,
        validation_type,
        normalize_value,
    )
}

/// `removeOTPCharacter` (`otp.ts:112-120`): deletes one char at an index; a
/// no-op for out-of-bounds indexes (behavior.md "Utils layer behavior").
pub fn remove_otp_character(current_value: &str, index: i64) -> String {
    // `index < 0 || index >= currentValue.length` (`otp.ts:113-115`).
    if index < 0 || index as usize >= current_value.chars().count() {
        return current_value.to_string();
    }

    let index = index as usize;
    let chars: Vec<char> = current_value.chars().collect();
    let head: String = chars.iter().take(index).collect();
    let tail: String = chars.iter().skip(index + 1).collect();
    format!("{head}{tail}")
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;

    // Mirrors `packages/react/src/otp-field/utils/otp.test.ts` — the four
    // directly-exercised utils (behavior.md "Public API surface": the pure
    // utils are exercised directly).

    // `stripOTPWhitespace` (`otp.test.ts:14-20`).
    #[test]
    fn strip_whitespace_removes_all_whitespace() {
        assert_eq!(strip_otp_whitespace(Some("1 2\t3\n4")), "1234");
        assert_eq!(strip_otp_whitespace(Some("1234")), "1234");
        assert_eq!(strip_otp_whitespace(None), "");
        assert_eq!(strip_otp_whitespace(Some("")), "");
    }

    // `normalizeOTPValue` per validationType (`otp.test.ts:22-47`, and the
    // component-level mirror `OTPFieldRoot.test.tsx:132-155`):
    // alpha keeps only letters, alphanumeric keeps [a-zA-Z0-9].
    #[test]
    fn validation_filters_per_type_then_clamps_to_length() {
        // '1a2b3Cd4' → 'abCd' under alpha (behavior.md "State model").
        assert_eq!(
            normalize_otp_value(Some("1a2b3Cd4"), 6, OtpValidationType::Alpha, None),
            "abCd"
        );
        // 'A1-B2c3' → 'A1B2c3' under alphanumeric.
        assert_eq!(
            normalize_otp_value(Some("A1-B2c3"), 6, OtpValidationType::Alphanumeric, None),
            "A1B2c3"
        );
        // numeric strips everything non-digit.
        assert_eq!(
            normalize_otp_value(Some("12a34b56"), 6, OtpValidationType::Numeric, None),
            "123456"
        );
        // clamp: '1234567890' at length 6 → '123456' (`otp.test.ts` clamp arm).
        assert_eq!(
            normalize_otp_value(Some("1234567890"), 6, OtpValidationType::Numeric, None),
            "123456"
        );
        // 'none' disables the built-in filter but still clamps.
        assert_eq!(
            normalize_otp_value(Some("a!b"), 2, OtpValidationType::None, None),
            "a!"
        );
    }

    // Custom normalization composes AFTER built-in validation
    // (`otp.test.ts` custom arm; `OTPFieldRoot.test.tsx:236-256`: 'a!' →
    // strip '!' → normalizeValue('a') → 'A').
    #[test]
    fn custom_normalization_composes_after_builtin_filtering() {
        let upper = |v: &str| v.to_uppercase();
        let (value, rejected) =
            normalize_otp_value_with_details(Some("a!"), 6, OtpValidationType::Alpha, Some(&upper));
        assert_eq!(value, "A");
        // The '!' was rejected by the built-in filter before the custom step.
        assert!(rejected);
    }

    // didRejectCharacters covers the custom step shrinking
    // (`normalizeValue` removing chars fires onValueInvalid — behavior.md
    // "Events").
    #[test]
    fn rejection_flag_covers_custom_shrinking_and_clamping() {
        let drop_last = |v: &str| {
            v.chars()
                .take(v.chars().count().saturating_sub(1))
                .collect()
        };
        let (_, rejected) = normalize_otp_value_with_details(
            Some("abcd"),
            6,
            OtpValidationType::Alpha,
            Some(&drop_last),
        );
        assert!(rejected, "the custom step removed a character");

        // Overlong input clamped → rejected too (`otp.ts:70-71`).
        let (_, rejected) =
            normalize_otp_value_with_details(Some("1234567"), 6, OtpValidationType::Numeric, None);
        assert!(rejected);
        // Exact fit → not rejected.
        let (_, rejected) =
            normalize_otp_value_with_details(Some("123456"), 6, OtpValidationType::Numeric, None);
        assert!(!rejected);
    }

    // `replaceOTPValue` (`otp.test.ts:49-70`): splice at index, suffix
    // preserved when a middle replacement shrinks.
    #[test]
    fn replace_splices_and_preserves_the_suffix() {
        // Insert at 0 over '1234'.
        assert_eq!(
            replace_otp_value("1234", 0, "ab", 6, OtpValidationType::Alphanumeric, None),
            "ab34"
        );
        // Middle replacement: value '1234', replace slot 1 with 'x' → '1x34'.
        assert_eq!(
            replace_otp_value("1234", 1, "x", 6, OtpValidationType::Alphanumeric, None),
            "1x34"
        );
        // Replacement shrinks (empty next): '1234' at index 1 → '134' — the
        // suffix '4'... wait, upstream splices length(next) chars: empty next
        // replaces NOTHING (slice(index + 0)) — '1234' stays '1234'? No: the
        // suffix starts at index + normalizedValue.length = 1, so it is '234'
        // — the char at `index` is CONSUMED by the splice only when next is
        // empty? Re-reading otp.ts:96-97: suffix = currentValue.slice(index +
        // normalizedValue.length) — with next='' that is slice(1) = '234', so
        // the char at index survives via the suffix and the result is '1234'.
        assert_eq!(
            replace_otp_value("1234", 1, "", 6, OtpValidationType::Alphanumeric, None),
            "1234"
        );
        // Multi-char paste over the middle: '123456', paste 'ab' at 2 → '12ab56'
        // (behavior.md "Focus management": paste into a middle slot).
        assert_eq!(
            replace_otp_value("123456", 2, "ab", 6, OtpValidationType::Alphanumeric, None),
            "12ab56"
        );
        // '12345', paste 'abcde' at 2 under numeric → the paste normalizes to
        // '' (letters stripped), so the splice is prefix '12' + '' + suffix
        // '345' = '12345' (nothing changes).
        assert_eq!(
            replace_otp_value("12345", 2, "abcde", 6, OtpValidationType::Numeric, None),
            "12345"
        );
        // Overlong paste re-normalizes (clamps) the concatenation: '12345' +
        // paste '67' at 2 → prefix '12' + '67' + suffix slice(2+2=4)='5' =
        // '12675' (the splice consumes normalizedValue.length chars of the
        // suffix region, so only '5' survives), already within length 6.
        assert_eq!(
            replace_otp_value("12345", 2, "67", 6, OtpValidationType::Numeric, None),
            "12675"
        );
    }

    // `removeOTPCharacter` (`otp.test.ts:72-93`): one char at an index, no-op
    // out of bounds.
    #[test]
    fn remove_deletes_one_char_and_is_a_noop_out_of_bounds() {
        assert_eq!(remove_otp_character("1234", 0), "234");
        assert_eq!(remove_otp_character("1234", 3), "123");
        // Out-of-bounds no-ops.
        assert_eq!(remove_otp_character("1234", 4), "1234");
        assert_eq!(remove_otp_character("1234", -1), "1234");
        assert_eq!(remove_otp_character("", 0), "");
    }

    // Config surface (behavior.md "Accessibility": built-in validationType
    // sets inputMode and pattern attributes).
    #[test]
    fn config_surfaces_match_upstream() {
        let numeric = get_otp_validation_config(OtpValidationType::Numeric).unwrap();
        assert_eq!(numeric.slot_pattern, r"\d{1}");
        assert_eq!((numeric.root_pattern)(6), r"\d{6}");
        assert_eq!(numeric.input_mode, "numeric");

        let alpha = get_otp_validation_config(OtpValidationType::Alpha).unwrap();
        assert_eq!(alpha.slot_pattern, "[a-zA-Z]{1}");
        assert_eq!((alpha.root_pattern)(4), "[a-zA-Z]{4}");
        assert_eq!(alpha.input_mode, "text");

        let alphanumeric = get_otp_validation_config(OtpValidationType::Alphanumeric).unwrap();
        assert_eq!(alphanumeric.slot_pattern, "[a-zA-Z0-9]{1}");
        assert_eq!((alphanumeric.root_pattern)(6), "[a-zA-Z0-9]{6}");

        assert!(get_otp_validation_config(OtpValidationType::None).is_none());
    }
}
