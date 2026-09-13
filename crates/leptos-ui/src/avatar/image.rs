//! `AvatarImage` — `packages/react/src/avatar/image/AvatarImage.tsx` (the whole
//! component) plus `useImageLoadingStatus.ts` (the probe —
//! [`use_image_loading_status`], the Tier-1 machine) and
//! `AvatarImageDataAttributes.ts` (the attribute constants).
//!
//! See the module docs on `super` for the two-tier machine and the Rust
//! adaptations (rg-0.2 machinery + leptos-canonical mirrors, the injectable
//! probe factory, the dynamic-view presence). The keepMounted listeners ride
//! the engine's `on_load`/`on_error` slots (the materialization seam attaches
//! them as real listeners) — the merge contract (the user's handler runs
//! first, `prevent_base_ui_handler()` cancels the internal update,
//! mergeProps.ts:221-274) is the engine bag's own tested behavior.

use std::cell::RefCell;
use std::rc::Rc;

use reactive_graph::signal::RwSignal as RgRwSignal;
use reactive_graph::traits::{Get, GetUntracked, Set};
use send_wrapper::SendWrapper;
use wasm_bindgen::JsCast;

/// The machinery's local-storage signal (the field-bridge `new_local` typing —
/// single-threaded wasm machinery state).
pub type LocalRwSignal<T> = reactive_graph::signal::RwSignal<T, reactive_graph::owner::LocalStorage>;

use leptos_ui_internals::merge_props::PropsSource;
use leptos_ui_internals::state_attributes::{StateAttributeProps, transition_status_mapping};
use leptos_ui_internals::types::BaseUIEvent;
use leptos_ui_internals::use_open_change_complete::{
    UseOpenChangeCompleteParams, use_open_change_complete,
};
use leptos_ui_internals::use_render_element::{
    RenderElementProps, RenderedElement, UseRenderElementComponentProps, UseRenderElementParams,
    static_attr, use_render_element,
};
use leptos_ui_internals::use_transition_status::{TransitionStatus, use_transition_status};
use leptos_ui_utils::use_merged_refs::{InputRef, RefCallback};

use super::context::{AvatarRootContextValue, ImageLoadingStatus, use_avatar_root_context};
use super::root::avatar_state_attributes_mapping;

// ---------------------------------------------------------------------------
// The data attributes — AvatarImageDataAttributes.ts
// ---------------------------------------------------------------------------

/// `loading` (`AvatarImageDataAttributes.ts:6`) — present while the image is
/// loading (keepMounted mode only; default mode's element only exists once
/// loaded).
pub const LOADING_DATA_ATTR: &str = "data-loading";
/// `error` (`:10`) — present when the image failed to load (keepMounted only).
pub const ERROR_DATA_ATTR: &str = "data-error";
// `startingStyle`/`endingStyle` (`:14`, `:18`) re-export the internals'
// TransitionStatusDataAttributes — STARTING_STYLE/ENDING_STYLE in
// leptos_ui_internals::state_attributes; no separate constants here.

// ---------------------------------------------------------------------------
// The probe — useImageLoadingStatus's `new window.Image()`
// ---------------------------------------------------------------------------

/// The probe's request configuration (`UseImageLoadingStatusOptions` + `src`,
/// `useImageLoadingStatus.ts:7-12`): applied to the detached image in the
/// upstream assignment order — `referrerPolicy`, `crossOrigin ?? null`,
/// `sizes`, `srcset`, `src` (`:46-58`).
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
/// `onload`/`onerror` wiring (`:33-45`).
pub trait Probe {
    /// `image.complete` (`:61`).
    fn complete(&self) -> bool;
    /// `image.naturalWidth` (`:62`).
    fn natural_width(&self) -> u32;
    /// `image.onload = …` (`:44`).
    fn set_on_load(&mut self, callback: Rc<dyn Fn()>);
    /// `image.onerror = …` (`:45`).
    fn set_on_error(&mut self, callback: Rc<dyn Fn()>);
    /// The probe's image element, when the probe exposes one — the wasm
    /// suite's assertion/driver seam (fires `load`/`error` on it, asserts the
    /// request-config assignments). `None` = a probe with no element surface
    /// (its events are driven through its own test handles).
    fn as_image(&self) -> Option<web_sys::HtmlImageElement> {
        None
    }
}

/// The default factory's probe — a detached `document.createElement("img")`,
/// never inserted into the document: the real `new Image()` (`:33`; a detached
/// `HTMLImageElement` and `new Image()` are behaviorally equivalent for load
/// probing). Holds its listener closures; dropping it unsets the IDL
/// attributes — the upstream `isMounted` cleanup's late-event disconnection
/// (`:65-67`).
pub struct RealProbe {
    image: web_sys::HtmlImageElement,
    on_load: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
    on_error: Option<wasm_bindgen::closure::Closure<dyn FnMut()>>,
}

impl RealProbe {
    fn new(image: web_sys::HtmlImageElement) -> Self {
        Self {
            image,
            on_load: None,
            on_error: None,
        }
    }
}

impl Probe for RealProbe {
    fn complete(&self) -> bool {
        self.image.complete()
    }

    fn natural_width(&self) -> u32 {
        self.image.natural_width()
    }

    fn set_on_load(&mut self, callback: Rc<dyn Fn()>) {
        let closure: wasm_bindgen::closure::Closure<dyn FnMut()> =
            wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()));
        self.image
            .set_onload(Some(closure.as_ref().unchecked_ref()));
        self.on_load = Some(closure);
    }

    fn set_on_error(&mut self, callback: Rc<dyn Fn()>) {
        let closure: wasm_bindgen::closure::Closure<dyn FnMut()> =
            wasm_bindgen::closure::Closure::wrap(Box::new(move || callback()));
        self.image
            .set_onerror(Some(closure.as_ref().unchecked_ref()));
        self.on_error = Some(closure);
    }

    fn as_image(&self) -> Option<web_sys::HtmlImageElement> {
        Some(self.image.clone())
    }
}

/// The probe constructor — upstream's `new window.Image()` (`:33`) behind a
/// trait object so the deterministic fake of the upstream suite
/// (`AvatarImage.test.tsx:27-73`'s `window.Image` stub: `completeOnSet`,
/// `naturalWidth`) can be injected in tests via [`with_probe_factory`].
pub type ProbeFactory = Rc<dyn Fn() -> Box<dyn Probe>>;

thread_local! {
    /// The probe-factory override slot. Scoped injection; the production path
    /// never takes the override arm.
    static PROBE_FACTORY_OVERRIDE: RefCell<Option<ProbeFactory>> = const { RefCell::new(None) };
}

/// Runs `body` with `factory` as the probe constructor — the deterministic
/// probe of the wasm suite. Scoped: nested calls restore the previous factory.
pub fn with_probe_factory<R>(factory: ProbeFactory, body: impl FnOnce() -> R) -> R {
    PROBE_FACTORY_OVERRIDE.with(|slot| {
        let previous = slot.borrow_mut().replace(Rc::clone(&factory));
        let result = body();
        *slot.borrow_mut() = previous;
        result
    })
}

fn construct_probe() -> Box<dyn Probe> {
    PROBE_FACTORY_OVERRIDE.with(|slot| match &*slot.borrow() {
        Some(factory) => factory(),
        None => Box::new(RealProbe::new(
            web_sys::window()
                .expect("no window")
                .document()
                .expect("no document")
                .create_element("img")
                .expect("create img")
                .dyn_into()
                .expect("the img is an HtmlImageElement"),
        )),
    })
}

/// The config assignment onto the probe (`:46-58`), verbatim order and guards —
/// including the unconditional `crossOrigin` assignment (`:49`,
/// `crossOrigin ?? null`, implementation.md untested item 6). An override probe
/// without an image surface receives no assignments (its config rides its own
/// test handles).
fn configure_probe(probe: &dyn Probe, config: &ProbeConfig) {
    let Some(image) = probe.as_image() else {
        return;
    };
    // `if (referrerPolicy)` (`:46`).
    if let Some(policy) = &config.referrer_policy {
        image.set_referrer_policy(policy.as_str());
    }
    // `image.crossOrigin = crossOrigin ?? null` — unconditional (`:49`).
    image.set_cross_origin(config.cross_origin.as_deref());
    // `if (sizes)` (`:50`).
    if let Some(sizes) = &config.sizes {
        image.set_sizes(sizes);
    }
    // `if (srcSet)` — the IDL attribute is `srcset` (`:53`).
    if let Some(src_set) = &config.src_set {
        image.set_srcset(src_set);
    }
    // `if (src)` (`:56`).
    if let Some(src) = &config.src {
        image.set_src(src);
    }
}

/// The live probes of one image instance. Upstream's probe lives in the effect
/// closure's scope from construction until the effect's next run or teardown
/// (`:33` — the closure keeps `image` alive so async loads can fire; the
/// cleanup's `isMounted = false` at `:65-67` only stops the *writes*). The
/// port reproduces both halves exactly: the slot **drains at each effect
/// run** (the previous probe drops → its closures drop → the IDL attributes
/// unset → late events from the previous source are disconnected) and **at
/// disposal** (the component unmount). A drained-and-empty slot after unmount
/// is upstream's `isMounted === false`.
#[derive(Default)]
pub(crate) struct ProbeSlot {
    probes: RefCell<Vec<Box<dyn Probe>>>,
}

impl ProbeSlot {
    fn push(&self, probe: Box<dyn Probe>) {
        self.probes.borrow_mut().push(probe);
    }

    /// The teardown (`:65-67`): drop every live probe, disconnecting the
    /// listeners.
    fn drain(&self) {
        self.probes.borrow_mut().clear();
    }
}

// ---------------------------------------------------------------------------
// Tier 1 — the image-local status machine (useImageLoadingStatus.ts:14-71)
// ---------------------------------------------------------------------------

/// The upstream dependency-array tuple (`:68`) as a stable string key — the
/// reactive source the scheduling effect tracks. `\u{1}` joins (a control
/// character no realistic config contains, so the split is lossless); the
/// keepMounted arm is a dedicated sentinel so an empty config can never
/// collide with it.
fn source_config_key(
    src: Option<&str>,
    src_set: Option<&str>,
    sizes: Option<&str>,
    referrer_policy: Option<&str>,
    cross_origin: Option<&str>,
    enabled: bool,
) -> String {
    if !enabled {
        return "\u{1}\u{1}keepMounted".to_string();
    }
    let part = |value: Option<&str>| value.unwrap_or("");
    format!(
        "{}\u{1}{}\u{1}{}\u{1}{}\u{1}{}",
        part(src),
        part(src_set),
        part(sizes),
        part(referrer_policy),
        part(cross_origin),
    )
}

const KEEP_MOUNTED_KEY: &str = "\u{1}\u{1}keepMounted";

/// `useImageLoadingStatus(src, { referrerPolicy, crossOrigin, sizes, srcSet },
/// enabled)` (`useImageLoadingStatus.ts:14-71`) — the probe-scheduling effect
/// over the image-local status signal. The rg-0.2 effect stands in for the
/// layout effect; **the tracked read is the upstream dependency array**
/// (`:68`: `enabled, src, srcSet, sizes, crossOrigin, referrerPolicy`) — a
/// change re-runs the effect body, which first drains the slot (tearing down
/// the previous probe) and then probes afresh: the reset-on-source-change
/// transition falls out exactly as upstream's effect restart does. A runtime
/// prop change is the caller's rebuild (the static-prop adaptation), and a
/// rebuilt body re-runs the hook, re-seeding the config signal — the restart.
///
/// Must be called inside a reactive owner (the `useIsoLayoutEffect` inside a
/// component upstream). Returns the status signal (upstream's `state` tuple
/// narrowed to the value arm — every write site passes a literal status).
pub fn use_image_loading_status(
    src: Option<String>,
    referrer_policy: Option<String>,
    cross_origin: Option<String>,
    sizes: Option<String>,
    src_set: Option<String>,
    enabled: bool,
    slot: Rc<ProbeSlot>,
) -> RgRwSignal<ImageLoadingStatus> {
    let status = RgRwSignal::new_local(ImageLoadingStatus::Idle);

    // The source-config signal — the dep-array analog, read tracked.
    let source_config = RgRwSignal::new_local(source_config_key(
        src.as_deref(),
        src_set.as_deref(),
        sizes.as_deref(),
        referrer_policy.as_deref(),
        cross_origin.as_deref(),
        enabled,
    ));

    // The scheduling effect — `useIsoLayoutEffect(() => {…}, [enabled, src,
    // srcSet, sizes, crossOrigin, referrerPolicy])` (`:22-68`).
    reactive_graph::effect::Effect::new({
        let status = status.clone();
        move |_| {
            let key = Get::get(&source_config);

            // `if (!enabled) return NOOP` (`:22-25`) — keepMounted bails
            // before touching `window.Image`; no probe is ever constructed.
            if key == KEEP_MOUNTED_KEY {
                return;
            }

            // The teardown of the previous run's probe (`:65-67`'s
            // isMounted flip, made ownership-exact — see ProbeSlot).
            slot.drain();

            let mut parts = key.split('\u{1}');
            let src = parts.next().filter(|part| !part.is_empty());
            let src_set = parts.next().filter(|part| !part.is_empty());
            let sizes = parts.next().filter(|part| !part.is_empty());
            let referrer_policy = parts.next().filter(|part| !part.is_empty());
            let cross_origin = parts.next().filter(|part| !part.is_empty());

            // `if (!src && !srcSet) { setLoadingStatus('error'); return NOOP;
            // }` (`:27-30`) — short-circuits to `'error'` without
            // constructing a probe.
            if src.is_none() && src_set.is_none() {
                Set::set(&status, ImageLoadingStatus::Error);
                return;
            }

            // The probe (`:33-63`).
            let mut probe = construct_probe();
            // `'loading'` set synchronously (`:43`).
            Set::set(&status, ImageLoadingStatus::Loading);
            {
                // The `isMounted`-guarded updater (`:35-41`) — guarded by the
                // closure handle's lifetime (the slot's drain disconnects).
                let status = status.clone();
                probe.set_on_load(Rc::new(move || {
                    Set::set(&status, ImageLoadingStatus::Loaded);
                }));
            }
            {
                let status = status.clone();
                probe.set_on_error(Rc::new(move || {
                    Set::set(&status, ImageLoadingStatus::Error);
                }));
            }
            configure_probe(
                probe.as_ref(),
                &ProbeConfig {
                    referrer_policy: referrer_policy.map(str::to_string),
                    cross_origin: cross_origin.map(str::to_string),
                    sizes: sizes.map(str::to_string),
                    src_set: src_set.map(str::to_string),
                    src: src.map(str::to_string),
                },
            );
            // The cached fast path (`:61-63`): resolve immediately from
            // `complete`/`naturalWidth` — a cached image never reports
            // `'loading'` and a cached error never reports `'idle'`
            // (behavior.md *Events*).
            if probe.complete() {
                let next = if probe.natural_width() > 0 {
                    ImageLoadingStatus::Loaded
                } else {
                    ImageLoadingStatus::Error
                };
                Set::set(&status, next);
            }
            // Held in the slot until the next run / disposal — the closure
            // scope that keeps `image` alive for async loads (`:33`).
            slot.push(probe);
        }
    });

    status
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
    /// The `...elementProps` rest — the MIDDLE bag: overrides the internal
    /// status attributes (an explicit `aria-hidden` survives, `:110`'s merge
    /// contract) but loses to `sourceProps` on `src`/`sizes`/`srcSet` keys.
    pub element_attributes: Vec<(String, String)>,
    /// The user's `onLoad` (`:111-113` chaining) — runs before the internal
    /// update; `prevent_base_ui_handler()` cancels it.
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
    pub ref_callback: Option<RefCallback<web_sys::Element>>,
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

/// The resolved state the element builder consumes — the per-rebuild snapshot
/// of `AvatarImageState` (`AvatarImage.tsx:146-151`) plus the two flags the
/// DOM shape depends on.
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

/// `shouldRender` (`:153`): `keepMounted || mounted`.
pub fn should_render(snapshot: &AvatarImageStateSnapshot) -> bool {
    snapshot.keep_mounted || snapshot.mounted
}

/// The combined `stateAttributesMapping` (`AvatarImage.tsx:16-19`): the root's
/// `imageLoadingStatus → null` suppression plus `transitionStatusMapping`.
/// Exported for the host-suite mapping pins.
pub fn avatar_image_state_attributes_mapping<'a>(
) -> impl Fn(&str, &serde_json::Value) -> Option<Option<StateAttributeProps>> + 'a {
    move |key: &str, value: &serde_json::Value| {
        if key == "imageLoadingStatus" {
            // The suppression (`stateAttributesMapping.ts:2`).
            return Some(None);
        }
        // `...transitionStatusMapping` (`:18`) — declines non-transition keys
        // with `None`, emits the style hooks during a transition, `Some(None)`
        // (nothing) otherwise.
        transition_status_mapping(key, value)
    }
}

/// The keepMounted status-attribute bag (`renderedStatusProps`,
/// `AvatarImage.tsx:101-118`) — the three conditional attributes. `None`
/// values are the upstream `undefined` (the attribute omitted).
fn rendered_status_attributes(status: ImageLoadingStatus) -> Vec<(&'static str, Option<String>)> {
    vec![
        // `data-loading: status === 'loading' ? '' : undefined` (`:106`).
        (
            LOADING_DATA_ATTR,
            (status == ImageLoadingStatus::Loading).then(|| String::new()),
        ),
        // `data-error: status === 'error' ? '' : undefined` (`:107`).
        (
            ERROR_DATA_ATTR,
            (status == ImageLoadingStatus::Error).then(|| String::new()),
        ),
        // `aria-hidden: status !== 'loaded' || undefined` (`:110`) — React
        // renders the boolean `true` as the string `"true"`.
        (
            "aria-hidden",
            (status != ImageLoadingStatus::Loaded).then(|| "true".to_string()),
        ),
    ]
}

/// Builds the `<img>` element description — upstream's `useRenderElement('img',
/// …)` call (`AvatarImage.tsx:166-172`) as a pure function of the state
/// snapshot, for the host suite and the wasm rebuild loop alike. `None` is
/// upstream's `!shouldRender → null` (`:174-176`). `extra_refs` ride the ref
/// fork after the forwarded ref (upstream's `imageRef` slot — the seam).
pub fn avatar_image_element(
    snapshot: AvatarImageStateSnapshot,
    props: &AvatarImageProps,
    extra_refs: Vec<InputRef<web_sys::Element>>,
) -> Option<RenderedElement> {
    if !should_render(&snapshot) {
        return None;
    }

    // The state record (`:146-151`) — the suppression mapping declines the
    // first member; the transition mapping emits the style hooks.
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

    // Bag 1 — `renderedStatusProps` (`:101-118`), keepMounted-scoped (the
    // upstream `undefined` spread contributes nothing otherwise).
    let mut status_bag = RenderElementProps::default();
    if snapshot.keep_mounted {
        for (name, value) in rendered_status_attributes(snapshot.image_loading_status) {
            if let Some(value) = value {
                status_bag
                    .handlers
                    .attributes
                    .push((name.to_string(), static_attr(value)));
            }
        }
    }

    // Bag 2 — `elementProps` (`:43` rest): user attributes + the user's
    // load/error handlers chained FIRST (mergeProps' handler order — the
    // user's handler runs before the internal update and can prevent it).
    let mut element_bag = RenderElementProps::default();
    for (name, value) in &props.element_attributes {
        element_bag
            .handlers
            .attributes
            .push((name.clone(), static_attr(value.clone())));
    }
    if let Some(on_load) = props.on_load.clone() {
        element_bag.handlers.on_load = Some(Rc::new(move |event: &BaseUIEvent<web_sys::Event>| {
            on_load(event)
        }));
    }
    if let Some(on_error) = props.on_error.clone() {
        element_bag.handlers.on_error =
            Some(Rc::new(move |event: &BaseUIEvent<web_sys::Event>| on_error(event)));
    }

    // Bag 3 — `sourceProps` (`:154-164`): the guarded conditional construction,
    // in `sizes` → `srcSet` → `src` order (the fetch-order shim, `:37-44`).
    let mut source_bag = RenderElementProps::default();
    if let Some(sizes) = &props.sizes {
        source_bag
            .handlers
            .attributes
            .push(("sizes".to_string(), static_attr(sizes.clone())));
    }
    if let Some(src_set) = &props.src_set {
        source_bag
            .handlers
            .attributes
            .push(("srcset".to_string(), static_attr(src_set.clone())));
    }
    if let Some(src) = &props.src {
        source_bag
            .handlers
            .attributes
            .push(("src".to_string(), static_attr(src.clone())));
    }

    let props_bags = vec![
        PropsSource::Static(status_bag),
        PropsSource::Static(element_bag),
        PropsSource::Static(source_bag),
    ];

    // The ref fork inputs (`:168`'s `[forwardedRef, imageRef]`).
    let mut refs: Vec<InputRef<web_sys::Element>> = Vec::new();
    if let Some(callback) = &props.ref_callback {
        refs.push(InputRef::Callback(Rc::clone(callback)));
    }
    refs.extend(extra_refs);

    let mapping = avatar_image_state_attributes_mapping();
    use_render_element(
        "img",
        // The component props ride through: className/style resolve against
        // the state; `render` replaces/merges wholesale.
        UseRenderElementComponentProps {
            class_name: props.class_style.class_name.clone(),
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
// The live hook — the full AvatarImage body
// ---------------------------------------------------------------------------

/// The `AvatarImage` handle: the leptos mirrors the caller's tree tracks.
pub struct UseAvatarImage {
    /// The root status (the context signal — the fallback gate reads it).
    pub root_status: leptos::prelude::RwSignal<ImageLoadingStatus>,
    /// The image-local status (Tier 1's canonical signal, machinery side).
    pub image_status: RgRwSignal<ImageLoadingStatus>,
    /// The image-local status mirror (leptos) — the views' tracked read.
    pub image_status_mirror: leptos::prelude::RwSignal<ImageLoadingStatus>,
    /// The masked transition-status mirror (leptos).
    pub status_mirror: leptos::prelude::RwSignal<Option<TransitionStatus>>,
    /// The mounted mirror (leptos) — the presence gate.
    pub mounted_mirror: leptos::prelude::RwSignal<bool>,
    /// `keepMounted` — the status-attribute scope and the presence override.
    pub keep_mounted: bool,
    /// The keepMounted element sync (`:61-99`), exposed for the ref seam to
    /// invoke (and for tests to drive directly).
    pub sync_from_element: Rc<dyn Fn()>,
    /// The ref-fork seam (`:168`'s `imageRef` slot — records the element and
    /// runs the keepMounted sync at ref-fire time). Rides the caller's ref
    /// fork next to the forwarded ref.
    pub element_seam: RefCallback<web_sys::Element>,
}

/// `AvatarImage` (`AvatarImage.tsx:27-179`) — the full body. Must be called
/// inside a reactive owner (a component body). The probe needs the DOM, so the
/// probe-scheduling effect only probes on wasm (the host target still wires
/// the machine; the dual-target suite convention — the machine contracts are
/// pinned through the pure builders and the wasm suite).
pub fn use_avatar_image(props: &AvatarImageProps) -> UseAvatarImage {
    // The root context (`:46`) — the writer side.
    let context: AvatarRootContextValue = use_avatar_root_context();

    // The keepMounted element ref (`:56`'s `imageRef`) and the first-commit
    // marker (`:57`).
    let image_ref: Rc<RefCell<Option<web_sys::Element>>> = Rc::new(RefCell::new(None));
    let initial_commit = Rc::new(std::cell::Cell::new(true));

    // The live probes of this image instance (see ProbeSlot).
    let probe_slot = Rc::new(ProbeSlot::default());

    // All rg-0.2 machinery lives in a dedicated owner (the field bridge's
    // `transition_status_signal` pattern): the machinery's runtime differs
    // from the views', so its effects must be scoped to an rg owner, not to
    // the ambient leptos one.
    let hook_owner = reactive_graph::owner::Owner::new();

    let (image_status, image_status_mirror, transition_status_rg, mounted_rg, status_mirror, mounted_mirror) =
        hook_owner.with(|| {
            // Tier 1 — the image-local status (`:47-51`'s
            // `useImageLoadingStatus`), the canonical signal.
            let image_status = use_image_loading_status(
                props.src.clone(),
                props
                    .element_attributes
                    .iter()
                    .find(|(name, _)| name == "referrerPolicy")
                    .map(|(_, value)| value.clone()),
                props
                    .element_attributes
                    .iter()
                    .find(|(name, _)| name == "crossOrigin")
                    .map(|(_, value)| value.clone()),
                props.sizes.clone(),
                props.src_set.clone(),
                !props.keep_mounted,
                Rc::clone(&probe_slot),
            );

            // `isVisible` (`:53`) — the transition machinery's open input.
            let is_visible = reactive_graph::computed::Memo::new(move |_| {
                Get::get(&image_status) == ImageLoadingStatus::Loaded
            });

            // -- Tier 2's transition machinery (`:54`) — the real hook,
            // mirrored to leptos for the views.
            let hook = use_transition_status(
                is_visible.clone(),
                RgRwSignal::new_local(false), // `enableIdleState` — the one-arg call (`:54`)
                RgRwSignal::new_local(false), // `deferEndingState`
                false,                        // `animateInitialOpen`
            );
            let status_mirror: leptos::prelude::RwSignal<Option<TransitionStatus>> =
                leptos::prelude::RwSignal::new(GetUntracked::get_untracked(
                    &hook.transition_status,
                ));
            let mounted_mirror: leptos::prelude::RwSignal<bool> =
                leptos::prelude::RwSignal::new(GetUntracked::get_untracked(&hook.mounted));
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
            // The image-local status mirror — the views' tracked read of Tier
            // 1 (an rg signal cannot be tracked from a leptos view closure).
            let image_status_mirror: leptos::prelude::RwSignal<ImageLoadingStatus> =
                leptos::prelude::RwSignal::new(GetUntracked::get_untracked(&image_status));
            {
                let image_status_mirror = image_status_mirror.clone();
                reactive_graph::effect::Effect::new(move |_| {
                    let next = Get::get(&image_status);
                    if image_status_mirror.get_untracked() != next {
                        image_status_mirror.set(next);
                    }
                });
            }

            // -- The exit-animation deferral (`:135-144`):
            // `useOpenChangeComplete` scoped to the closing direction,
            // completing into `setMounted(false)`.
            {
                let enabled = reactive_graph::computed::Memo::new(move |_| !Get::get(&is_visible));
                let on_complete: Rc<dyn Fn()> = {
                    let image_status = image_status.clone();
                    let mounted_rg = hook.mounted.clone();
                    Rc::new(move || {
                        // The `if (!isVisible)` guard (`:140-142`).
                        if GetUntracked::get_untracked(&image_status)
                            != ImageLoadingStatus::Loaded
                        {
                            Set::set(&mounted_rg, false);
                        }
                    })
                };
                let reference = {
                    let image_ref = Rc::clone(&image_ref);
                    move || image_ref.borrow().clone()
                };
                use_open_change_complete(UseOpenChangeCompleteParams {
                    enabled,
                    open: is_visible.clone(),
                    reference,
                    batch: RgRwSignal::new_local(false),
                    on_complete,
                });
            }

    (
        image_status,
        image_status_mirror,
        hook.transition_status,
        hook.mounted,
        status_mirror,
        mounted_mirror,
    )
        });
    // The hook's owner outlives the subtree (the bridge's forgotten-owner
    // convention; its effects idle once the mirrors are abandoned).
    std::mem::forget(hook_owner);

    // The keepMounted element sync (`:61-99`'s `useIsoLayoutEffect` on
    // `imageRef`), run at ref-fire time — the port's commit analog.
    let sync_from_element: Rc<dyn Fn()> = {
        let image_ref = Rc::clone(&image_ref);
        let initial_commit = Rc::clone(&initial_commit);
        let image_status = image_status.clone();
        let mounted_rg = mounted_rg.clone();
        Rc::new(move || {
            // `const isInitialCommit = initialCommitRef.current` (`:66-67`).
            let is_initial = initial_commit.get();
            initial_commit.set(false);
            // `const image = imageRef.current; if (!image) return` (`:69-74`)
            // — the dropped-ref bail: the element's own load/error events
            // remain the only source of truth; nothing is overwritten.
            let image = image_ref.borrow().clone();
            let Some(image) = image else {
                return;
            };
            let Ok(image) = image.dyn_into::<web_sys::HtmlImageElement>() else {
                // A non-img render override exposes no load state — the same
                // not-complete path upstream's falsy `complete` takes.
                Set::set(&image_status, ImageLoadingStatus::Loading);
                return;
            };
            // `if (!image.complete) { setLoadingStatus('loading'); return; }`
            // (`:76-78`).
            if !image.complete() {
                Set::set(&image_status, ImageLoadingStatus::Loading);
                return;
            }
            // `complete` → resolved from `naturalWidth` (`:79-82`).
            let status = if image.natural_width() > 0 {
                ImageLoadingStatus::Loaded
            } else {
                ImageLoadingStatus::Error
            };
            Set::set(&image_status, status);
            // The pre-seed (`:84-88`): an image complete on the first commit
            // was painted before hydration — mount it without going through
            // `'starting'` so the enter animation is not replayed.
            if status == ImageLoadingStatus::Loaded && is_initial {
                Set::set(&mounted_rg, true);
            }
        })
    };

    // The element seam: the ref-fork callback records the element and runs the
    // keepMounted sync at ref-fire time. `onLoad`/`onError` ride the engine's
    // handler slots (the module docs) — the seam only mirrors upstream's ref
    // bookkeeping.
    let element_seam: RefCallback<web_sys::Element> = {
        let image_ref = Rc::clone(&image_ref);
        let sync = Rc::clone(&sync_from_element);
        let keep_mounted = props.keep_mounted;
        Rc::new(move |instance: Option<&web_sys::Element>| {
            match instance {
                Some(element) => {
                    *image_ref.borrow_mut() = Some(element.clone());
                    if keep_mounted {
                        // The `useIsoLayoutEffect` on `imageRef` (`:61-99`).
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

    // -- The fan-out (`:120-129`): every status except `'idle'` → the user
    // callback then the root mirror, from one rg effect (inside the hook
    // owner's scope — but the owner was already forgotten above; the effect
    // creation needs the scope, so this block runs under a fresh `with`).
    // Wait — the forgotten owner can still be `with`ed (the leak only skips
    // its Drop); the effects join its tree and idle forever after.
    hook_owner.with(|| {
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

        // -- The unmount reset (`:131-133`) — the root falls back to `'idle'`
        // when the image unmounts, which is what makes the fallback reappear
        // (behavior.md *Edge cases*). **Leptos-side** `on_cleanup` — the
        // context signal is leptos-runtime; an rg-0.2 `on_cleanup` under the
        // leptos mount owner would never fire (the dual-runtime law; the
        // direction-provider page's documented Owner::set hazard). The
        // closure crosses as `SendWrapper` (the dialog trigger's cleanup
        // convention). Also the probe slot's final drain (the isMounted
        // teardown at unmount).
        {
            let root_status = context.image_loading_status.clone();
            let probe_slot = Rc::clone(&probe_slot);
            let reset =
                SendWrapper::new(move || {
                    probe_slot.drain();
                    Set::set(&root_status, ImageLoadingStatus::Idle);
                });
            leptos::prelude::on_cleanup(move || (*reset)());
        }
    });

    UseAvatarImage {
        root_status: context.image_loading_status.clone(),
        image_status,
        image_status_mirror,
        status_mirror,
        mounted_mirror,
        keep_mounted: props.keep_mounted,
        sync_from_element,
        element_seam,
    }
}
