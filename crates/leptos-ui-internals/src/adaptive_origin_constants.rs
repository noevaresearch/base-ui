//! Port of `packages/react/src/utils/adaptiveOriginConstants.ts` — the default CSS
//! property names for the adaptive-origin coordinate split, plus the deserialization
//! shape for a custom `adaptiveOrigin` middleware's data.

use serde_json::Value;

/// `DEFAULT_SIDES` (`adaptiveOriginConstants.ts:1-5`) — the CSS properties the popup's
/// x/y coordinates apply to when no `adaptiveOrigin` middleware is configured. Upstream
/// is a frozen object; the port is a function returning a fresh value (the fields are
/// owned `String`s for deserialization symmetry — see [`AdaptiveOriginData`]).
pub fn default_sides() -> AdaptiveOriginData {
    AdaptiveOriginData {
        side_x: "left".to_owned(),
        side_y: "top".to_owned(),
    }
}

/// The data a custom `adaptiveOrigin` middleware attaches under its own name
/// (`useAnchorPositioning.ts:497` reads `middlewareData.adaptiveOrigin`; the upstream
/// type is the self-contained stand-in in `adaptiveOriginConstants.ts:8-15`). The
/// engine's JSON middleware-data map (`floating_ui_core::MiddlewareData`) is read
/// field-by-field rather than through a `Deserialize` derive — the crate deliberately
/// has no direct `serde` dependency.
#[derive(Clone, Debug, PartialEq)]
pub struct AdaptiveOriginData {
    /// `sideX` — the CSS property the popup's x coordinate applies to (`'left'`/`'right'`).
    pub side_x: String,
    /// `sideY` — the CSS property the popup's y coordinate applies to (`'top'`/`'bottom'`).
    pub side_y: String,
}

impl AdaptiveOriginData {
    /// Reads the data from the engine's JSON map (`middlewareData.get("adaptiveOrigin")`),
    /// falling back to [`default_sides`] when absent or malformed — upstream's
    /// `middlewareData.adaptiveOrigin || DEFAULT_SIDES` (`useAnchorPositioning.ts:497`).
    pub fn from_middleware_data(data: Option<&Value>) -> AdaptiveOriginData {
        let Some(data) = data else {
            return default_sides();
        };
        let side_x = data
            .get("sideX")
            .and_then(Value::as_str)
            .unwrap_or("left")
            .to_owned();
        let side_y = data
            .get("sideY")
            .and_then(Value::as_str)
            .unwrap_or("top")
            .to_owned();
        AdaptiveOriginData { side_x, side_y }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod host_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_sides_are_left_and_top() {
        assert_eq!(
            default_sides(),
            AdaptiveOriginData {
                side_x: "left".to_owned(),
                side_y: "top".to_owned(),
            }
        );
    }

    #[test]
    fn reads_the_configured_sides_from_the_middleware_data() {
        let data = json!({ "sideX": "right", "sideY": "bottom" });
        assert_eq!(
            AdaptiveOriginData::from_middleware_data(Some(&data)),
            AdaptiveOriginData {
                side_x: "right".to_owned(),
                side_y: "bottom".to_owned(),
            }
        );
    }

    #[test]
    fn falls_back_to_the_defaults_when_absent_or_malformed() {
        assert_eq!(
            AdaptiveOriginData::from_middleware_data(None),
            default_sides()
        );
        assert_eq!(
            AdaptiveOriginData::from_middleware_data(Some(&json!({ "sideX": 3 }))),
            default_sides()
        );
    }
}
