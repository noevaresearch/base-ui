//! `Input` — port of `packages/react/src/input/Input.tsx` (the `library: input` TODO
//! item; `specs/library/input/behavior.md`, `specs/library/input/implementation.md`).
//!
//! Upstream's component is one statement: a `React.forwardRef` whose body is
//! `return <Field.Control ref={forwardedRef} {...props} />`
//! (`packages/react/src/input/Input.tsx:12-17`). It calls no hooks, reads no context and owns
//! no state — `implementation.md`, "State machine / hooks used" — so the port is the same
//! delegation: the value model, the field-state `data-*` attributes, the validation wiring and
//! the rendered element all come from the ported `Field.Control`
//! (`crates/leptos-ui/src/field/field_control.rs:96-628`), and this module contributes the
//! props mapping and nothing else.
//!
//! ## The surface, and where it comes from
//!
//! `InputProps extends BaseUIComponentProps<'input', InputState>`
//! (`packages/react/src/input/Input.tsx:19`) supplies `className`, `style` and `render` plus the
//! rest-spread of arbitrary DOM props (`packages/react/src/internals/types.ts:36-61` — the
//! conformance suites spread `lang`, `data-*`, `data-testid` and `style` onto the root,
//! `packages/react/test/conformanceTests/propForwarding.tsx:23-36`), and the unit re-declares
//! exactly three narrowed members: `onValueChange`, `defaultValue`, `value`
//! (`packages/react/src/input/Input.tsx:23-31`).
//!
//! The port spells that surface the way the crate spells every component's — `class`, `style`
//! as ordered declarations, and `element_attributes` for the `...elementProps` rest
//! (`crates/leptos-ui/src/otp_field.rs:1811-1823`'s `render_class_style`,
//! `crates/leptos-ui/src/toggle_group.rs:104-127`'s `class_style_bag`) — so a caller writes the
//! same `view!` call site it writes for any other part.
//!
//! ## Recorded adaptations (never silent)
//!
//! - **`render` is not exposed.** Upstream's prop replaces the rendered element
//!   (`packages/react/src/internals/useRenderElement.tsx:164-196`; the unit's own type fixture
//!   is `render={<textarea />}`, `packages/react/src/input/Input.spec.tsx:4-6`), but the
//!   delegation target's view path builds a fixed `<input>`
//!   (`crates/leptos-ui/src/field/field_control.rs:604-628`), so a component-level `render` here
//!   would promise a substitution that path cannot keep. This is the `toggle_group.rs:64`
//!   precedent ("the component exposes `class`/`style`, not `render`") and it is inherited from
//!   `Field.Control`, not introduced here: the engine-side gap is already scoped as its own
//!   ledger item, `library: the view paths drop render's element form`.
//! - **`ref` is not exposed, for the same reason.** Upstream forwards it into the merged ref
//!   (`packages/react/src/input/Input.tsx:14`, `:16`) and the conformance suite asserts the
//!   attached instance is the rendered element
//!   (`packages/react/test/conformanceTests/refForwarding.tsx:32-38`), but the port's
//!   `FieldControlViewProps` carries no ref slot
//!   (`crates/leptos-ui/src/field/field_control.rs:34-54`): the control owns its node internally
//!   and `Input` builds no node of its own to forward one from.
//! - **`style` rides the rest-spread bag.** The delegation target has no style slot either, so
//!   the declarations are folded into `element_attributes` as a single `style` member placed
//!   BEFORE the caller's own members. That is upstream's own path — its `{...props}` spread
//!   (`packages/react/src/input/Input.tsx:16`) carries `style` into the same `...elementProps`
//!   bag the port materializes as `element_attributes` — and the position preserves upstream's
//!   later-bag-wins merge order (`mergeProps`, `packages/react/src/merge-props/mergeProps.ts:166-184`).
//! - **Props are static per body run** (the meter/checkbox view convention): a runtime prop
//!   change is the caller's re-invocation, and the view is not reactive over its own props.

use std::rc::Rc;

use leptos::prelude::*;

use crate::field::field_control::{
    FieldControlViewProps, ValueChangeEventDetails, field_control_view,
};

/// The `onValueChange` handler — upstream's
/// `((value: string, eventDetails: Input.ChangeEventDetails) => void)`
/// (`packages/react/src/input/Input.tsx:23`); `details.cancel()` is the shared veto cell the
/// control reads back (`crates/leptos-ui/src/field/field_control.rs:82-92`).
pub type InputChangeHandler = Rc<dyn Fn(String, &ValueChangeEventDetails)>;

/// The view-level props — upstream's `InputProps` (`packages/react/src/input/Input.tsx:19-31`),
/// i.e. everything `Input` hands to `Field.Control` through its `{...props}` spread
/// (`packages/react/src/input/Input.tsx:16`).
pub struct InputViewProps {
    /// `className` (`packages/react/src/internals/types.ts:36-61`) — the port's `class`.
    ///
    /// Upstream also accepts the function form of `className`
    /// (`packages/react/test/conformanceTests/renderProp.tsx:163-178`); this view layer takes
    /// the static spelling only, the crate's component-surface convention.
    pub class: Option<String>,
    /// `style` (`packages/react/src/internals/types.ts:36-61`) — ordered declarations; see the
    /// module docs for why they ride the `element_attributes` bag.
    pub style: Vec<(String, String)>,
    /// Upstream's `...props` rest (`packages/react/src/input/Input.tsx:16`): the arbitrary DOM
    /// props the conformance suites spread onto the root (`placeholder`, `required`, `data-*`,
    /// `lang`, `aria-*` — `packages/react/test/conformanceTests/propForwarding.tsx:23-36`).
    pub element_attributes: Vec<(String, String)>,
    /// `id` — the explicit id (`packages/react/src/field/control/FieldControl.tsx:40`).
    pub id: Option<String>,
    /// `name` (`packages/react/src/field/control/FieldControl.tsx:41`).
    pub name: Option<String>,
    /// `value` — the controlled member (`packages/react/src/input/Input.tsx:31`).
    pub value: Option<String>,
    /// `defaultValue` — the uncontrolled seed (`packages/react/src/input/Input.tsx:27`).
    pub default_value: Option<String>,
    /// `disabled` (`packages/react/src/field/control/FieldControl.tsx:43`).
    pub disabled: bool,
    /// `autoFocus` — the delegation target's own member
    /// (`packages/react/src/field/control/FieldControl.tsx:46`).
    pub auto_focus: bool,
    /// `onValueChange` (`packages/react/src/input/Input.tsx:23`).
    pub on_value_change: Option<InputChangeHandler>,
}

impl Default for InputViewProps {
    fn default() -> Self {
        InputViewProps {
            class: None,
            style: Vec::new(),
            element_attributes: Vec::new(),
            id: None,
            name: None,
            value: None,
            default_value: None,
            disabled: false,
            auto_focus: false,
            on_value_change: None,
        }
    }
}

/// Upstream's entire body (`packages/react/src/input/Input.tsx:12-17`): the caller's props
/// arrive at `Field.Control` unchanged.
///
/// The ONLY transformation is the `style` fold the module docs describe ("Recorded
/// adaptations"); with no style declared this function is the identity over the caller's
/// members, which is the delegation contract the host suite pins field by field.
pub fn input_view_props(props: InputViewProps) -> FieldControlViewProps {
    let InputViewProps {
        class,
        style,
        element_attributes,
        id,
        name,
        value,
        default_value,
        disabled,
        auto_focus,
        on_value_change,
    } = props;

    let mut merged_attributes = Vec::with_capacity(element_attributes.len() + 1);
    if !style.is_empty() {
        merged_attributes.push(("style".to_string(), style_declarations(&style)));
    }
    // The caller's own members follow, so a `style` they pass through `element_attributes`
    // still wins — upstream's later-bag-wins order (`mergeProps.ts:166-184`) over the bag the
    // component prop populated first.
    merged_attributes.extend(element_attributes);

    FieldControlViewProps {
        id,
        name,
        value,
        default_value,
        disabled,
        on_value_change,
        auto_focus,
        class,
        element_attributes: merged_attributes,
    }
}

/// `"color: red; display: block;"` — the declaration spelling the crate's own style writers
/// use (`crates/leptos-ui/src/fieldset/root.rs:336-344`,
/// `crates/leptos-ui/src/checkbox/root.rs:1281-1289`).
pub fn style_declarations(style: &[(String, String)]) -> String {
    style
        .iter()
        .map(|(property, value)| format!("{property}: {value};"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The public `Input` component — upstream's `<Input />`
/// (`packages/react/src/input/Input.tsx:12`).
///
/// A single-part component with no subcomponents (behavior.md, "Public API surface": the whole
/// conformance suite mounts the bare element), so there is no `Input::Root` to add: the call
/// site is `<Input .. />`, exactly as upstream teaches it.
#[component]
pub fn Input(
    /// `className` — the port's `class`.
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` — ordered declarations.
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// Upstream's `...props` rest — plain DOM attributes (`placeholder`, `required`, `data-*`).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The explicit id.
    #[prop(default = None, optional)]
    id: Option<String>,
    /// The control's name.
    #[prop(default = None, optional)]
    name: Option<String>,
    /// The controlled value.
    #[prop(default = None, optional)]
    value: Option<String>,
    /// The uncontrolled seed.
    #[prop(default = None, optional)]
    default_value: Option<String>,
    /// `disabled`.
    #[prop(default = false, optional)]
    disabled: bool,
    /// `autoFocus`.
    #[prop(default = false, optional)]
    auto_focus: bool,
    /// `onValueChange(value, details)`.
    #[prop(default = None, optional)]
    on_value_change: Option<InputChangeHandler>,
) -> impl IntoView {
    field_control_view(input_view_props(InputViewProps {
        class,
        style,
        element_attributes,
        id,
        name,
        value,
        default_value,
        disabled,
        auto_focus,
        on_value_change,
    }))
}
