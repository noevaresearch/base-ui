//! Port of the Base UI Radio unit (`packages/react/src/radio`) — the `library: radio`
//! TODO item (`specs/library/radio/behavior.md`, `specs/library/radio/implementation.md`).
//!
//! Upstream's shape (implementation.md, "State machine / hooks used"): **`Radio.Root` is a
//! stateless selection proxy**. It owns no checked state — `checked` is a pure per-render
//! derivation (`RadioRoot.tsx:83`), computed against the surrounding `RadioGroup`'s context
//! in group mode and against the empty string standalone. Every interaction funnels through
//! the single native `change` event of the hidden `<input type="radio">` (`:184-203`): the
//! visible control re-dispatches its own click onto that input (`:139-152`) instead of
//! mutating state. `Radio.Indicator` is a pure function of the Root's `checked` plus the
//! transition-status machine (`RadioIndicator.tsx:25-27`).
//!
//! Module map:
//! - [`state`] — the pure folds, the state records, the state→attribute walk and the
//!   `visuallyHidden`/`serializeValue` recipes (host-testable, view-independent).
//! - [`context`] — the Root context the parts read, with upstream's throw-on-missing contract.
//! - [`root`] — `Radio.Root`: the two render branches (composite vs plain), the context
//!   provider, the Field/Labelable integration, the hidden input and its listeners.
//! - [`indicator`] — `Radio.Indicator`: the transition-status mount machine and the
//!   mirrored attribute walk.
//!
//! Runtime law (the crate's dual-runtime split — `field/mod.rs`): leptos 0.7 tracks
//! reactive-graph 0.1 while the internals crate's hooks are 0.2-typed. The unit re-homes
//! its state machine on leptos signals (the checkbox/switch precedent) and bridges the
//! rg-0.2 handles it still consumes (`useBaseUiId`, `useLabelableId`, `useAriaLabelledBy`,
//! `useButton`) one crossing at a time.
//!
//! ## Why the parts take `Option<Option<String>>` for `value`
//!
//! Upstream's `value` admits `string | number | null` and `null` is a *selectable* value
//! (behavior.md:65), while an *absent* `value` is a separate state (the change funnel
//! refuses to commit it, `RadioRoot.tsx:190`). Both are preserved here rather than collapsed
//! — see [`crate::radio::state::is_checked`] for the equality this protects and
//! `crate::radio::state`'s module docs for the encoding, which is the one the sibling unit
//! `library: radio-group` already published for a controlled value
//! (`crates/leptos-ui/src/radio_group.rs:769`).

pub mod context;
pub mod indicator;
pub mod root;
pub mod state;

pub use context::{
    MISSING_ROOT_CONTEXT_MESSAGE, RadioRootContextValue, provide_radio_root_context,
    try_use_radio_root_context, use_radio_root_context,
};
pub use indicator::{RadioIndicatorHandlers, RadioIndicatorViewProps, radio_indicator_view};
pub use root::{
    RadioRootHandlers, RadioRootViewProps, radio_root_view,
};
pub use state::{
    DATA_CHECKED, DATA_DISABLED, DATA_DIRTY, DATA_ENDING_STYLE, DATA_FILLED, DATA_FOCUSED,
    DATA_INVALID, DATA_READONLY, DATA_REQUIRED, DATA_STARTING_STYLE, DATA_TOUCHED, DATA_UNCHECKED,
    DATA_VALID, MANAGED_STATE_ATTRIBUTES, RadioChangeEventDetails, RadioIndicatorState,
    RadioRootState, aria_bool_attr, effective_disabled, effective_read_only, effective_required,
    get_radio_state_attributes_mapping, has_value, hidden_input_id, input_style, input_value_attr,
    is_checked, radio_indicator_state_attributes, radio_state_attributes, root_id, serialize_value,
    style_string, visually_hidden, visually_hidden_input,
};

// ---------------------------------------------------------------------------
// The namespaced part surface (`Radio::Root`, `Radio::Indicator`)
// ---------------------------------------------------------------------------
//
// Upstream teaches `<Radio.Root><Radio.Indicator /></Radio.Root>`; this port's spelling is the
// same tree with Rust's path separator (`specs/docs-content/CONTRACT.md`, the React→Rust mapping
// table; the macro-level pin is `crates/leptos-ui/tests/ns_component_path.rs`). `Radio` documents
// exactly TWO parts (behavior.md "Public API surface": `Radio.Root` + `Radio.Indicator`; the
// `index.parts.ts` of the unit re-exports only those two), and both are exposed here as the
// ergonomic surface over the `radio_root_view` / `radio_indicator_view` helpers — the same view
// functions, with upstream's prop names as `view!` attributes.
//
// `pub use self::radio as Radio;` in `lib.rs` is what makes `<Radio::Root>` resolvable from a
// consumer. The flattened helpers stay for internal callers: this adds a surface and renames or
// removes nothing.

use std::rc::Rc;

use leptos::children::ChildrenFn;
use leptos::prelude::*;
use leptos_ui_internals::use_render_element::{ClassNameSource, RenderProp, StyleSource};

/// `Radio.Root` — upstream's `<Radio.Root>` (`RadioRoot.tsx:34-266`).
///
/// The props mirror upstream's own destructured set (`RadioRoot.tsx:38-51`), field for field
/// with [`RadioRootViewProps`] (which is what the body receives); the doc comment on each one
/// names the upstream line the port's implementation spec cites.
#[allow(non_snake_case)]
#[component]
pub fn Root(
    /// `value` (`:315`) — the radio's identity inside the group. Double `Option`, deliberately:
    /// `Some(Some("a".into()))` is a string identity, `Some(None)` is upstream's `value={null}`
    /// (a selectable null identity, behavior.md:65) and omitting the prop is upstream's
    /// `undefined` (an identity tag that can never match, and that the change funnel refuses to
    /// commit, `:190`). See the module docs.
    #[prop(default = None, optional)]
    value: Option<Option<String>>,
    /// `disabled` (`:319`, default `false`).
    #[prop(default = false, optional)]
    disabled: bool,
    /// `required` (`:323`, default `false`).
    #[prop(default = false, optional)]
    required: bool,
    /// `readOnly` (`:327`, default `false`).
    #[prop(default = false, optional)]
    read_only: bool,
    /// `nativeButton` (`:47`, default `false`) — render a real `<button>` instead of a `span`.
    #[prop(default = false, optional)]
    native_button: bool,
    /// `id` (`:48`) — the labelable id: the hidden input's by default, the control's under
    /// `nativeButton` (`:117`, `:131`).
    #[prop(default = None, optional)]
    id: Option<String>,
    /// `aria-labelledby` (`:44`) — the explicit override of the fallback association.
    #[prop(default = None, optional)]
    aria_labelledby: Option<String>,
    /// `inputRef` (`:331`) — a handle on the hidden input, fired at mount.
    #[prop(default = None, optional)]
    input_ref: Option<Rc<dyn Fn(Option<web_sys::HtmlInputElement>)>>,
    /// `render` (`:39`) — its element form's tag selects the visible element (`<button />`).
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// `className` (`:40`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` (`:49`) — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest's plain attributes (`:50`), applied after the internal bags.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The rest's handler members.
    #[prop(default = RadioRootHandlers::default(), optional)]
    handlers: RadioRootHandlers,
    /// The Root's subtree — the Indicator lives inside the visible control (`:248-263`).
    #[prop(default = None, optional)]
    children: Option<ChildrenFn>,
) -> impl IntoView {
    radio_root_view(RadioRootViewProps {
        value,
        disabled,
        required,
        read_only,
        aria_labelledby,
        input_ref,
        native_button,
        id,
        render_class_style: leptos_ui_internals::use_render_element::UseRenderElementComponentProps {
            class_name: class.map(ClassNameSource::Static),
            render,
            style: (!style.is_empty()).then(|| StyleSource::Static(style)),
        },
        element_attributes,
        handlers,
        children,
    })
}

/// `Radio.Indicator` — upstream's `<Radio.Indicator>` (`RadioIndicator.tsx:17-62`).
///
/// The props mirror upstream's destructured set (`RadioIndicator.tsx:21`), field for field with
/// [`RadioIndicatorViewProps`].
#[allow(non_snake_case)]
#[component]
pub fn Indicator(
    /// `keepMounted` (`:69`, default `false`) — keep the element in the DOM while inactive.
    #[prop(default = false, optional)]
    keep_mounted: bool,
    /// `render` (`:21`).
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// `className` (`:21`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` (`:21`) — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest (`:21`) — plain attributes.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The rest's handler members (`onAnimationEnd` / `onTransitionEnd`, behavior.md:60).
    #[prop(default = RadioIndicatorHandlers::default(), optional)]
    handlers: RadioIndicatorHandlers,
) -> impl IntoView {
    radio_indicator_view(RadioIndicatorViewProps {
        keep_mounted,
        render_class_style: leptos_ui_internals::use_render_element::UseRenderElementComponentProps {
            class_name: class.map(ClassNameSource::Static),
            render,
            style: (!style.is_empty()).then(|| StyleSource::Static(style)),
        },
        element_attributes,
        handlers,
    })
}
