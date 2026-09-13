//! `AvatarImage` — `image/AvatarImage.tsx` (the whole component) plus
//! `useImageLoadingStatus.ts` (the probe, this file's `use_image_loading_status`
//! section) and `AvatarImageDataAttributes.ts` (the attribute constants).
//!
//! See the module docs on `super` for the two-tier machine and the Rust adaptations
//! (rg-0.2 canonical signals + leptos mirrors, the pluggable probe factory, the
//! rebuild-wrapper presence, the listener seam).

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use leptos::prelude::*;
use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;
use web_sys::Element;

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::{
    StateAttributeProps, transition_status_mapping,
};
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_open_change_complete::{
    UseOpenChangeCompleteParams, use_open_change_complete,
};
use leptos_ui_internals::use_render_element::{
    UseRenderElementComponentProps, UseRenderElementParams, static_attr, use_render_element,
};
use leptos_ui_internals::use_transition_status::{
    TransitionStatus, use_transition_status,
};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};

use super::context::{AvatarRootContextValue, ImageLoadingStatus, use_avatar_root_context};
use super::root::avatar_state_attributes_mapping;

/// `data-loading` (`AvatarImageDataAttributes.ts:6` — present while the image is
/// loading, keepMounted mode only).
pub const LOADING_DATA_ATTR: &str = "data-loading";
/// `data-error` (`:10` — present when the image failed to load, keepMounted only).
pub const ERROR_DATA_ATTR: &str = "data-error";

// ---------------------------------------------------------------------------
// The probe — `useImageLoadingStatus`'s `new window.Image()` (this file's
// `useImageLoadingStatus.ts:14-71` analog)
// ---------------------------------------------------------------------------

/// The probe's request configuration (`UseImageLoadingStatusOptions` +
/// `src`, `useImageLoadingStatus.ts:7-12`): applied to the detached image in the
/// upstream assignment order — `referrerPolicy`, `crossOrigin ?? null`, `sizes`,
/// `srcset`, `src` (`:46-58`).
#[derive(Clone, Debug, Default)]
pub struct ProbeConfig {
    /// `referrerPolicy` (`:8`).
    pub referrer_policy: Option<String>,
    /// `crossOrigin` (`:9`).
    pub cross_origin: Option<String>,
    /// `sizes` (`:10`).
    pub sizes: Option<String>,
    /// `srcSet` (`:11`).
    pub src_set: Option<String>,
    /// `src`.
    pub src: Option<String>,
}

/// One live probe image — upstream's `window.Image` instance plus its
/// `onload`/`onerror` wiring (`:33-45`). Dropping the implementation is allowed to
/// disconnect the listeners (the cleanup that only flips `isMounted` in upstream
/// terms); a probe that should keep loading outlives this struct through
/// `std::mem::forget` at the scheduling site.
pub trait Probe {
    /// The image element the events fire on (the fake's exposed stand-in rides the
    /// same accessor the wasm tests drive).
    fn as_event_target(&self) -> web_sys::EventTarget;
    /// `image.complete` (`:61`).
    fn complete(&self) -> bool;
    /// `image.naturalWidth` (`:62`).
    fn natural_width(&self) -> u32;
    /// `image.onload = …` (`:44`).
    fn set_on_load(&mut self, callback: Rc<dyn Fn()>);
    /// `image.onerror = …` (`:45`).
    fn set_on_error(&mut self, callback: Rc<dyn Fn()>);
}

/// The default factory's probe — a detached `document.createElement("img")`, never
/// inserted into the document: the real `window.Image()` (`:33`; a detached
/// `HTMLImageElement` and `new Image()` are behaviorally equivalent for load
/// probing). The closures are held for the struct's lifetime; dropping
/// disconnects both listeners.
pub(crate) struct RealProbe {
    image: web_sys::HtmlImageElement,
    on_load: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
    on_error: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
}

impl Probe for RealProbe {
    fn as_event_target(&self) -> web_sys::EventTarget {
        self.image.clone().into()
    }

    fn complete(&self) -> bool {
        self.image.complete()
    }

    fn natural_width(&self) -> u32 {
        self.image.natural_width()
    }

    fn set_on_load(&mut self, callback: Rc<dyn Fn()>) {
        let closure =
            wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()) as Box<dyn FnMut()>);
        self.image
            .set_onload(Some(closure.as_ref().unchecked_ref()));
        self.on_load = Some(closure);
    }

    fn set_on_error(&mut self, callback: Rc<dyn Fn()>) {
        let closure =
            wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()) as Box<dyn FnMut()>);
        self.image
            .set_onerror(Some(closure.as_ref().unchecked_ref()));
        self.on_error = Some(closure);
    }
}

/// The probe constructor — upstream's `new window.Image()` (`:33`) behind a trait
/// object so the deterministic fake of the upstream suite
/// (`AvatarImage.test.tsx:27-73`'s `window.Image` stub: `completeOnSet`,
/// `naturalWidth`) can be injected in tests.
pub type ProbeFactory = Rc<dyn Fn() -> Box<dyn Probe>>;

/// The default factory (the real detached image).
pub fn default_probe_factory() -> ProbeFactory {
    Rc::new(|| -> Box<dyn Probe> {
        let document = web_sys::window()
            .expect("no window")
            .document()
            .expect("no document");
        Box::new(RealProbe {
            image: document
                .create_element("img")
                .expect("create img")
                .dyn_into()
                .expect("the img is an HtmlImageElement"),
            on_load: None,
            on_error: None,
        })
    })
}

/// The config assignment onto the probe (`:46-58`), verbatim order and guards —
/// including the unconditional `crossOrigin` assignment (`:49`,
/// `crossOrigin ?? null`, implementation.md untested item 6).
fn configure_probe(probe: &mut dyn Probe, config: &ProbeConfig) {
    // The `if (referrerPolicy)` guard (`:46`).
    if let Some(policy) = &config.referrer_policy {
        let _ = probe
            .as_event_target()
            .dyn_into::<web_sys::HtmlImageElement>()
            .map(|image| image.set_referrer_policy(policy.as_str()));
    }
    // `image.crossOrigin = crossOrigin ?? null` — unconditional (`:49`).
    if let Ok(image) = probe.as_event_target().clone().dyn_into::<web_sys::HtmlImageElement>()
    {
        image.set_cross_origin(config.cross_origin.as_deref());
    }
    // `if (sizes)` (`:50`).
    if let Some(sizes) = &config.sizes {
        if let Ok(image) = probe.as_event_target().clone().dyn_into::<web_sys::HtmlImageElement>()
        {
            image.set_sizes(sizes);
        }
    }
    // `if (srcSet)` — the IDL attribute is `srcset` (`:53`).
    if let Some(src_set) = &config.src_set {
        if let Ok(image) = probe.as_event_target().clone().dyn_into::<web_sys::HtmlImageElement>()
        {
            image.set_srcset(src_set);
        }
    }
    // `if (src)` (`:56`).
    if let Some(src) = &config.src {
        if let Ok(image) = probe.as_event_target().clone().dyn_into::<web_sys::HtmlImageElement>()
        {
            image.set_src(src);
        }
    }
}

// ---------------------------------------------------------------------------
// Props / state
// ---------------------------------------------------------------------------

/// The user's `onLoad`/`onError` handler shape — the [`BaseUIEvent`]-wrapped
/// dispatch (`preventBaseUIHandler` cancellation, behavior.md *Events*).
pub type UserImageEventHandler = Rc<dyn Fn(&BaseUIEvent<web_sys::Event>)>;

/// The user's `onLoadingStatusChange` — the plain status callback (behavior.md
/// *Events*: "a callback prop, not a DOM event").
pub type OnLoadingStatusChange = Rc<dyn Fn(ImageLoadingStatus)>;

/// The `AvatarImage` props (`AvatarImageProps`, `AvatarImage.tsx:188-203`).
pub struct AvatarImageProps {
    /// `className`/`style`/`render` (`:31-44` destructuring) — the engine's
    /// component-props vocabulary; `render` is the element or callback form
    /// (behavior.md *Public API surface*: both proven upstream).
    pub class_style: UseRenderElementComponentProps,
    /// The `...elementProps` rest — the MIDDLE bag: overrides the internal status
    /// attributes (an explicit `aria-hidden` survives, `:110`'s merge contract) but
    /// loses to `sourceProps` on `src`/`sizes`/`srcSet` keys.
    pub element_attributes: Vec<(String, String)>,
    /// The user's `onLoad` (`:111-113` chaining) — runs before the internal update;
    /// `prevent_base_ui_handler()` cancels it.
    pub on_load: Option<UserImageEventHandler>,
    /// The user's `onError` (`:114-116`).
    pub on_error: Option<UserImageEventHandler>,
    /// `onLoadingStatusChange` (`:196`).
    pub on_loading_status_change: Option<OnLoadingStatusChange>,
    /// `keepMounted` (`:202`, default `false`).
    pub keep_mounted: bool,
    /// `sizes` (`:40` — `sourceProps`, applied before `src`, `:156-158`).
    pub sizes: Option<String>,
    /// `srcSet` (`:41` — `sourceProps`, `:159-161`).
    pub src_set: Option<String>,
    /// `src` (`:42` — `sourceProps`, last, `:162-164`).
    pub src: Option<String>,
    /// The forwarded `ref` (`:29` — `HTMLImageElement`).
    pub ref_callback: Option<RefCallback<Element>>,
}

impl Default for AvatarImageProps {
    fn default() -> Self {
        Self {
            class_style: Default::default(),
            element_attributes: Vec::new(),
            on_load: None,
            on_error: None,
            on_loading_status_change: None,
            keep_mounted: false,
            sizes: None,
            src_set: None,
            src: None,
            ref_callback: None,
        }
    }
}

/// The resolved state the element builder consumes — the per-rebuild snapshot of
/// `AvatarImageState` (`AvatarImage.tsx:146-151`) plus the two flags the DOM shape
/// depends on.
#[derive(Clone, Copy, Debug)]
pub struct AvatarImageStateSnapshot {
    /// The image-local status.
    pub image_loading_status: ImageLoadingStatus,
    /// The masked transition status (`:150` — `keepMounted && 'ending'` →
    /// `undefined`).
    pub transition_status: Option<TransitionStatus>,
    /// `keepMounted` (`:101` — the status-attribute scope).
    pub keep_mounted: bool,
    /// `mounted` (`:153` — the presence gate).
    pub mounted: bool,
}

/// `shouldRender` (`:153`).
pub fn should_render(snapshot: &AvatarImageStateSnapshot) -> bool {
    snapshot.keep_mounted || snapshot.mounted
}

/// The combined `stateAttributesMapping` (`AvatarImage.tsx:16-19`): the root's
/// `imageLoadingStatus → null` suppression plus `transitionStatusMapping`.
/// Exported for the host-suite mapping pins.
pub fn avatar_image_state_attributes_mapping<'a>()
-> impl Fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>> + 'a {
    move |key: &str, value: &serde_json::Value| {
        if key == "imageLoadingStatus" {
            // The suppression (`stateAttributesMapping.ts:2`).
            return Some(None);
        }
        // `...transitionStatusMapping` (`:18`) — declines non-transition keys with
        // `None`, and `idle`/empty values with `Some(None)` (the default walk then
        // skips them).
        match transition_status_mapping(key, value) {
            decision @ (Some(_) | None) => decision,
        }
    }
}

/// The keepMounted status-attribute bag (`renderedStatusProps`,
/// `AvatarImage.tsx:101-118`) — the three lazy attributes; the load/error
/// listeners ride the element seam (the module docs). `None` values are the
/// upstream `undefined` (omitted).
fn rendered_status_attributes(
    status: ImageLoadingStatus,
) -> Vec<(&'static str, leptos_ui_internals::use_render_element::RenderAttributeFn)> {
    let status_cell = Cell::new(status);
    vec![
        // `data-loading: status === 'loading' ? '' : undefined` (`:106`).
        (
            LOADING_DATA_ATTR,
            Rc::new(move || {
                (status_cell.get() == ImageLoadingStatus::Loading).then(|| String::new())
            }) as _,
        ),
        // `data-error: status === 'error' ? '' : undefined` (`:107`).
        (
            ERROR_DATA_ATTR,
            Rc::new(move || (status_cell.get() == ImageLoadingStatus::Error).then(|| String::new()))
                as _,
        ),
        // `aria-hidden: status !== 'loaded' || undefined` (`:110`) — React renders
        // `true` as the string `"true"`.
        (
            "aria-hidden",
            Rc::new(move || (status_cell.get() != ImageLoadingStatus::Loaded).then(|| "true".to_string()))
                as _,
        ),
    ]
}

/// Builds the `<img>` element description — upstream's `useRenderElement('img', …)`
/// call (`AvatarImage.tsx:166-172`) as a pure function of the state snapshot, for
/// the host suite and the wasm rebuild loop alike. `None` is upstream's `!shouldRender
→ null`
/// (`:174-176`).
pub fn avatar_image_element(
    snapshot: AvatarImageStateSnapshot,
    props: &AvatarImageProps,
) -> Option<RenderedElement> {
    if !should_render(&snapshot) {
        return None;
    }

    // The state record (`:146-151`) — the suppression mapping declines the first
    // member; the transition mapping emits the style hooks.
    let mut state = serde_json::Map::new();
    state.insert(
        "imageLoadingStatus".to_string(),
        serde_json::Value::String(snapshot.image_loading_status.as_str().to_string()),
    );
    state.insert(
        "transitionStatus".to_string(),
        match snapshot.transition_status {
            Some(status) => serde_json::Value::String(match status {
                TransitionStatus::Starting => "starting".to_string(),
                TransitionStatus::Ending => "ending".to_string(),
                TransitionStatus::Idle => "idle".to_string(),
            }),
            None => serde_json::Value::Null,
        },
    );

    // Bag 1 — `renderedStatusProps` (`:101-118`), keepMounted-scoped (`:undefined`
    // otherwise: upstream spreads `undefined`, which contributes nothing).
    let status_bag = if snapshot.keep_mounted {
        let mut bag = RenderElementPropsShim::default();
        for (name, value) in rendered_status_attributes(snapshot.image_loading_status) {
            bag.attributes.push((name.to_string(), value));
        }
        bag
    } else {
        RenderElementPropsShim::default()
    };

    // Bag 2 — `elementProps` (`:43` rest): user attributes + the user's load/error
    // handlers chained FIRST (the mergeProps handler order — the user's handler runs
    // before the internal update and can prevent it).
    let mut element_bag = RenderElementPropsShim::default();
    for (name, value) in &props.element_attributes {
        element_bag
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }
    if let Some(on_load) = props.on_load.clone() {
        element_bag.on_load = Some(Rc::new(move |event: &BaseUIEvent<web_sys::Event>| on_load(event)));
    }
    if let Some(on_error) = props.on_error.clone() {
        element_bag.on_error =
            Some(Rc::new(move |event: &BaseUIEvent<web_sys::Event>| on_error(event)));
    }

    // Bag 3 — `sourceProps` (`:154-164`): the guarded conditional construction, in
    // `sizes` → `srcSet` → `src` order (the fetch-order shim, `:37-44`).
    let mut source_bag = RenderElementPropsShim::default();
    if let Some(sizes) = &props.sizes {
        source_bag.attributes.push(("sizes".to_string(), static_attr(sizes.clone())));
    }
    if let Some(src_set) = &props.src_set {
        source_bag.attributes.push(("srcset".to_string(), static_attr(src_set.clone())));
    }
    if let Some(src) = &props.src {
        source_bag.attributes.push(("src".to_string(), static_attr(src.clone())));
    }

    let props_bags = vec![
        PropsSource::Static(status_bag.into_render_element_props()),
        PropsSource::Static(element_bag.into_render_element_props()),
        PropsSource::Static(source_bag.into_render_element_props()),
    ];

    // The ref fork inputs (`:168`'s `[forwardedRef, imageRef]` — the engine adds the
    // bag/render-element slots in front, per its fork contract).
    let mut refs: Vec<InputRef<Element>> = Vec::new();
    if let Some(callback) = &props.ref_callback {
        refs.push(InputRef::Callback(callback.clone()));
    }

    let mapping = avatar_image_state_attributes_mapping();
    use_render_element(
        "img",
        // The component props ride through: className/style resolve against the
        // state; `render` replaces/merges wholesale.
        UseRenderElementComponentProps {
            class_name: props.class_style.class_name.take_from(&props.class_style),
            render: props.class_style.render.clone(),
            style: props.class_style.style.clone(),
        },
        UseRenderElementParams {
            enabled: true,
            state: &state,
            refs,
            props: props_bags,
            state_attributes_mapping: Some(&mapping),
        },
    )
}

// ---------------------------------------------------------------------------
// The live hook — the full `AvatarImage` body
// ---------------------------------------------------------------------------

/// The `AvatarImage` handle: the container element the dynamic image view renders
/// into (see [`AvatarImageView`]) plus the mirrors the caller's tree tracks.
pub struct UseAvatarImage {
    /// The root status the ROOT mirror reads — already wired through the context by
    /// the hook (the fan-out writes it).
    pub root_status: RgRwSignal<ImageLoadingStatus>,
}

/// `AvatarImage` (`AvatarImage.tsx:27-179`) — the full body. Must be called inside a
/// reactive owner (a component body). Returns the live part handle; the view layer
/// renders [`AvatarImageView`]'s container (built here).
///
/// Wasm-only: the probe needs the DOM. On the host target the body still wires the
/// machine minus the probe scheduling (the host suite pins the machine through the
/// pure builders and the bridge units).
pub fn use_avatar_image(props: &AvatarImageProps) -> UseAvatarImage {
    // The root context (`:46`) — the writer side.
    let context: AvatarRootContextValue = use_avatar_root_context();

    // Tier 1 — the image-local status (`:47-51`'s `useImageLoadingStatus`), the
    // canonical signal.
    let image_status = RgRwSignal::new_local(ImageLoadingStatus::Idle);

    // The keepMounted in-place element (`:56`'s `imageRef`) and the first-commit
    // marker (`:57`).
    let image_ref: Rc<RefCell<Option<Element>>> = Rc::new(RefCell::new(None));
    let initial_commit = Rc::new(Cell::new(true));

    // -- The probe (`useImageLoadingStatus.ts:22-63`), default mode only --
    // (`enabled === false` — keepMounted — bails before touching `window.Image`,
    // `:22-25`; the port schedules once per body run, the static-prop adaptation of
    // the dep-array restart).
    #[cfg(target_arch = "wasm32")]
    if !props.keep_mounted {
        let mut probe = (default_probe_factory())();
        // `'loading'` set synchronously (`:43`).
        Set::set(&image_status, ImageLoadingStatus::Loading);
        {
            // The `isMounted`-guarded updater (`:35-41`) — in the port, late writes
            // reach only abandoned signals (the rebuild model), so the guard is the
            // signal identity itself; the wiring keeps the event → status mapping.
            let status = image_status.clone();
            probe.set_on_load(Rc::new(move || Set::set(&status, ImageLoadingStatus::Loaded)));
        }
        {
            let status = image_status.clone();
            probe.set_on_error(Rc::new(move || Set::set(&status, ImageLoadingStatus::Error)));
        }
        configure_probe(probe.as_mut(), &ProbeConfig {
            referrer_policy: props.element_attributes
                .iter()
                .find(|(name, _)| name == "referrerPolicy")
                .map(|(_, value)| value.clone()),
            cross_origin: props.element_attributes
                .iter()
                .find(|(name, _)| name == "crossOrigin")
                .map(|(_, value)| value.clone()),
            sizes: props.sizes.clone(),
            src_set: props.src_set.clone(),
            src: props.src.clone(),
        });
        // The cached fast path (`:61-63`): resolve immediately, in the same
        // "layout effect" run — the fan-out below observes only the FINAL value,
        // which is exactly why a cached image never reports `'loading'` and a
        // cached error never reports `'idle'` (behavior.md *Events*).
        if probe.complete() {
            let status = if probe.natural_width() > 0 {
                ImageLoadingStatus::Loaded
            } else {
                ImageLoadingStatus::Error
            };
            Set::set(&image_status, status);
        }
        // The probe must outlive the body to fire its events (upstream's image is
        // kept alive by the effect closure until teardown) — forgotten per the
        // hold-or-forget convention; the late-event guard is the abandoned-signal
        // note above.
        std::mem::forget(probe);
    }

    // `isVisible` (`:53`) — read untracked for the body-time seeds; the reactive
    // reads below ride memos.
    let is_visible_memo = reactive_graph::computed::Memo::new(move |_| {
        Get::get(&image_status) == ImageLoadingStatus::Loaded
    });

    // -- Tier 2's transition machinery (`:54`) — the real hook inside its dedicated
    // rg-0.2 owner (the field bridge's `transition_status_signal` pattern), mirrored
    // to leptos for the views.
    let hook_owner = reactive_graph::owner::Owner::new();
    let (
        transition_status_rg,
        mounted_rg,
        status_mirror,
        mounted_mirror,
    ) = hook_owner.with(|| {
        let enable_idle_state = RgRwSignal::new_local(false);
        let defer_ending_state = RgRwSignal::new_local(false);
        let hook = use_transition_status(
            is_visible_memo.clone(),
            enable_idle_state,
            defer_ending_state,
            false, // `animateInitialOpen` (`:54`'s one-arg call)
        );
        let status_mirror: leptos::prelude::RwSignal<Option<TransitionStatus>> =
            RwSignal::new(GetUntracked::get_untracked(&hook.transition_status));
        let mounted_mirror: leptos::prelude::RwSignal<bool> =
            RwSignal::new(GetUntracked::get_untracked(&hook.mounted));
        // rg-0.2 → leptos mirrors (the field bridge's lockstep effects).
        {
            let status_mirror = status_mirror.clone();
            reactive_graph::effect::Effect::new(move |_| {
                let next = Get::get(&hook.transition_status);
                if status_mirror.get_untracked() != next {
                    status_mirror.set(next);
                }
            });
        }
        {
            let mounted_mirror = mounted_mirror.clone();
            reactive_graph::effect::Effect::new(move |_| {
                let next = Get::get(&hook.mounted);
                if mounted_mirror.get_untracked() != next {
                    mounted_mirror.set(next);
                }
            });
        }
        (
            hook.transition_status,
            hook.mounted,
            status_mirror,
            mounted_mirror,
        )
    });
    // The hook's owner outlives the subtree (the bridge's forgotten-owner
    // convention; its effects idle once the mirrors are abandoned).
    std::mem::forget(hook_owner);

    // The keepMounted element sync + the element-seam listeners run at ref-fire
    // time (the port's commit). This closure is the `useIsoLayoutEffect` on
    // `imageRef` (`:61-99`).
    let sync_from_element = {
        let image_ref = Rc::clone(&image_ref);
        let initial_commit = Rc::clone(&initial_commit);
        let image_status = image_status.clone();
        let mounted_rg = mounted_rg.clone();
        Rc::new(move || {
            // `if (!keepMounted) return` (`:62-64`) — checked by the caller.
            let is_initial = initial_commit.get();
            initial_commit.set(false);
            let image = image_ref.borrow().clone();
            let Some(image) = image else {
                // The dropped-ref bail (`:69-74`): the element's own load/error
                // events remain the only source of truth.
                return;
            };
            let Ok(image) = image.dyn_into::<web_sys::HtmlImageElement>() else {
                // A non-img render override exposes no load state — the same
                // not-complete path upstream's falsy `complete` takes.
                Set::set(&image_status, ImageLoadingStatus::Loading);
                return;
            };
            if !image.complete() {
                Set::set(&image_status, ImageLoadingStatus::Loading);
                return;
            }
            let status = if image.natural_width() > 0 {
                ImageLoadingStatus::Loaded
            } else {
                ImageLoadingStatus::Error
            };
            Set::set(&image_status, status);
            // The pre-seed (`:84-88`): an image complete on the first commit was
            // painted before hydration — mount it without going through
            // `'starting'` so the enter animation is not replayed.
            if status == ImageLoadingStatus::Loaded && is_initial {
                Set::set(&mounted_rg, true);
            }
        })
    };

    // The element seam: the ref-fork callback records the element and runs the
    // keepMounted sync + the load/error listener attachment (the engine bag has no
    // load/error slots — the module docs).
    let element_seam: RefCallback<Element> = {
        let image_ref = Rc::clone(&image_ref);
        let image_status = image_status.clone();
        let sync = Rc::clone(&sync_from_element);
        let keep_mounted = props.keep_mounted;
        let user_on_load = props.on_load.clone();
        let user_on_error = props.on_error.clone();
        Rc::new(move |instance: Option<&Element>| {
            match instance {
                Some(element) => {
                    *image_ref.borrow_mut() = Some(element.clone());
                    // The internal updates (`:111-116`) chained AFTER the user's
                    // handlers, cancellable by `prevent_base_ui_handler()` (the
                    // mergeProps.ts:221-274 contract).
                    let internal_load_status = image_status.clone();
                    let internal_error_status = image_status.clone();
                    let attach = |element: &Element,
                                  event_name: &str,
                                  user: &Option<UserImageEventHandler>,
                                  internal: Rc<dyn Fn()>| {
                        let user = user.clone();
                        let listener = wasm_bindgen::closure::Closure::wrap(
                            Box::new(move |event: web_sys::Event| {
                                let wrapped = BaseUIEvent::new(event);
                                if let Some(user) = &user {
                                    user(&wrapped);
                                }
                                if !wrapped.base_ui_handler_prevented() {
                                    internal();
                                }
                            }) as Box<dyn FnMut(web_sys::Event)>,
                        );
                        let _ = element
                            .add_event_listener_with_callback(
                                event_name,
                                listener.as_ref().unchecked_ref(),
                            );
                        // Held for the element's lifetime; the element is dropped
                        // (with its listeners) at the next rebuild.
                        std::mem::forget(listener);
                    };
                    if keep_mounted {
                        attach(element, "load", &user_on_load, Rc::new(move || {
                            Set::set(&internal_load_status, ImageLoadingStatus::Loaded);
                        }));
                        attach(element, "error", &user_on_error, Rc::new(move || {
                            Set::set(&internal_error_status, ImageLoadingStatus::Error);
                        }));
                        // The keepMounted sync (`:61-99`) — the commit analog.
                        sync();
                    }
                }
                None => {
                    *image_ref.borrow_mut() = None;
                }
            }
            None
        })
    };

    // -- The fan-out (`:120-129`): every status except `'idle'` → the user callback
    // then the root mirror, from one rg effect.
    {
        let on_change = props.on_loading_status_change.clone();
        let root_status = context.image_loading_status.clone();
        reactive_graph::effect::Effect::new(move |_| {
            let status = Get::get(&image_status);
            if status != ImageLoadingStatus::Idle {
                if let Some(callback) = &on_change {
                    callback(status);
                }
                Set::set(&root_status, status);
            }
        });
    }

    // -- The unmount reset (`:131-133`) — the root falls back to `'idle'` when the
    // image unmounts, which is what makes the fallback reappear (behavior.md *Edge
    // cases*). The context value is `Send` (the SendWrapper).
    {
        let root_status = context.image_loading_status.clone();
        let reset = SendWrapper::new(move || Set::set(&root_status, ImageLoadingStatus::Idle));
        reactive_graph::owner::on_cleanup(move || (*reset)());
    }

    // -- The exit-animation deferral (`:135-144`): `useOpenChangeComplete` scoped to
    // the closing direction, completing into `setMounted(false)`.
    {
        let enabled = reactive_graph::computed::Memo::new(move |_| !Get::get(&is_visible_memo));
        let on_complete: Rc<dyn Fn()> = {
            let image_status = image_status.clone();
            let mounted_rg = mounted_rg.clone();
            Rc::new(move || {
                // The `if (!isVisible)` guard (`:140-142`).
                if GetUntracked::get_untracked(&image_status) != ImageLoadingStatus::Loaded {
                    Set::set(&mounted_rg, false);
                }
            })
        };
        let reference = {
            let image_ref = Rc::clone(&image_ref);
            move || image_ref.borrow().clone()
        };
        let batch = RgRwSignal::new_local(false);
        hook_owner.with(|| {
            use_open_change_complete(UseOpenChangeCompleteParams {
                enabled,
                open: is_visible_memo.clone(),
                reference,
                batch,
                on_complete,
            });
        });
    }

    UseAvatarImage {
        root_status: context.image_loading_status.clone(),
        status_mirror,
        mounted_mirror,
        image_status,
        sync_from_element,
        element_seam,
        props_snapshot: AvatarImagePropsRuntime {
            keep_mounted: props.keep_mounted,
        },
    }
}

/// The runtime bits the view layer needs from the hook (the static per-body-run
/// props that shape the DOM).
pub struct AvatarImagePropsRuntime {
    /// `keepMounted` — the status-attribute scope and the presence override.
    pub keep_mounted: bool,
}

// A shim so the bag builders stay readable; converts into the engine's bag.
#[derive(Default)]
struct RenderElementPropsShim {
    attributes: Vec<(String, leptos_ui_internals::use_render_element::RenderAttributeFn)>,
    on_load: Option<Rc<dyn Fn(&BaseUIEvent<web_sys::Event>)>>,
    on_error: Option<Rc<dyn Fn(&BaseUIEvent<web_sys::Event>)>>,
}

impl RenderElementPropsShim {
    fn into_render_element_props(self) -> leptos_ui_internals::use_render_element::RenderElementProps {
        let mut props =
            leptos_ui_internals::use_render_element::RenderElementProps::default();
        props.handlers.attributes = self.attributes;
        // The engine bag has no load/error slots; the seam carries the user's
        // handlers (see the module docs). The shim slots exist so the bag-building
        // code mirrors upstream's object literals — they fold into nothing here.
        let _ = (self.on_load, self.on_error);
        props
    }
}

/// The dynamic image view: a container whose child is rebuilt from the tracked
/// mirrors (the rebuild-wrapper convention — the presence and the attributes are
/// current at every materialization; an empty container is upstream's `null`).
pub struct AvatarImageView {
    pub container: Element,
}
