//! Port of the Base UI Checkbox unit (`packages/react/src/checkbox`) — the
//! `library: checkbox` TODO item (`specs/library/checkbox/behavior.md`,
//! `specs/library/checkbox/implementation.md`,
//! `specs/library/checkbox/implementation.citations.json`).
//!
//! Upstream's shape (implementation.md, "State machine / hooks used"): no reducer —
//! one controlled boolean (`useControlled`, `CheckboxRoot.tsx:137-145`) plus a
//! render-only `indeterminate` flag, wrapped in hooks that each own one concern
//! (`useButton`, the two layout effects, `useValueChanged`,
//! `useRegisterFieldControl`, the labelable pair, the CheckboxGroup/Field/Form
//! contexts). All user interaction funnels through the hidden input's single native
//! `change` event (`:211-246`); the visible control re-dispatches its click onto that
//! input (`:365-378`).
//!
//! Module map:
//! - [`state`] — the pure folds, the state record, the state→attribute walk and the
//!   change-funnel gates (host-testable, view-independent).
//! - [`root`] — `Checkbox.Root`: the three-sibling DOM, the context provider, the
//!   Field/Form/Labelable/CheckboxGroup integration, the listeners.
//! - [`indicator`] — `Checkbox.Indicator`: the transition-status mount machine and
//!   the mirrored attribute walk.
//!
//! Runtime law (the crate's dual-runtime split — `field/mod.rs`): leptos 0.7 tracks
//! reactive-graph 0.1 while the internals crate's hooks are 0.2-typed. The unit
//! re-homes its state machine on leptos signals (the accordion/`Field.Control`
//! precedent) and bridges the rg-0.2 handles it still consumes
//! (`CheckboxGroupContext.value`, `useLabelableId`, `useAriaLabelledBy`,
//! `useTransitionStatus`, `useOpenChangeComplete`) one crossing at a time.

pub mod indicator;
pub mod root;
pub mod state;

pub use indicator::{
    CheckboxIndicatorHandlers, CheckboxIndicatorRenderState, CheckboxIndicatorViewProps,
    checkbox_indicator_view,
};
pub use root::{
    CheckboxRootContextValue, CheckboxRootHandlers, CheckboxRootViewProps,
    MISSING_ROOT_CONTEXT_MESSAGE, checkbox_root_view, provide_checkbox_root_context,
    try_use_checkbox_root_context, use_checkbox_root_context,
};
pub use state::{
    ChangeFunnel, CheckboxChangeEventDetails, CheckboxRootState, DATA_CHECKED, DATA_DIRTY,
    DATA_DISABLED, DATA_FILLED, DATA_FOCUSED, DATA_INDETERMINATE, DATA_INVALID, DATA_READONLY,
    DATA_REQUIRED, DATA_TOUCHED, DATA_UNCHECKED, DATA_VALID, PARENT_CHECKBOX, change_event_details,
    checkbox_state_attributes, computed_checked, computed_indeterminate, controlled_checked,
    effective_disabled, effective_name, effective_value, get_checkbox_state_attributes_mapping,
    indicator_rendered, indicator_should_render, input_id, input_name, input_value,
    run_change_funnel, should_render_unchecked_value_input, splice_group_value,
};

// ---------------------------------------------------------------------------
// The namespaced part surface (`Checkbox::Root`, `Checkbox::Indicator`)
// ---------------------------------------------------------------------------
//
// Upstream teaches `<Checkbox.Root><Checkbox.Indicator /></Checkbox.Root>`; this port's spelling
// is the same tree with Rust's path separator (`specs/docs-content/CONTRACT.md`, the React→Rust
// mapping table; the macro-level pin is `crates/leptos-ui/tests/ns_component_path.rs`).
// `Checkbox` documents exactly TWO parts (behavior.md "Public API surface": "only Root +
// Indicator are public"), and both were until now reachable only as the flattened
// `checkbox_root_view(..) / checkbox_indicator_view(..)` helpers — behaviour without the
// ergonomics (`check-part-surface.mjs`: "2 exist only in the flattened form").
//
// These two components are the ergonomic surface over those helpers: the SAME view functions, with
// upstream's prop names as view! attributes. `pub use self::checkbox as Checkbox;` in `lib.rs` is
// what makes `<Checkbox::Root>` resolvable from a consumer. The flattened helpers stay for
// internal callers (this adds a surface, it renames and removes nothing).

use std::rc::Rc;

use leptos::children::{Children, ChildrenFn};
use leptos::prelude::*;
use leptos_ui_internals::use_render_element::RenderProp;

/// `Checkbox.Root` — upstream's `<Checkbox.Root>` (`CheckboxRoot.tsx:46-408`).
///
/// The props mirror upstream's own destructured set (`CheckboxRoot.tsx:50-71`), field for field
/// with [`CheckboxRootViewProps`] (which is what the body receives); the doc comment on each one
/// names the upstream line the port's implementation spec cites.
#[allow(non_snake_case)]
#[component]
pub fn Root(
    /// `checked` (`:51`) — the controlled prop; omitting it is uncontrolled.
    #[prop(default = None, optional)]
    checked: Option<bool>,
    /// `defaultChecked` (`:53`, default `false`).
    #[prop(default = None, optional)]
    default_checked: Option<bool>,
    /// `onCheckedChange(checked, eventDetails)` (`:61`) — vetoable via `details.cancel()`.
    #[prop(default = None, optional)]
    on_checked_change: Option<Rc<dyn Fn(bool, &CheckboxChangeEventDetails)>>,
    /// `disabled` (`:55`, default `false`).
    #[prop(default = false, optional)]
    disabled: bool,
    /// `readOnly` (`:63`, default `false`).
    #[prop(default = false, optional)]
    read_only: bool,
    /// `required` (`:65`, default `false`).
    #[prop(default = false, optional)]
    required: bool,
    /// `indeterminate` (`:58`, default `false`) — the render-only flag.
    #[prop(default = false, optional)]
    indeterminate: bool,
    /// The reactive `indeterminate` source (the docs' NESTED parent recipe); `None` keeps the
    /// one-shot [`Root`]`indeterminate` flag (see [`CheckboxRootViewProps::indeterminate_source`]).
    #[prop(default = None, optional)]
    indeterminate_source: Option<Signal<bool>>,
    /// `name` (`:60`) — the form field name.
    #[prop(default = None, optional)]
    name: Option<String>,
    /// `form` (`:56`) — an external form id.
    #[prop(default = None, optional)]
    form: Option<String>,
    /// `id` (`:57`) — the labelable control's id.
    #[prop(default = None, optional)]
    id: Option<String>,
    /// `value` (`:67`) — falls back to `name` (`:96`).
    #[prop(default = None, optional)]
    value: Option<String>,
    /// `uncheckedValue` (`:66`).
    #[prop(default = None, optional)]
    unchecked_value: Option<String>,
    /// `parent` (`:62`, default `false`) — a group parent.
    #[prop(default = false, optional)]
    parent: bool,
    /// `nativeButton` (`:68`, default `false`).
    #[prop(default = false, optional)]
    native_button: bool,
    /// `aria-labelledby` (`:54`) — the explicit override.
    #[prop(default = None, optional)]
    aria_labelledby: Option<String>,
    /// `inputRef` (`:59`) — the hidden input's ref callback.
    #[prop(default = None, optional)]
    input_ref: Option<Rc<dyn Fn(Option<web_sys::HtmlInputElement>)>>,
    /// `render` (`:64`) — its element form's tag selects the visible element (`<button />` for
    /// `nativeButton`); its props merge after the internal bags.
    #[prop(default = None, optional)]
    render: Option<RenderProp>,
    /// `className` (`:52`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` (`:69`) — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest's plain attributes (`:70`), applied last-but-one.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The rest's handler members.
    #[prop(default = CheckboxRootHandlers::default(), optional)]
    handlers: CheckboxRootHandlers,
    /// The parts subtree (the Indicator lives here upstream, `:401`).
    children: Children,
) -> impl IntoView {
    checkbox_root_view(CheckboxRootViewProps {
        checked,
        default_checked,
        on_checked_change,
        disabled,
        read_only,
        required,
        indeterminate,
        indeterminate_source,
        name,
        form,
        id,
        value,
        unchecked_value,
        parent,
        native_button,
        aria_labelledby,
        input_ref,
        render,
        class,
        style,
        element_attributes,
        handlers,
        children: Some(children),
    })
}

/// `Checkbox.Indicator` — upstream's `<Checkbox.Indicator>` (`CheckboxIndicator.tsx:15-...`).
///
/// The props mirror upstream's destructured set (`CheckboxIndicator.tsx:23`), field for field with
/// [`CheckboxIndicatorViewProps`].
#[allow(non_snake_case)]
#[component]
pub fn Indicator(
    /// `keepMounted` (`:23`, default `false`).
    #[prop(default = false, optional)]
    keep_mounted: bool,
    /// `className` (`:23`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The `...elementProps` rest (`:23`) — plain attributes.
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The rest's handler members.
    #[prop(default = CheckboxIndicatorHandlers::default(), optional)]
    handlers: CheckboxIndicatorHandlers,
    /// The `render` prop's *function* arm (`:23` + `useRenderElement.tsx:165-170`).
    #[prop(default = None, optional)]
    render: Option<Rc<dyn Fn(CheckboxIndicatorRenderState) -> AnyView>>,
    /// The consumer's children — re-emitted on each mount transition.
    #[prop(default = None, optional)]
    children: Option<ChildrenFn>,
) -> impl IntoView {
    checkbox_indicator_view(CheckboxIndicatorViewProps {
        keep_mounted,
        class,
        style,
        element_attributes,
        handlers,
        children,
        render,
    })
}
