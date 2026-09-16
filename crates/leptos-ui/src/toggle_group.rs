//! Port of `packages/react/src/toggle-group/ToggleGroup.tsx` +
//! `ToggleGroupContext.ts` + `ToggleGroupDataAttributes.ts` — the
//! `library: toggle-group` TODO item (`specs/library/toggle-group/behavior.md`,
//! `specs/library/toggle-group/implementation.md`).
//!
//! Upstream is a *provider with almost no state of its own*
//! (`ToggleGroup.tsx:20-124`): one `useControlled` value slot (`:48-53`), one
//! reducer-shaped committer (`:55-81`), a context value memoized over
//! `{ disabled, setGroupValue, value, isValueInitialized }` (`:85-93`), and one
//! `CompositeRoot` that owns every keyboard/focus behavior the unit's tests assert
//! (`:111-121`). The group never learns which items exist: each `Toggle` derives
//! `pressed` as membership in the group's value array
//! (`packages/react/src/toggle/Toggle.tsx:63-68`) and sends `(value, nextPressed,
//! details)` up through the context callback (`Toggle.tsx:96-98`) — so the value
//! array is the only shared encoding of "which items are pressed"
//! (implementation.md, "State machine / hooks used").
//!
//! ## Rust adaptations
//!
//! - **The value duality rides [`leptos_ui_utils::use_controlled`]**, the port's one
//!   controlled/uncontrolled utility (`specs/architecture.md`, "State:
//!   controlled/uncontrolled"): `controlled` is `value`/`value_source`, `default` is
//!   `defaultValue ?? EMPTY_ARRAY` (`:41`), and the exposed [`Signal`] is the
//!   reactive `groupValue` the children re-derive from.
//! - **The context is the toggle unit's own `ToggleGroupContext`**, whose shape was
//!   fixed when `library: toggle` landed (`crates/leptos-ui/src/toggle/mod.rs:113-125`)
//!   because that unit is the provider's only in-source consumer (`Toggle.tsx:8`).
//!   The port provides it once per value *snapshot* in the reactive owner chain (the
//!   `checkbox_group`/`field_root` `provide_context` bridge), and the snapshot is what
//!   `Toggle`'s membership probe reads at body time — so the group re-runs its subtree
//!   when the value changes, which is exactly the division of labour that unit records
//!   ("the group's reactive value source re-rendering the subtree is the ToggleGroup
//!   unit's concern", `toggle/mod.rs:54-56`).
//! - **The committer crosses an `Arc<dyn Fn(..) + Send + Sync>` boundary** because that
//!   is the shape the landed `ToggleGroupContext` exposes. Its captures are
//!   local-storage handles (`Signal`/`Rc`), so they ride `SendWrapper` exactly as the
//!   dialog and meter contexts do. The details payload type
//!   ([`ToggleGroupChangeEventDetails`]) is likewise the landed context's — see the
//!   alias's own note.
//! - **`CompositeRoot` is the ported composite view** (`composite_view.rs:116`), whose
//!   module docs assign its view-layer consumer to this crate's first view consumer —
//!   this unit. The description path calls [`composite_root`]; the view path reuses that
//!   same description and re-attaches its resolved handlers and attributes to the
//!   `view!`-built `<div>`, so there is ONE wiring of the roving-focus engine.
//! - **The element-level props are [`ToggleGroupElementProps`]**, not `ToggleGroupProps`:
//!   this is the crate's first unit that carries both a description builder and a
//!   `view!`-usable `#[component] ToggleGroup`, and leptos derives a `ToggleGroupProps`
//!   from that component signature — so the description's props take the longer name
//!   rather than colliding with the generated one.
//! - **The toolbar branch is structurally absent, not emulated.** Upstream reads
//!   `useToolbarRootContext(true)`/`useToolbarGroupContext()` (`:38-39`) purely to OR
//!   their `disabled` into its own (`:45-46`) and to pick the render branch
//!   (`:100,108`). `library: toolbar` is not-started, so this port has no toolbar
//!   context to read: the read is the "no provider in scope" arm upstream also takes
//!   outside a toolbar, `Boolean(toolbarContext)` is `false`, and the group always takes
//!   the `CompositeRoot` branch (`:111-121`). The toolbar-nesting behavior asserted by
//!   `ToggleGroup.test.tsx:313-329,339-365` is therefore blocked on that unit, not
//!   silently dropped.
//! - **The aria attribute battery is the default state mapping, not hand-written**:
//!   `state = { disabled, multiple, orientation }` (`:83`) flows through
//!   [`get_state_attributes_props`], which is why `data-disabled`/`data-multiple` are
//!   *absent* when false while `data-orientation` carries the orientation string — and
//!   why `aria-orientation` is never emitted (behavior.md, "Accessibility").
//! - **The component exposes `class`/`style`, not `render`** (see [`class_style_bag`]):
//!   the view path renders a fixed `<div>`, so the element form of `render` — which
//!   replaces the tag (`useRenderElement.tsx:164-196`) — would be silently dropped on
//!   that surface. The element-level props resolve it, and the gap is named in the
//!   unit's `TODO.md` entry rather than papered over by exposing a prop the view path
//!   cannot honour.
//!
use std::rc::Rc;

use leptos::prelude::*;
// Two `reactive_graph` versions coexist in this workspace: the port's signals are 0.2.x
// (`reactive_graph` direct dependency) while `NodeRef` arrives through `tachys`, whose
// `Get` is leptos's re-export. Both traits are imported under distinct names so each
// receiver resolves to the trait that actually implements it.
use leptos::prelude::Get as LeptosGet;
use reactive_graph::computed::Memo;
use reactive_graph::owner::LocalStorage;
use reactive_graph::traits::{Get, GetUntracked};
use reactive_graph::wrappers::read::Signal;
use send_wrapper::SendWrapper;
use serde_json::{Value, json};

use leptos_ui_internals::composite_view::{CompositeRootComponentProps, composite_root};
use leptos_ui_internals::create_base_ui_event_details::BaseUIChangeEventDetails;
use leptos_ui_internals::direction_context::{TextDirection, use_direction};
use leptos_ui_internals::floating_ui::element_props::{ElementAttributeFn, ElementEventHandler};
use leptos_ui_internals::floating_ui::types::Orientation as CompositeOrientation;
use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::get_state_attributes_props;
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_composite_root::{UseCompositeRootParams, use_composite_root};
use leptos_ui_internals::use_render_element::{
    ClassNameSource, RenderElementHandlers, RenderElementProps, RenderedElement, StyleSource,
    UseRenderElementComponentProps,
};
use leptos_ui_utils::use_controlled::{SetValueAction, UseControlledProps, use_controlled};
use leptos_ui_utils::use_merged_refs::InputRef;

use crate::toggle::{ToggleGroupContext, ToggleItemMetadata};

/// The engine's `className`/`style` bag built from the view layer's attribute spellings —
/// the `class_style_bag` convention `avatar/mod.rs:170` and `otp_field.rs:1813`
/// established.
///
/// `render` is deliberately fixed to `None`, and that is a MEASURED limitation rather than
/// an oversight: the `render` prop's element form selects the tag of the visible element
/// (`useRenderElement.tsx:164-196`), and this unit's view path builds a fixed `<div>`
/// ([`toggle_group_view`]), so a component-level `render` would promise a substitution the
/// view surface cannot keep. The element-level
/// [`ToggleGroupElementProps::render_class_style`] still carries it — the engine resolves
/// it (pinned by `the_render_element_form_selects_the_resolved_tag`) — for callers that
/// materialize the description themselves.
pub(crate) fn class_style_bag(
    class: Option<String>,
    style: Vec<(String, String)>,
) -> UseRenderElementComponentProps {
    UseRenderElementComponentProps {
        class_name: class.map(ClassNameSource::Static),
        render: None,
        // `None` when the caller declared no style: upstream's `style === undefined`, which
        // the engine keeps distinct from an empty style record.
        style: (!style.is_empty()).then_some(StyleSource::Static(style)),
    }
}

/// The change-event details type — upstream's
/// `BaseUIChangeEventDetails<typeof REASONS.none>` (`ToggleGroup.tsx:59,194-196`), whose
/// reason carries no payload of its own.
///
/// The payload type parameter is fixed by the *landed* `ToggleGroupContext`
/// (`toggle/mod.rs:118-120`), which spells the shared details object as
/// `BaseUIChangeEventDetails<(), MouseEvent>` because the object is created by the
/// `Toggle` child from its own click event (`Toggle.tsx:86`) and then shared upward
/// through this call — one details object, one veto chain covering item and group
/// (implementation.md, "Crossing the boundary to children").
pub type ToggleGroupChangeEventDetails = BaseUIChangeEventDetails<(), web_sys::MouseEvent>;

/// `onValueChange` (`:168-169`): `(groupValue: Value[], eventDetails)` — the *computed*
/// next array, not the current one (`:73`).
pub type OnGroupValueChange =
    std::sync::Arc<dyn Fn(Vec<String>, &ToggleGroupChangeEventDetails) + Send + Sync>;

/// `orientation` (`:178`) — upstream's `Orientation` from `internals/types`, i.e. the
/// two-valued prop the data attribute documents (`ToggleGroupDataAttributes.ts:5-8`).
/// [`CompositeOrientation::Both`] is the composite engine's own third value and is not a
/// ToggleGroup prop upstream, so it is not one here.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToggleGroupOrientation {
    /// `'horizontal'` (the `@default`, `:29`).
    #[default]
    Horizontal,
    /// `'vertical'`.
    Vertical,
}

impl ToggleGroupOrientation {
    /// The `data-orientation` value (`ToggleGroupDataAttributes.ts:5-8`).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }

    /// The composite engine's orientation input (`:120`).
    fn composite(self) -> CompositeOrientation {
        match self {
            Self::Horizontal => CompositeOrientation::Horizontal,
            Self::Vertical => CompositeOrientation::Vertical,
        }
    }
}

/// `ToggleGroupState` (`:131-147`): the record driving both the state→`data-*` mapping
/// and the `CompositeRoot` wiring (`:83,115`). The attribute names come from
/// `ToggleGroupDataAttributes.ts:1-13` — declared there, produced here by the shared
/// truthiness mapping rather than by importing the constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ToggleGroupState {
    /// `disabled` (`:135`) — the effective, toolbar-merged value (`:45-46`).
    pub disabled: bool,
    /// `multiple` (`:142`) — `false` in single-selection mode.
    pub multiple: bool,
    /// `orientation` (`:146`).
    pub orientation: ToggleGroupOrientation,
}

impl ToggleGroupState {
    /// The `serde_json` state map [`get_state_attributes_props`] walks. Yields
    /// `data-disabled`/`data-multiple` only when truthy (presence/absence semantics, not
    /// `="false"`) and `data-orientation="horizontal|vertical"`.
    pub fn to_state_map(self) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert("disabled".to_string(), json!(self.disabled));
        map.insert("multiple".to_string(), json!(self.multiple));
        map.insert(
            "orientation".to_string(),
            Value::String(self.orientation.as_str().to_string()),
        );
        map
    }

    /// The resolved `data-*` attribute battery (`name`, `value`) both layers apply —
    /// the `isTruthy` gate is what makes `data-disabled`/`data-multiple` absent when
    /// false. Order-insensitive.
    pub fn data_attributes(self) -> Vec<(String, String)> {
        get_state_attributes_props(&self.to_state_map(), None)
            .into_iter()
            .collect()
    }
}

/// `setGroupValue`'s array maths (`:61-71`) — the whole reducer, split out so the
/// runtime and the host suite exercise the identical function.
///
/// Faithful to upstream *including its edge semantics*: in `multiple` mode the removal
/// arm is `groupValue.splice(groupValue.indexOf(newValue), 1)` (`:67`), so a `newValue`
/// that is **not** in the array makes `indexOf` return `-1` and JS `splice(-1, 1)`
/// removes the **last** element. That is upstream's behavior, not a transcription slip
/// (implementation.md, "Anything in source not explained by any test", item 7).
pub fn next_group_value(
    current: &[String],
    multiple: bool,
    new_value: &str,
    next_pressed: bool,
) -> Vec<String> {
    if multiple {
        let mut next = current.to_vec();
        if next_pressed {
            next.push(new_value.to_string());
        } else {
            match current.iter().position(|value| value == new_value) {
                Some(index) => {
                    next.remove(index);
                }
                // `indexOf` → `-1`; `splice(-1, 1)` drops the last element.
                None => {
                    next.pop();
                }
            }
        }
        next
    } else if next_pressed {
        vec![new_value.to_string()]
    } else {
        Vec::new()
    }
}

/// The unit's runtime: the value duality, the effective `disabled`, the dev-warning
/// gate, and the committer's closure environment. One per group render; both the
/// description path and the view path build it from the same props.
pub struct ToggleGroupRuntime {
    /// `groupValue` (`:48`) — the exposed `useControlled` value, a reactive read.
    pub value: Signal<Vec<String>, LocalStorage>,
    /// `setValueState` (`:48`) — `useControlled`'s setter (a no-op in controlled mode,
    /// `packages/utils/src/useControlled.ts:82-89`).
    set_value_state: Rc<dyn Fn(SetValueAction<Vec<String>>)>,
    /// `multiple` (`:29`).
    pub multiple: bool,
    /// The effective `disabled` (`:45-46`; the toolbar arms are absent — module docs).
    pub disabled: bool,
    /// `isValueInitialized` (`:43`) — computed from the *raw* props so an explicit empty
    /// `defaultValue` still counts as initialized.
    pub is_value_initialized: bool,
    /// `onValueChange` (`:28`).
    pub on_value_change: Option<OnGroupValueChange>,
}

impl ToggleGroupRuntime {
    /// Builds the runtime — upstream's `:38-53` block.
    pub fn new(props: &ToggleGroupElementProps) -> Self {
        let is_value_initialized =
            props.value.is_some() || props.value_source.is_some() || props.default_value.is_some();

        // `useControlled({ controlled: valueProp, default: defaultValue ?? EMPTY_ARRAY, … })`
        // (`:41,48-53`). The port's controlled slot accepts either the one-shot prop or a
        // reactive source (the demos' `useState` analog); the uncontrolled seed is the
        // `EMPTY_ARRAY` fallback.
        let controlled: Signal<Option<Vec<String>>, LocalStorage> = match &props.value_source {
            Some(source) => source.clone(),
            None => {
                let value = props.value.clone();
                Signal::derive_local(move || value.clone())
            }
        };

        let (value, set_value_state) = use_controlled::<Vec<String>, _, _>(UseControlledProps::new(
            controlled,
            Signal::derive_local({
                let seeded = props.default_value.clone().unwrap_or_default();
                move || seeded.clone()
            }),
            "ToggleGroup",
        ));

        Self {
            value,
            set_value_state: Rc::new(set_value_state),
            multiple: props.multiple,
            disabled: props.disabled,
            is_value_initialized,
            on_value_change: props.on_value_change.clone(),
        }
    }

    /// `setGroupValue` (`:55-81`) in the `Send + Sync` shape the landed
    /// `ToggleGroupContext` carries: compute the next array, fire `onValueChange`
    /// **first**, bail on a cancel, and only then commit (`:73-79`).
    pub fn set_group_value(
        &self,
    ) -> std::sync::Arc<dyn Fn(&str, bool, &ToggleGroupChangeEventDetails) + Send + Sync> {
        let value = SendWrapper::new(self.value);
        let set_value_state = SendWrapper::new(Rc::clone(&self.set_value_state));
        let multiple = self.multiple;
        let on_value_change = self.on_value_change.clone();

        std::sync::Arc::new(
            move |new_value: &str,
                  next_pressed: bool,
                  event_details: &ToggleGroupChangeEventDetails| {
                let current = value.get_untracked();
                let new_group_value = next_group_value(&current, multiple, new_value, next_pressed);

                if let Some(on_value_change) = &on_value_change {
                    on_value_change(new_group_value.clone(), event_details);
                }

                if event_details.is_canceled() {
                    return;
                }

                set_value_state(SetValueAction::Value(new_group_value));
            },
        )
    }

    /// The context value (`:85-93`) for one value snapshot — the memo's `value` member is
    /// the array the children's membership probes read.
    pub fn context_value(&self, value: Vec<String>) -> ToggleGroupContext {
        ToggleGroupContext {
            value: std::sync::Arc::new(value),
            set_group_value: Some(self.set_group_value()),
            disabled: self.disabled,
            is_value_initialized: self.is_value_initialized,
        }
    }
}

/// The group's element-level props — upstream's destructured
/// `ToggleGroup.Props<Value>` (`:24-36`) plus the two arrivals an element-only builder
/// cannot take from a `view!` call site (`value_source`, `root_ref`).
///
/// Named `…ElementProps` rather than `ToggleGroupProps` because the `#[component]
/// ToggleGroup` below makes leptos derive a `ToggleGroupProps` from its signature
/// (module docs).
pub struct ToggleGroupElementProps {
    /// `value` (`:31,158`) — controlled; `None` while uncontrolled.
    pub value: Option<Vec<String>>,
    /// The controlled value as a reactive read — the page-owned-state form the checkbox
    /// group's `value_source` establishes (upstream's `value={value}` demos).
    pub value_source: Option<Signal<Option<Vec<String>>, LocalStorage>>,
    /// `defaultValue` (`:25,164`) — the uncontrolled seed; `?? EMPTY_ARRAY` upstream.
    pub default_value: Option<Vec<String>>,
    /// `onValueChange` (`:28`).
    pub on_value_change: Option<OnGroupValueChange>,
    /// `disabled` (`:26`, `@default false`).
    pub disabled: bool,
    /// `orientation` (`:29`, `@default 'horizontal'`).
    pub orientation: ToggleGroupOrientation,
    /// `multiple` (`:30`, `@default false`).
    pub multiple: bool,
    /// `loopFocus` (`:27`, `@default true`) — forwarded to `CompositeRoot` (`:118`).
    pub loop_focus: bool,
    /// `className`/`style`/`render` (`:32-34`).
    pub render_class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest (`:35`) — the consumer's plain attributes, the last bag
    /// of the spread (`:103,117`).
    pub element_attributes: Vec<(String, String)>,
    /// `forwardedRef` (`:22`) — `CompositeRoot`'s `refs={[forwardedRef]}` (`:116`).
    pub root_ref: Option<InputRef<web_sys::Element>>,
}

impl Default for ToggleGroupElementProps {
    fn default() -> Self {
        Self {
            value: None,
            value_source: None,
            default_value: None,
            on_value_change: None,
            disabled: false,
            orientation: ToggleGroupOrientation::default(),
            multiple: false,
            // Upstream's destructuring default (`:27`).
            loop_focus: true,
            render_class_style: UseRenderElementComponentProps::default(),
            element_attributes: Vec::new(),
            root_ref: None,
        }
    }
}

impl ToggleGroupElementProps {
    /// `state` (`:83`).
    pub fn state(&self) -> ToggleGroupState {
        ToggleGroupState {
            disabled: self.disabled,
            multiple: self.multiple,
            orientation: self.orientation,
        }
    }
}

/// The `defaultProps` bag (`:95-97`): `{ role: 'group' }`, the only base prop.
fn default_props_bag() -> RenderElementProps {
    RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: vec![(
                "role".to_string(),
                Rc::new(|| Some("group".to_string())) as ElementAttributeFn,
            )],
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    }
}

/// The `...elementProps` rest as a bag (`:103,117`) — the consumer's plain attributes.
fn element_props_bag(attributes: &[(String, String)]) -> RenderElementProps {
    RenderElementProps {
        handlers: RenderElementHandlers {
            attributes: attributes
                .iter()
                .map(|(name, value)| {
                    let value = value.clone();
                    (
                        name.clone(),
                        Rc::new(move || Some(value.clone())) as ElementAttributeFn,
                    )
                })
                .collect(),
            ..RenderElementHandlers::default()
        },
        ..RenderElementProps::default()
    }
}

/// The `CompositeRoot` wiring (`:111-121`) — shared by [`toggle_group_element`] and the
/// view path so the roving-focus engine is built exactly once. Must be called inside a
/// reactive owner; provisions the composite root + list contexts.
fn composite_root_element(props: ToggleGroupElementProps) -> RenderedElement {
    let state_map = props.state().to_state_map();
    let direction = use_direction();
    let refs: Vec<InputRef<web_sys::Element>> = props.root_ref.clone().into_iter().collect();
    let element_props = element_props_bag(&props.element_attributes);

    composite_root::<ToggleItemMetadata, Memo<Option<i32>>, Memo<TextDirection>>(
        CompositeRootComponentProps {
            render_class_style: props.render_class_style,
            tag: "div".to_string(),
            state: state_map,
            state_attributes_mapping: None,
            refs,
            // `props={[defaultProps, elementProps]}` (`:117`).
            props: vec![
                PropsSource::Static(default_props_bag()),
                PropsSource::Static(element_props),
            ],
            element_props: RenderElementProps::default(),
            on_map_change: None,
            // Not a ToggleGroup prop (`:111-121` passes none); the composite view's
            // default is `false`.
            highlight_item_on_hover: false,
        },
        UseCompositeRootParams {
            orientation: Some(props.orientation.composite()),
            grid: None,
            loop_focus: props.loop_focus,
            on_loop: None,
            highlighted_index: None,
            on_highlighted_index_change: None,
            direction,
            // Upstream's `CompositeRoot` passes `refs={[forwardedRef]}`, never a
            // `rootRef` prop, so the hook's internal root slot is the only root ref.
            root_ref: InputRef::Empty,
            // `enableHomeAndEndKeys` (`:119`).
            enable_home_and_end_keys: true,
            // `CompositeRoot.tsx:33`'s default.
            stop_event_propagation: true,
            disabled_indices: None,
            modifier_keys: Vec::new(),
        },
    )
    .expect("the composite root always renders while enabled")
}

/// Provides the group context for one value snapshot — the port's
/// `ToggleGroupContext.Provider` (`:107`). Re-called with a fresh snapshot whenever the
/// value changes, which is what makes the children's body-time membership probes see the
/// new array (module docs, "Rust adaptations").
///
/// The call is QUALIFIED deliberately: this workspace carries two `reactive_graph`
/// versions, and `leptos::prelude::provide_context` targets the other one, so the
/// prelude's re-export writes into a context map the consuming unit's
/// `reactive_graph::owner::use_context` never reads. Every other ported provider uses
/// the path below (`checkbox_group/mod.rs:202`, `field/field_root.rs:430`).
pub fn provide_toggle_group_context(runtime: &ToggleGroupRuntime, value: Vec<String>) {
    reactive_graph::owner::provide_context(runtime.context_value(value));
}

/// Builds the group's root element description — upstream's `<CompositeRoot … />` branch
/// (`:111-121`), the branch a group outside a toolbar always takes (module docs) — and
/// provisions the group context for the current value snapshot plus the composite
/// root/list pair.
///
/// Must be called inside a reactive owner. The description is either materialized
/// (`RenderedElement::create_element`) or replayed onto a `view!`-built `<div>`
/// (the view path).
pub fn toggle_group_element(props: ToggleGroupElementProps) -> RenderedElement {
    let runtime = ToggleGroupRuntime::new(&props);
    let snapshot = runtime.value.get_untracked();
    provide_toggle_group_context(&runtime, snapshot);
    composite_root_element(props)
}

/// Adapts a bag handler back to the native event a `view!` listener receives — the
/// inverse of the internals crate's `native_to_base_ui`, so the view path reattaches the
/// composite root's own keyboard/focus pipeline without rewiring the hook.
fn base_ui_to_native<E: Clone + 'static>(
    handler: ElementEventHandler<BaseUIEvent<E>>,
) -> impl Fn(&E) {
    move |event: &E| handler(&BaseUIEvent::new(event.clone()))
}

/// The view-layer props: the element props plus the consumer's subtree. Embedded rather
/// than re-listed so the two layers cannot drift apart.
pub struct ToggleGroupViewProps {
    /// The element-level props (`:24-35`).
    pub element: ToggleGroupElementProps,
    /// Upstream's `<ToggleGroup>{children}</ToggleGroup>` — `ChildrenFn` because the
    /// subtree is built once per value snapshot (module docs).
    pub children: Option<ChildrenFn>,
}

impl Default for ToggleGroupViewProps {
    fn default() -> Self {
        Self {
            element: ToggleGroupElementProps::default(),
            children: None,
        }
    }
}

/// The group root view — upstream's `<ToggleGroupContext.Provider>` wrapping the
/// `CompositeRoot` element with the consumer's subtree inside it (`:106-124`).
///
/// Must be called inside a reactive owner. The subtree is re-built whenever the group's
/// value changes, because the landed `ToggleGroupContext` hands children a value
/// *snapshot* at body time; each rebuild re-provides the context for the new snapshot
/// first, which is upstream's "provider re-renders children with the new memoized value"
/// (`:85-93`).
pub fn toggle_group_view(props: ToggleGroupViewProps) -> impl IntoView {
    let ToggleGroupViewProps { element, children } = props;

    let bridge_owner = reactive_graph::owner::Owner::new();
    let view = bridge_owner.with(move || {
        let runtime = ToggleGroupRuntime::new(&element);
        let snapshot = runtime.value.get_untracked();
        provide_toggle_group_context(&runtime, snapshot);
        let rendered = composite_root_element(element);

        // The description's resolved props, replayed onto the `view!`-built node.
        let class = rendered.props.class.clone();
        let style = {
            let declarations = &rendered.props.style;
            (!declarations.is_empty()).then(|| {
                declarations
                    .iter()
                    .map(|(property, value)| format!("{property}: {value};"))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
        };
        let resolved_attributes: Vec<(String, Option<String>)> = {
            let mut resolved: Vec<(String, Option<String>)> = Vec::new();
            if let Some(class) = class {
                resolved.push(("class".to_string(), Some(class)));
            }
            if let Some(style) = style {
                resolved.push(("style".to_string(), Some(style)));
            }
            resolved.extend(
                rendered
                    .props
                    .handlers
                    .attributes
                    .iter()
                    .map(|(name, value)| (name.clone(), value())),
            );
            resolved
        };
        let on_key_down = rendered
            .props
            .handlers
            .on_key_down
            .clone()
            .map(base_ui_to_native);
        let on_focus = rendered
            .props
            .handlers
            .on_focus
            .clone()
            .map(base_ui_to_native);
        let ref_callback = rendered.props.ref_callback.clone();

        let root_node: NodeRef<leptos::html::Div> = NodeRef::new();

        // The attribute writer — the element bag on a real node (the checkbox-group /
        // meter convention: attributes that vanish between runs are removed, so a stale
        // `data-*` never misreports state to a stylesheet or to AT).
        {
            let resolved_attributes = resolved_attributes.clone();
            Effect::new(move |_| {
                let Some(node) = root_node.get() else {
                    return;
                };
                let node: &web_sys::Element = node.as_ref();
                for (name, value) in &resolved_attributes {
                    match value {
                        Some(value) => {
                            let _ = node.set_attribute(name, value);
                        }
                        None => {
                            let _ = node.remove_attribute(name);
                        }
                    }
                }
            });
        }

        // The forwarded root ref (`CompositeRoot`'s `refs={[forwardedRef]}`) fires once
        // the node exists.
        if let Some(ref_callback) = ref_callback {
            Effect::new(move |_| {
                if let Some(node) = root_node.get() {
                    let node: &web_sys::Element = node.as_ref();
                    ref_callback(Some(node));
                }
            });
        }

        // The subtree: rebuilt per value snapshot, with the context re-provided first.
        // The runtime and the subtree closure cross a `Send + Sync` boundary because a
        // `view!` child closure is stored that way; `SendWrapper` is the crate's bridge
        // for local reactive handles (dialog/meter precedent).
        let runtime_for_children = SendWrapper::new(runtime);
        let children = SendWrapper::new(children);
        let children_view = move || {
            let snapshot = runtime_for_children.value.get();
            provide_toggle_group_context(&runtime_for_children, snapshot);
            children.as_ref().map(|children| children())
        };

        view! {
            <div
                node_ref=root_node
                on:keydown=move |event: web_sys::KeyboardEvent| {
                    if let Some(handler) = &on_key_down {
                        handler(&event);
                    }
                }
                on:focus=move |event: web_sys::FocusEvent| {
                    if let Some(handler) = &on_focus {
                        handler(&event);
                    }
                }
            >
                {children_view}
            </div>
        }
    });
    // The bridge owner outlives the subtree (the `field_root_view`/`checkbox_group_view`
    // precedent).
    std::mem::forget(bridge_owner);
    view
}

/// The `ToggleGroup` component — upstream's exported component (`:20`) in the port's
/// `view!`-usable form.
#[leptos::component]
pub fn ToggleGroup(
    /// `value` (`:158`) — controlled.
    #[prop(default = None, optional)]
    value: Option<Vec<String>>,
    /// `defaultValue` (`:164`) — the uncontrolled seed.
    #[prop(default = None, optional)]
    default_value: Option<Vec<String>>,
    /// `onValueChange` (`:168`).
    #[prop(default = None, optional)]
    on_value_change: Option<OnGroupValueChange>,
    /// `disabled` (`:174`, `@default false`).
    #[prop(default = false, optional)]
    disabled: bool,
    /// `multiple` (`:191`, `@default false`).
    #[prop(default = false, optional)]
    multiple: bool,
    /// `loopFocus` (`:184`, `@default true`).
    #[prop(default = true, optional)]
    loop_focus: bool,
    /// `orientation` (`:178`, `@default 'horizontal'`).
    #[prop(default = ToggleGroupOrientation::Horizontal, optional)]
    orientation: ToggleGroupOrientation,
    /// `className` (`:32`).
    #[prop(default = None, optional)]
    class: Option<String>,
    /// `style` (`:34`) — upstream's style record as ordered `(property, value)`
    /// declarations (the avatar/otp-field spelling).
    #[prop(default = Vec::new(), optional)]
    style: Vec<(String, String)>,
    /// The consumer's `...elementProps` attributes (`:35`).
    #[prop(default = Vec::new(), optional)]
    element_attributes: Vec<(String, String)>,
    /// The consumer's subtree.
    children: ChildrenFn,
) -> impl IntoView {
    toggle_group_view(ToggleGroupViewProps {
        element: ToggleGroupElementProps {
            value,
            default_value,
            on_value_change,
            disabled,
            orientation,
            multiple,
            loop_focus,
            render_class_style: class_style_bag(class, style),
            element_attributes,
            root_ref: None,
            value_source: None,
        },
        children: Some(children),
    })
}
