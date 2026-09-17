## Spec Citation Discrepancies - library: form

### implementation.md citation issues:
- **Line 56**: Claims `TODO.md:407-413` has no `wraps-external:` field for the form component
  - **Actual content at TODO.md:407-413**: Avatar component (done) with exemption-from-docs-pairing note and extensive commit notes
  - **Correct location**: Form component is at TODO.md:502-508, which indeed has no `wraps-external:` field
  - **Impact**: The citation is pointing to the wrong line numbers but the actual claim (no `wraps-external:` field) is correct for the form item

### Resolution:
The citation contains correct line numbers but points to the wrong TODO.md entry. The form component at TODO.md:502-508 correctly has no `wraps-external:` field, so the spec's claim is factually correct even though the citation reference is stale. The implementation can proceed.

**Date**: 2026-09-13  
**Item**: library: form

## Spec Citation Discrepancies - library: drawer

### behavior.md citation issues:
- **Line 393**: Claims `TODO.md:393-393` says `library: drawer` is flagged `needs-batched-mining: true`
  - **Actual content at TODO.md:393**: `- [ ] library: autocomplete`
  - **Correct location**: `needs-batched-mining: true` is at TODO.md:484 for the drawer item
  - **Impact**: The citation hash doesn't match current content

### implementation.md citation issues:
- **Lines 385-392**: Claims these contain `wraps-external:` field for drawer
  - **Actual content at TODO.md:385-392**: Alert-dialog item (done) with notes about citation drift
  - **Correct location**: Drawer item is at TODO.md:477-484, no `wraps-external:` field present
  - **Impact**: The citation hash doesn't match current content

- **Line 388**: Claims this contains precise per-unit requirements
  - **Actual content at TODO.md:388**: Note about alert-dialog citation drift resolution
  - **Correct location**: No specific per-unit requirements found at this line
  - **Impact**: The citation hash doesn't match current content

### Resolution:
These appear to be stale citations from previous iterations that weren't updated when TODO.md content changed. The drawer item at TODO.md:477-484 is correctly structured and the implementation should proceed based on the actual TODO entry, not the stale citations.

**Date**: 2026-09-13
**Item**: library: drawer
# Appended by the `library: field` part-surface iteration.

## run-regression.sh's docs-rendering gate cannot see Phase D items

- **Where**: `ralph/scripts/run-regression.sh:51-52` — step 4 keys off the item's own
  `docs-pair:` field (`get-todo-field.mjs "$TODO_ID" docs-pair`).
- **Problem**: Phase D items (`docs-content: components/*`) carry `owner: library: X` and
  **no** `docs-pair:` field (see `TODO.md`'s Phase D entries and
  `ralph/scripts/check-todo-schema.mjs`, whose pairing rule is oriented the other way:
  the *library* item points at the docs item). So for exactly the items whose `done-when`
  IS "docs-app renders <page>.mdx with all its demos using crates/leptos-ui's real
  component", the gate skips the `cargo leptos build` check entirely and passes on the
  crate tests + TODO schema alone.
- **Impact**: a Phase D item can be marked `done` with the docs-app build broken or the
  route missing. The Stage 3 prompt's own step 6 covers the gap ("it has a `docs-pair:`
  field, **or its `done-when` mentions docs-app**") — the runner does not implement the
  second half of that disjunction.
- **Not fixed here**: this iteration's item (`library: field`) does carry `docs-pair:`, so
  the gate ran the docs-app build; changing `run-regression.sh` is loop-tooling scope, not
  the item's crate. Recording it so the audit loop can align the runner with the prompt.

**Date**: 2026-09-15
**Item**: library: field (observed while assessing docs-content: components/field)

# Appended by the `library: form` iteration.

## `specs/library/form/implementation.md`'s TODO.md line citation has drifted out of range

- **Where**: `specs/library/form/implementation.md:56` cites "The `library: form` entry in
  `TODO.md:502-508` has no `wraps-external:` field"; the recorded baseline key is
  `TODO.md:407-413` (`specs/library/form/implementation.citations.json:52`).
- **Problem**: the assertion itself still holds — the `library: form` entry carries no
  `wraps-external:` field — but the entry has moved far from both cited ranges: it now
  lives at `TODO.md:619-643` (the file grew as entry notes accumulated above it).
- **Impact**: none for this iteration's work: the citation checker's shift tolerance
  reports it rather than blocking, and neither cited range (`407-413`, `502-508`) falls
  inside the window this iteration's `blocked-by`/`note` edit touched, so no baseline was
  invalidated. Recording it so the audit loop can re-anchor the range to the entry's true
  position (the `context-menu`/`accordion` re-anchoring precedent) instead of this
  iteration rewriting a spec citation — the specs are not mine to silently "fix".

**Date**: 2026-09-15
**Item**: library: form (citation check by inspection, not by failure)

# Appended by the `docs-content: components/field` iteration.

## The port's `Field.Error` stays MOUNTED (marked `hidden`) while upstream renders `null`

- **Where**: `crates/leptos-ui/src/field/field_parts.rs:600-668` (`field_error_view`) vs
  `packages/react/src/field/error/FieldError.tsx:128-134`.
- **Observed** (real browser, mounting the docs page's hero demo — the pristine, untouched
  state, `crates/docs-app/src/render_test.rs::field_hero_demo_renders_the_real_part_composition`):
  the rendered tree contains
  `<div class="text-sm text-red-700 dark:text-red-400" id="base-ui-4" hidden="">Please enter your name</div>`
  alongside the label/control/description. Upstream returns `null` when `!mounted`
  (`FieldError.tsx:130-134`), and `useTransitionStatus(rendered)` seeds `mounted` from
  `rendered` — so the React DOM has NO error element at all before validation surfaces.
  The port instead always renders the `<div>`, gating visibility on
  `hidden=move || (!status.mounted()).then(|| "".to_string())` (`field_parts.rs:625`) and
  rendering the user's `children` OUTSIDE the mounted gate (`:667`; only the derived-message
  arm is gated, `:627-666`).
- **Impact**: a pristine field's `innerHTML` carries the error message text where upstream's
  does not (visually identical — `hidden` — but not DOM-identical, which is what the docs-page
  differential tests compare). Related: the message-id registration is a body-time
  unconditional push (`field_parts.rs:505`) while upstream registers inside a
  `rendered`-gated layout effect with a clear-on-unrender teardown
  (`FieldError.tsx:64-78`) — whether that reaches `aria-describedby` on the control (the
  pristine DOM shows the control with `aria-labelledby` only, no `aria-describedby` at all)
  needs its own verification; recorded, not claimed.
- **Not fixed here**: this iteration's item is `docs-content: components/field` (crate
  `docs-app`); `crates/leptos-ui` belongs to the `library: field` item, whose own suite is
  green at the done-marked tree. The docs page pins the port's ACTUAL contract (mounted +
  `hidden`) and cross-references this entry, rather than asserting an upstream DOM shape the
  port does not produce and calling the page verified. Recording it so the audit loop (or a
  `library: field` reopen) can decide whether the unmount arm is a real parity gap.

**Date**: 2026-09-15
**Item**: docs-content: components/field (observed while porting the page's hero demo)

## `specs/docs-content/field/demos.json` over-claims the hero demo's error trigger

- **Where**: `specs/docs-content/field/demos.json`, the hero entry's
  `nonTrivialInteractions[1]`: "Field.Error renders only when the native
  valueMissing constraint matches (empty required input), surfacing once validation is
  triggered per Field.Root's default onBlur validation mode".
- **Problem (two claims, both contradicted by upstream source)**: (1) Field.Root's
  default validation mode is **`'onSubmit'`**, not `onBlur`
  (`packages/react/src/internals/form-context/FormContext.ts:42`,
  `packages/react/src/internals/field-root-context/FieldRootContext.ts:48`). (2) Nothing in
  the demo triggers validation at all: the demo (`demos/hero/tailwind/index.tsx:1-21`) has no
  `<Form>` and no submit control, and with `validationMode === 'onSubmit'` +
  `submitCountRef.current === 0` the change path takes the revalidate early-return
  (`packages/react/src/field/root/useFieldValidation.ts:244-246` — `if (revalidate) { if
  (state.valid !== false || !element) return; … }`, and the pristine state's `valid` is
  `null`, never `false`). Upstream's OWN FieldError tests drive the error through a `<Form>`
  + `<button type="submit">`: `FieldError.test.tsx:32-53` asserts the message is absent after
  focus + two `change`s + blur, and only appears after the submit click; the `match="valueMissing"`
  test (`:56-79`) shows the same submit-first ordering, then a change to `''` keeps it visible
  *because the submit already committed `state.valid === false`*.
- **Impact**: a docs-app iteration that trusts this line will pin the wrong behavior — the
  field page's wasm tests first asserted "blur surfaces the error" (failed: blur only commits
  under `onBlur` mode, `field_control.rs:403`) and then "type-then-clear surfaces the error"
  (also failed: the revalidate early-return above). The page now pins the upstream-true
  behavior — the slot stays hidden in this demo — and cites this entry.
- **Not fixed here**: `specs/**` is not the iteration's to rewrite. The demo JSON's prose is a
  Stage-2 mining over-claim about a trigger; the demo source and the port agree.

**Date**: 2026-09-15
**Item**: docs-content: components/field (observed while pinning the hero demo's behavior)

## behavior.md's Enter `defaultPrevented` observable is a React-synthetic artifact

- **Where**: `specs/library/checkbox/behavior.md` → "Keyboard interactions": "at the time
  the ancestor handler runs, `event.defaultPrevented` is still `false`, proving Root does not
  itself `preventDefault()` on Enter (`packages/react/src/checkbox/root/CheckboxRoot.test.tsx:849-852`, `:871`)".
- **Problem**: the claim holds upstream but is a property of **React's synthetic event**, not
  of the DOM. `packages/react/src/checkbox/root/CheckboxRoot.tsx:354` calls
  `originalNativePreventDefault.call(nativeEvent)` — the NATIVE event IS cancelled, and that
  cancellation is precisely how native button activation is suppressed. An ancestor handler
  observes `false` only because React stamps the native event's members onto a fresh synthetic
  object when that object is created (before Root's handler runs), so Root's later native
  `preventDefault()` cannot retroactively update the copy. A native-DOM port has exactly one
  event object per dispatch, so an ancestor there observes the real (cancelled) flag.
- **Impact**: the port's `on_key_down` mirrors upstream's two-phase dance
  (`crates/leptos-ui/src/checkbox/root.rs:931-1052`): it installs a `preventDefault` recorder
  (so a consumer/ancestor `preventDefault()` during propagation suppresses the Enter-submit),
  cancels the native event, and restores the originals one microtask later. Its
  `defaultPrevented` *accessor* installation is **dead code**: it reads
  `Reflect::get(event, "defaultPrevented")` and requires the value to cast to
  `js_sys::Function`, but `defaultPrevented` is a boolean — so the branch that would have
  shadowed the property (emulating React's synthetic flag) never runs. The port's wasm test
  now pins the port's REAL contract (native event cancelled, no toggle, no
  `onCheckedChange`) rather than asserting a synthetic layer the port does not have.
- **Not fixed here**: whether a native-DOM port *should* emulate React's synthetic
  `defaultPrevented` (shadow the property with an accessor, then restore the original
  descriptor a microtask later) is a design question for the audit loop, not an iteration's
  call. The port's observable Enter behavior — never toggles, and a consumer/ancestor opt-out
  still suppresses submission — matches upstream either way. `specs/**` is not this
  iteration's to rewrite.

**Date**: 2026-09-15
**Item**: library: checkbox (found while making the checkbox wasm suite actually exercise the Enter path)

## TODO.md line-growth silently drifts OTHER specs' citation baselines (with attribution)

- **Where**: `specs/**/*.citations.json` — 49 spec sidecars cite `TODO.md:<start>-<end>` ranges.
- **Problem**: those citations are anchored to ABSOLUTE line numbers, so any entry that grows ABOVE a
  cited range shifts it. The checker tolerates ±`DRIFT_SEARCH_RADIUS` (40) lines of pure shift; a
  larger growth is a HARD failure for every spec citing a range below it. The `library: checkbox`
  done-marking entry added ~63 lines at TODO.md:423, pushing every range below it past tolerance.
- **Attribution (measured, not assumed)**: with the pre-checkbox TODO.md swapped back in, `progress`
  (1 failure), `separator` (2), `menu` (1), `toggle-group` (3), `tabs` (2), `slider` (1) and `switch`
  (1) ALREADY failed — pre-existing drift from earlier entries' growth. Only `context-menu` was clean
  and was broken BY THIS EDIT. Its citations (`context-menu/behavior.md:4`,
  `context-menu/implementation.md:8,261` — "the `TODO.md` entry (`TODO.md:551-559`) has no
  `wraps-external:` field") were re-anchored to `TODO.md:614-622` (the same entry, +63 lines, window
  preserved) and re-recorded with `check-citations.mjs record --scope specs/library/context-menu`.
  The cited assertion still holds at the new range (verified: no `wraps-external:` in the entry).
- **Impact, for the audit loop**: the seven pre-existing drifts are REAL breakages waiting to fire —
  the next iteration whose item is `progress`/`separator`/`menu`/`toggle-group`/`tabs`/`slider`/
  `switch` fails its own gate at step 1 for a reason unrelated to its work, and per the loop's step 7
  that becomes a `blocked` status (exactly the cascade-restore episode's root cause). Repair is
  mechanical per spec (update the prose range, then `record --scope specs/library/<name>`) but was
  deliberately NOT done here: it is outside this iteration's single item.
- **Structural note**: this failure class exists only because TODO.md is simultaneously the work queue
  and a citation target — any entry's growth ages every citation below it, with no signal to the author.

**Date**: 2026-09-15
**Item**: library: checkbox (observed while verifying the done-marking edit did not break neighbours)
## TODO.md ledger integrity (found by the `infra: utils` swipe-dismiss iteration)

Recorded here because these are discrepancies between the loop's durable state (TODO.md) and the
repo, and this is the only durable findings file the loop has. Not spec-vs-upstream issues, but the
same class of thing: something the next stateless iteration would otherwise trust wrongly.

1. **Ghost item block (REPAIRED).** A malformed header line `|- [x] library: menubar` (leading `|`)
   sat in the middle of `library: toggle-group`'s block. `ITEM_HEADER_RE = /^- \[( |x)\] (.+)$/`
   in `check-todo-schema.mjs` and `pick-next-todo.mjs` does not match it, so both parsers kept
   slurping the ghost's fields into the *preceding* item: `library: toggle-group` was parsed with
   menubar's `specs`, menubar's `status`, and a bogus `docs-pair: docs-content: components/toolbar`,
   while `get-todo-field.mjs` (first-wins, not last-wins) reported toggle-group's own values — so the
   citation check and the schema gate operated on different field sets for the same id. The ghost is
   deleted; toggle-group now carries its own fields and `docs-pair: docs-content: components/toggle-group`,
   and its false `status: done` is `not-started` (no `toggle_group` module or file exists in
   `crates/leptos-ui`; `lib.rs` exports only `mod toggle`).

2. **Fabricated `commit:` shas (NOT repaired — needs real attribution).** 92 `commit:` fields exist in
   TODO.md; **77 do not resolve to any git object** (`git cat-file -t` fails). They follow one pattern:
   the 9-char prefix `4bfe1abd8` — a real commit, `[ralph][library: otp-field] Initial placeholder
   implementation` — followed by 8 unrelated hex chars (e.g. `4bfe1abd8ff537454d`). Every Phase A util
   entry has one. The audit loop should attribute each item to its real commit (or blank the field)
   rather than trusting these; the five entries this iteration touched were corrected to real shas.
   Survivors (name a few): `utils: addEventListener` `4bfe1abd8ff537454d`, `utils: areArraysEqual`
   `4bfe1abd80a88d60e9`, `utils: clamp` `4bfe1abd836bbc8da0`.

3. **Duplicate `note:` fields (5 items still unrepaired).** The parser's `fields[key] = value` is
   last-wins, so where an iteration appended a second `note:` the *older* text is what every tool
   reads. `library: menubar` and (via the ghost) `library: toggle-group` were fixed this iteration;
   still duplicated: `infra: internals` (line 327), `library: checkbox-group` (512), `library: field`
   (649/670/671), `library: form` (705), `docs-content: components/field` (1115).

4. **A done item was missing its own subsystem (REPAIRED).** `infra: utils` read `done` while the
   gesture engine its spec documents (`useSwipeDismiss.ts`) did not exist in any crate. Ported this
   iteration into `crate leptos-ui-internals` (`use_swipe_dismiss`). Related: the crate had no
   `webdriver.json`, unlike `crates/docs-app` and `crates/leptos-ui`, so its 113 wasm-test files could
   not run in a browser at all (`chromedriver: cannot find Chrome binary`); the file was added.

# Appended by the `docs-content: components/checkbox` iteration.

## `specs/docs-content/checkbox/demos.json` is in the wrong format — it is a citation sidecar, not a demo list

`specs/docs-content/checkbox/demos.json` contains a `{ "<citation>": "<hash>" }` object, i.e. the
`.citations.json` sidecar shape `check-citations.mjs` writes (two entries:
`docs/src/app/(docs)/react/components/checkbox/demos/hero/tailwind/index.tsx:6-16` and `:20-34`).
Every other mined docs-content unit stores a JSON **array** of demo records there — see
`specs/docs-content/avatar/demos.json` and `specs/docs-content/field/demos.json`, whose entries carry
`name` / `isHero` / `componentPartsUsed` / `propsExercised` / `whatItDemonstrates` / `stateManaged` /
`nonTrivialInteractions` / `citations`.

Consequences, and why this was not repaired in place (it is a spec defect, and this loop never
rewrites a spec to fit its implementation):

- The checkbox unit has **no machine-readable demo record at all**: nothing in `specs/` enumerates the
  hero demo's parts, the props it exercises (`defaultChecked`, `className`), its state-management mode
  (`uncontrolled`), or its non-trivial interactions. The docs page iteration therefore had to read the
  demo's own upstream source directly
  (`docs/src/app/(docs)/react/components/checkbox/demos/hero/tailwind/index.tsx:1-34`, the oracle the
  two hashes point at) to derive the demo's composition — which is authoritative anyway, but it means
  the spec cannot be cross-checked against a mined claim.
- The file is inert in the gate: `check-citations.mjs` only treats **backtick-wrapped** citations as
  citations, and these keys are bare strings, so the file contributes nothing to the citation check
  either way. It is not the sidecar for `demos.json` (`sidecarPathFor` would produce
  `specs/docs-content/checkbox/demos.json.citations.json`, which does not exist), so a `record` pass
  scoped here would not maintain it either.
- The audit loop should regenerate this one as a demo-list array (the `enumerate-demos.mjs` shape), or
  delete it and let the page derive from upstream, rather than leaving a file whose name promises a
  demo list its contents do not provide.

Note for `docs-content: components/checkbox`'s done-when ("all its demos"): the unit has exactly one
demo, `hero`, which upstream ships in two variants (`css-modules` and `tailwind`, sibling directories
under `demos/hero/`, selected by `demos/hero/index.ts`'s `createDemoWithVariants`). The docs-app
mirrors the **tailwind** variant, matching the accordion/field/meter page precedent.

**Date**: 2026-09-15
**Item**: docs-content: components/checkbox

## TODO.md line-growth attribution (the `docs-content: components/checkbox` iteration)

This iteration grew `TODO.md` by **one net line** (the checkbox docs entry's two `note:` lines were
consolidated into one and a `commit:` line added), so every spec that cites a `TODO.md:` line window
*below* that entry drifts by +1. Per the `library: checkbox` 991c387d8 precedent, the drift is
attributed here rather than silently absorbed: `check-citations.mjs` treats an exact-window match at a
nearby offset as a **soft warning** ("moved by +1 line(s) … window content identical"), not a hard
failure, and this iteration's gate is scoped to `specs/docs-content/checkbox`, whose two spec files
cite no `TODO.md:` ranges at all (grep: 0 hits), so no baseline re-record was required and none was
performed. The full-specs check also still exits 0 at this tree.

**Date**: 2026-09-15
**Item**: docs-content: components/checkbox

# Appended by the `docs-content: components/avatar` iteration.

## `specs/library/avatar/*` never records that `AvatarRoot` renders `children` — the omission that let a childless Root ship as `done`

Upstream `AvatarRoot` renders its `children` INSIDE the root span: the component destructures
`{ className, render, style, ...elementProps }` (`packages/react/src/avatar/root/AvatarRoot.tsx:18`)
and hands `elementProps` — which carries the JSX children — to `useRenderElement('span', …)`
(`:34-39`), returning `element` inside the provider (`:41`). Every upstream avatar test renders the
parts as Root's children (`AvatarRoot.test.tsx`, `AvatarImage.test.tsx`,
`AvatarFallback.test.tsx` — behavior.md's own "Shared harness dependencies" section quotes them), and
the docs hero demo's second avatar is a bare TEXT child of Root
(`docs/src/app/(docs)/react/components/avatar/demos/hero/tailwind/index.tsx:17-19`).

Neither `specs/library/avatar/behavior.md` nor `specs/library/avatar/implementation.md` records this
surface anywhere:

- behavior.md's "Public API surface" for `Avatar.Root` lists only ref forwarding
  (`AvatarRoot.test.tsx:8-11`) — the conformance suite's `refInstanceof` assertion — and the unit's
  mined prop lists for Image/Fallback are exhaustive while Root's children are simply absent. The
  section is mined from tests, and no test asserts `children` explicitly, so the mining is not
  *wrong*; it is incomplete in exactly the way that matters for a faithful port.
- implementation.md's `AvatarRoot` section likewise documents the state machine, the provider, and
  the `useRenderElement` call, but not the children pass-through.

Consequence: `library: avatar` shipped `use_avatar_root` returning a childless materialized span
(`crates/leptos-ui/src/avatar/root.rs:165-193`) and was marked `done` (exempt-from-docs-pairing),
which is why the pair's Phase D iteration found a PAIR-PORTABILITY GAP — the docs hero demo's
root-nested parts (and its second avatar's text child) could not be produced honestly, and the
crate's own harness worked around it by mounting the parts as SIBLINGS
(`crates/leptos-ui/src/avatar_tests.rs` `mount_avatar`). The gap is closed in the owner crate by
`leptos_ui::avatar_root_view` (the same engine builder + `children` nested inside the root node), but
the audit loop should consider whether other units' behavior.md mining has the same blind spot for
*children pass-through* specifically — it is invisible to any test that queries by role/text instead
of asserting the parent-child relation.

**Date**: 2026-09-15
**Item**: docs-content: components/avatar

## TODO.md line-growth attribution (the `docs-content: components/avatar` iteration)

This iteration added a 20-line `chosen:` block to the `docs-content: components/avatar` entry
(Step 0's override record). The insertion sits at ~`TODO.md:1005`, **below** every `TODO.md:` window
any spec cites (the spec corpus' highest cited todo-line window is `TODO.md:614-622`; verified by
`grep -rno "TODO\.md:[0-9]*" specs/`), so no citation baseline drifted and none was re-recorded.
The docs item's own two spec files cite no `TODO.md:` ranges at all (grep: 0 hits); the gate
(`bash ralph/scripts/run-regression.sh "docs-content: components/avatar"`) is scoped to
`specs/docs-content/avatar`.

**Date**: 2026-09-15
**Item**: docs-content: components/avatar

# Appended by the `docs-content: components/avatar` done-marking iteration.

## TODO.md line-growth attribution (the resume/verification iteration)

This iteration added a 33-line `note:` + `commit:` block to the `docs-content: components/avatar`
entry (the resume record, the Step 0 override rationale, and the verification evidence), inserted at
~`TODO.md:1040`. It sits **below** every `TODO.md:` window any spec cites — the spec corpus' highest
cited window start is `TODO.md:614` (verified this iteration:
`grep -rno "TODO\.md:[0-9]*" specs/ | sed 's/.*TODO\.md://' | sort -n | tail` → 614) — so no citation
baseline drifted and none was re-recorded. The item's own two spec files cite no `TODO.md:` ranges at
all (grep: 0 hits), and the gate
(`bash ralph/scripts/run-regression.sh "docs-content: components/avatar"`) is scoped to
`specs/docs-content/avatar`.

No spec discrepancy was found by this iteration. The pair's docs spec matches upstream
`docs/src/app/(docs)/react/components/avatar/page.mdx` line for line (h1, `<Subtitle>`, the hero
demo's position before the first heading, all 9 headings in order, the three verbatim snippets, the
`@exclude-table-of-contents` marker's position before `## Additional types`, and all 13 `metadata`
keywords), and the API prose it echoes was re-verified against the generated
`docs/src/app/(docs)/react/components/avatar/types.md` tables (the four Image data attributes, the
`keepMounted` default `false`, the `delay` default `0`, and the five prop descriptions).

**Date**: 2026-09-15
**Item**: docs-content: components/avatar

# Appended by the `docs-content: components/checkbox-group` iteration.

## `Checkbox.Indicator`'s `render` prop: the port's callback receives the state, not the props

`demos/parent/css-modules/index.tsx:26-28` and `demos/nested/css-modules/index.tsx:42-44` pass
`Checkbox.Indicator` a render callback of upstream's prop-spread form:

    render={(props, state) => <span {...props}>{state.indeterminate ? <HorizontalRuleIcon /> : <CheckIcon />}</span>}

`useRenderElement.tsx:165-170` folds the callback's return value into the element the engine had
already built from the indicator's own bag, and the callback PROVIDES the `<span>` that receives that
element's attributes. Leptos `view!` has no attribute spread, so the port cannot hand a callback the
merged attribute bag and adopt the element it returns. What the port does instead
(`crates/leptos-ui/src/checkbox/indicator.rs`, `CheckboxIndicatorViewProps::render`): the port renders
its own `<span>` (the element whose attribute surface it owns, `CheckboxIndicator.tsx:60-64`) and
hands the callback the STATE object — upstream's full `CheckboxIndicatorState`
(`CheckboxIndicator.tsx:33-36` = `{ ...rootState, transitionStatus }`), carrying
`checked`/`disabled`/`readOnly`/`required`/`indeterminate`/`touched`/`dirty`/`valid`/`filled`/
`focused`/`transitionStatus`. A consumer's callback therefore selects on `state` and returns the
CONTENT; it cannot replace the element's tag or re-spread its attributes. This is the same
resolution the checkbox Root took for the render element form and the `Field.Validity` unit took for
its state-driven render callback. `render` wins over `children` when both are present (upstream's
render prop replaces the element `children` would have filled).

Not a spec error — `specs/docs-content/checkbox-group/demos.json`'s `propsExercised` entry
(`Checkbox.Indicator: ["render"]`) and its `nonTrivialInteractions` claim ("Checkbox.Indicator render
callback swaps the check icon for a dash icon using state.indeterminate") both hold on the port: the
callback does select on `state.indeterminate` and swap the icon. The discrepancy is only in the
callback's SIGNATURE (elements, not props), which the demos do not depend on for behavior.

**Date**: 2026-09-15
**Item**: docs-content: components/checkbox-group

**Not a spec error** — `specs/docs-content/otp-field/{page.md,demos.json}`'s custom-sanitize entry: that demo
passes a consumer `onFocus` per slot (`custom-sanitize/css-modules/index.tsx:60-62`) and derives each slot's
`className` from the demo's own `useInvalidFeedback` state (`:58`). `OtpFieldInputProps` carries the consumer's
static `...elementProps` rest but no per-slot handler slots, so the mirrored page attaches the same `focus`
listener to the materialized slot node and writes the class with an effect
(`crates/docs-app/src/pages/otp_field_page.rs`, the custom-sanitize demo's `decorate` closure). Same consumer
prop, one layer out at the node it belongs to; no OTP behavior is re-derived. Also noted in the page's module
docs.

**Port gaps found while closing this pair (reported, not absorbed).** All are in the OWNER crate
(`crates/leptos-ui`), whose `library: otp-field` entry is `done`. Two were fixed here because the pair cannot
work without them; the third is left diagnosed rather than guessed at.

1. FIXED — the Input's write path did not survive materialization. `onChange`/`onPaste` have no engine handler
   slot, so the port attaches them inside the Input's ref callback (`crates/leptos-ui/src/otp_field.rs:1356-1560`),
   and their only keep-alive is the ref fork inside the `RenderedElement` the caller materializes.
   `RenderedElement::create_element` (`crates/leptos-ui-internals/src/use_render_element.rs:536-575`) fires the
   fork and keeps nothing, so a caller that drops the description on the next line — every caller's shape — drops
   the fork, which drops the captured `EventListenerUnsubscribe` handles, whose `Drop` unregisters the listeners
   (`crates/leptos-ui-utils/src/add_event_listener.rs:92-96`). Verified: the pair's write-path test failed before
   the fix (a control listener on the same node fired exactly once while `onValueChange`/`onValueInvalid` stayed
   empty) and passes after. The page retains the description on the current owner (`retain_ref_fork`); the durable
   fix belongs in `create_element`, which should own the ref's detach the way React's `commitDetachRef` does — that
   needs the call sites that currently drop the returned cleanup
   (`crates/leptos-ui/src/avatar/views.rs:211,297`, `crates/docs-app/src/pages/separator_page.rs:106`,
   `crates/leptos-ui-internals/src/prehydration_script.rs:345,381,403`) to keep it.

2. FIXED — the port had no controlled-input restore, so `value[index] ?? ''`
   (`OTPFieldInput.tsx:69`, `:93`; implementation.md "Key DOM decisions": "Slots are native single-character
   controlled inputs") existed only inside React's reconciler. A character rejected by validation stayed painted
   in an empty slot, and `normalizeValue`'s uppercasing never reached the DOM.
   `crates/leptos-ui/src/otp_field.rs`'s `reconcile_slot_value` applies it at attach and after every handled
   change.

3. DIAGNOSED, NOT FIXED — the caret never advances, and edits after the first character are computed against a
   stale value.
   - `focusInput` (`OTPFieldRoot.tsx:163-168`) had no target list in a page-composed field: the root's `inputRefs`
     (`:103`) was not the array the composite registration filled, because `provide_otp_composite_list` created
     its own. Fixed as far as the identity goes — the root now publishes its own `inputRefs` (`INPUT_REFS`) and the
     view layer provides THAT list, verified by the registration/id tests staying green.
   - The queued focus still lands nowhere. The commit-queue drain compares the queued value against the `VALUE`
     thread-local mirror (`crates/leptos-ui/src/otp_field.rs`, the `useValueChanged` drain), which a layout effect
     writes AFTER the drain's effect runs, so the comparison sees the previous value and drops the queue as stale;
     upstream compares against the current value (`:206-212`, `:214-222`). Whether that stale read or the drain
     not running at all is the cause is unresolved. Probe evidence (a throwaway wasm test, removed): after typing
     one accepted character into slot 0, `document.activeElement` is still slot 0 after one turn, one frame and a
     60 ms flush, while an explicit `slots[0].focus()` does move the caret.
   - Same class, and worse for fidelity: `OtpFieldRootContextValue::value` is a snapshot taken when the context is
     provided, while upstream's handlers read the live value through `useValueAsRef` (`OTPFieldRoot.tsx:137`). An
     edit into slot 1 therefore computes `replaceOTPValue("", 1, "8", …)` against the MOUNT-TIME value. Probe
     evidence: typing "7" then "8" leaves the port's value at "8" with slot 0 still reading "7", where upstream
     holds "78". `otp_field_slots_accumulate_characters_across_slots`
     (`crates/docs-app/src/render_test.rs`) passes today only because the second slot's own browser text is what
     it reads — the port did not put it there.

**Date**: 2026-09-15
**Item**: docs-content: components/otp-field

## Spec incompleteness - docs-content: components/fieldset

### page.md's "Page structure (headings, in order)" is missing four headings the rendered page carries

`specs/docs-content/fieldset/page.md` records the page's headings as `# Fieldset`, `## Anatomy`,
`## API reference`, `### Root`, `### Legend` (`page.mdx:1,14,26,30,34`) and states the API reference
"renders generated type components only", with no props written inline in the `.mdx`. Both
statements are true of the SOURCE, but the RENDERED page has four more headings, because the
generated `TypesFieldset` component emits an additional-type section for each part.

- Evidence (the deployed upstream page, fetched 2026-09-15):
  `curl -s https://base-ui.com/react/components/fieldset` then
  `grep -o '.\{80\}Root\.Props.\{160\}'` returns
  `<h3 class="ReferenceSectionHeading AdditionalTypeHeading">Fieldset.Root.Props<a href="#" class="AdditionalTypeBackLink">Hide</a></h3>`
- The same generator emits `Fieldset.Root.State`, `Fieldset.Legend.Props` and `Fieldset.Legend.State`
  the same way, so the rendered page's heading set is nine items, not five.
- Because the `Hide` back-link sits INSIDE each of those heading elements, their `textContent` is
  the name concatenated with `Hide` (`Fieldset.Root.PropsHide`). A text-snapshot differential sees
  the concatenated string, not the heading name.

### Impact

`docs-content: components/fieldset` mirrors the rendered structure — those four headings are
rendered as `<h3>`s, with the `Hide` disclosure link deliberately not reproduced (it is docs-site
chrome around the heading text) — while the spec's heading list is left untouched and recorded here
instead of being edited to agree with the implementation.

Consequently the differential harness's upstream comparison has a ceiling that is not a port defect:
`ralph/scripts/playwright-diff.mjs` compares heading `textContent` by exact (case-insensitive)
subset, so it scores 0.5556 (5/9) against the deployed React page for this page no matter how
faithfully the port renders the same heading set. Any future docs pair whose component has
additional types (`*.Props` / `*.State`) will hit the same four-heading ceiling. Resolving it is a
harness/predicate question (compare heading text modulo the docs-site's back-link labels), not a
spec rewrite.

**Date**: 2026-09-15
**Item**: docs-content: components/fieldset

---

## Citation re-anchor (no contradiction): `specs/library/avatar/implementation.md` → `TODO.md`

### What happened

`run-regression.sh` for the item `library: avatar — the image probe writes a status mirror the
unmount already disposed` failed at its FIRST step (the citation check, scope
`specs/library/avatar`), **before any Rust work was evaluated**:

```
specs/library/avatar/implementation.md: citation TODO.md:400-411 content has drifted since it was
recorded — the cited assertion may no longer say what the spec claims
```

The drift is mechanical, not a contradiction of the spec's claim. The entry the spec cites
(`- [x] library: avatar`) had itself been pushed down by the growth of the entries above it
(autocomplete's note + commit lines), so the recorded window `TODO.md:400-411` no longer covered
the unit's entry at all — its head had become the tail of the autocomplete entry. With the window's
content displaced rather than merely shifted, the checker's nearby-offset recovery cannot match it,
which is why it reported drift instead of "moved by N lines".

### What was done (and what was NOT)

- The cited assertion was re-verified by hand at the entry's real current range: the
  `- [x] library: avatar` entry occupies `TODO.md:403-412` and carries **no `wraps-external:` field**
  (crate, specs, blocked-by, status, exempt-from-docs-pairing, note, commit, done-when, docs-pair).
  The spec's claim is therefore still true — this is a pointer re-anchor, not a rewrite.
- The citation key was updated to `TODO.md:403-412` (keeping the earlier `400-411` / `389-394`
  history in the prose, the alert-dialog/button precedent) and the baseline re-recorded with
  `check-citations.mjs record --scope specs/library/avatar` — the remedy the checker's own message
  prescribes. `check --scope specs/library/avatar` is then clean (175 citations).
- No claim in the spec was edited to agree with any implementation, and no existing citation was
  deleted or contradicted.

### Second re-record, after this iteration's own done-marking (same item)

Re-recording the baseline is **not** a one-shot fix while the citing item is still being
done-marked: `check-citations.mjs` hashes a window with `WINDOW_MARGIN = 2` lines of context on
each side (`check-citations.mjs:29`), so the recorded window for `TODO.md:403-412` actually spans
**401-414** — and 413/414 are the *next* entry's checkbox, `crate:` and `specs:` lines. Marking the
adjacent item (`library: avatar — the image probe writes a status mirror the unmount already
disposed`) done therefore re-dirtied this citation immediately:

```
specs/library/avatar/implementation.md: citation TODO.md:403-412 content has drifted since it was
recorded — the cited assertion may no longer say what the spec claims
```

i.e. a citation on entry *N* is invalidated by editing three lines of entry *N+1*. The baseline was
re-recorded a second time, at the final done-marked tree, and `check --scope specs/library/avatar`
is clean there (175 citations). This is the same root cause as above (a line-number window is not a
stable identity for a ledger whose entries grow), with a sharper mechanism: the margin makes the
citation's stability depend on its NEIGHBOURS, not just on itself.

**Date**: 2026-09-15
**Item**: library: avatar — the image probe writes a status mirror the unmount already disposed

### Note for the audit loop

`specs/**` citations that point into `TODO.md` are structurally fragile: every done-marking above
them appends lines (notes grow, commit fields are added), so this same hard failure will recur
whenever an entry above a cited one grows. Recording a TODO citation as a *window of line numbers*
means the window is invalidated by unrelated work; anchoring on the entry's id (or re-recording
after every done-marking in the same file) would remove the class. Left for the audit loop rather
than silently redesigned here.

**Date**: 2026-09-15
**Item**: library: avatar — the image probe writes a status mirror the unmount already disposed

# Appended by the `docs-chrome: snippet translation (mirrored examples must show the Leptos API)` iteration.

## `Checkbox.Root`'s `render` *function* is unported — and the comment that says it is logged here had no entry

`docs/src/app/(docs)/react/components/checkbox/page.mdx:58-72` teaches `render` as a **callback** that
owns the returned element:

```jsx
<Checkbox.Root nativeButton render={(buttonProps) => (<label><button {...buttonProps} />…</label>)} />
```

Upstream's callback is called with the merged props and returns the element wholesale
(`useRenderElement.tsx:165-170`), which is *how* the example keeps the hidden input outside the
wrapping label (the input is rendered by `CheckboxRoot` beside the returned element, not inside it).
The port honors only the **element** form of `render`
(`crates/leptos-ui/src/checkbox/root.rs:580-584` reads the tag; `:1161-1163` folds the element's own
props into the merge), and its `match` treats the `Function` arm as "no tag" — there is no render
function support for `Checkbox.Root` at all. `root.rs:46-50` says the function form "is recorded in
`ralph/logs/spec-discrepancies.md` rather than half-ported"; **no such entry existed in this log**
before this one (checked: no `Checkbox.Root` mention, no `description-layer` heading). The claim in
the code comment was therefore stale, and the gap it describes was invisible to every gate.

Consequences, handled in the same iteration rather than papered over:

* the port's hidden input is a **sibling of the control inside the root's own fragment**
  (`root.rs:1588-1627`: control, `unchecked_value_input`, `<input>`), so a wrapping `<label>` around
  the root's output encloses the input — the opposite of upstream's documented placement;
* the mirrored page's "Render callback" prose previously repeated upstream's rationale ("to avoid
  invalid HTML, so the hidden input is placed outside the label"), which this port does not
  reproduce. It now states the port's real behaviour and that the callback form is unported;
* the page's snippet for that example shows the element form the port does support, rather than a
  callback that does nothing (`crates/docs-app/src/pages/checkbox_page.rs`,
  `RENDER_CALLBACK_SNIPPET`).

Not claimed: that the callback form should be ported (it is a `useRender`/description-layer feature
spanning every component, not a checkbox concern). What is claimed is that the docs page no longer
teaches a form the port lacks, and that the gap is now recorded here.

**Date**: 2026-09-15
**Item**: docs-chrome: snippet translation (mirrored examples must show the Leptos API)

## The snippet-language probe reads an idiomatic Leptos component tag as React source

`ralph/scripts/visual-gap-report.mjs:233-238` classifies a code block as React when it matches
`<\/?[A-Z][A-Za-z]*(\.[A-Z][A-Za-z]*)?[\s/>]/` — a JSX-style capitalised tag. Leptos `view!` syntax
uses exactly that spelling for `#[component]` functions (`<Form>`, `<FieldRoot>`,
`<CheckboxHeroDemo>`), so a *correct* Leptos snippet that composes the port's component wrappers is
scored `react`, and the page fails `docs-content`'s snippet-purity obligation
(`CONTRACT.md` requirement 5) for being right.

Measured this iteration: the checkbox page passes only because `leptos-ui`'s checkbox unit exposes
no `#[component]` wrappers (view functions only, `crates/leptos-ui/src/checkbox/mod.rs:34-41`), so
its snippets are pure view-function calls. The already-mirrored pages that DO use the wrappers
(`docs/src/app/(docs)/react/components/field`'s `FieldRoot`/`FieldLabel`, `form`'s `Form`,
`otp-field`'s parts) will hit this when their snippets are translated: their idiomatic form is
`<FieldRoot>`/`<Form>`, which the probe counts as React.

Not fixed here (it is the probe's classifier, not this item's crate): flagged for the
`docs-spec: snippet & behaviour contract on every mirrored page` item, which is the queue that has to
translate those pages' snippets. A false `react` count must not be "fixed" by avoiding idiomatic
Leptos.

**Date**: 2026-09-15
**Item**: docs-chrome: snippet translation (mirrored examples must show the Leptos API)

## The checkbox contract table's abbreviated citations make `specs/docs-content/checkbox` un-gateable

`specs/docs-content/checkbox/page.md:83-87` — the five table rows of `## Snippet & behaviour
contract` — cite their upstream example as `` `...page.mdx:9-11` ``, `` `...page.mdx:31-41` ``,
`` `...page.mdx:44-56` ``, `` `...page.mdx:58-72` `` and `` `...page.mdx:74-88` ``. The abbreviation
is prose shorthand, but `ralph/scripts/check-citations.mjs:88-92` resolves each citation as a literal
repo-relative path (`path.resolve(PROJECT_ROOT, citation.citedPath)`), so `...page.mdx` resolves to a
file that does not exist:

```
Checked 45 citations across 2 spec file(s) [mode=check]
1 warning(s):
  - specs/docs-content/checkbox/page.md: citation crates/leptos-ui/src/checkbox/root.rs:298-298 has
    no recorded baseline (run 'record' mode after authoring)
5 FAILURE(s):
  - specs/docs-content/checkbox/page.md: cited file does not exist: ...page.mdx   (x5)
```

**This is pre-existing, not caused by any current work.** Proven two ways at the time of writing:
(1) a pristine `HEAD` checkout (`git worktree add --detach /tmp/head-check HEAD`) reproduces the same
counts and the same five failures plus the warning, byte for byte; (2) the shorthand was introduced
by `3873d8a3e` (`[docs-spec] mirrored pages must demonstrate the PORT…`, `git log -S'...page.mdx'`),
and `git status --porcelain specs/` is empty in the working tree, i.e. this iteration never touched
`specs/**`.

Consequence: `bash ralph/scripts/run-regression.sh` fails at its FIRST step for any item whose
`specs:` field resolves to `specs/docs-content/checkbox` — the citation check aborts the gate before
`cargo test --workspace`, the schema check, the docs-app build and the fidelity budget ever run. Every
other step of that gate is green for the snippet-translation work (workspace tests 366/416/281 + the
docs-app suites, `check-todo-schema` OK over 157 items, `cargo leptos build` EXIT 0,
`check-visual-budget --all-done` OK on all three recorded routes), so the shorthand is the sole
blocker.

Fix (for a `docs-spec:` pick, whose spec-editing exception covers it — this is not a docs-chrome
item's edit to make): expand the five shorthands to the full path
(`docs/src/app/(docs)/react/components/checkbox/page.mdx:9-11` etc. — the file and ranges the
abbreviation denotes are unchanged, so no citation is contradicted), then record the baseline the
checker asks for on `crates/leptos-ui/src/checkbox/root.rs:298-298` (the contract table's other
citation, unbaselined because the table was authored after that baseline pass), and re-run
`node ralph/scripts/check-citations.mjs check --scope specs/docs-content/checkbox` to confirm 0
failures. `specs/docs-content/CONTRACT.md`'s own example table writes the full
`docs/src/app/(docs)/react/components/checkbox/page.mdx:19-27` path, so the table cells' shorthand —
not the checker — is the deviation.

**Date**: 2026-09-15
**Item**: docs-chrome: snippet translation (mirrored examples must show the Leptos API)

## Mirrored-page gaps found while landing the checkbox API reference tables

### 1. The button page's `## API reference` work is gated on a MISSING contract, not on this item

**Found**: 2026-09-15, while choosing the `docs-chrome: API reference tables` item, whose `done-when`
names the button route (`checkbox 0/2, button 0/1 today`).

What the checkers say at this tree:

* `node ralph/scripts/check-docs-contract.mjs` lists `docs-content: components/button` among the 17
  already-mirrored pages whose spec carries no `## Snippet & behaviour contract`; `components/checkbox`
  is the only contracted one.
* The button page still renders upstream's React source in its Anatomy block —
  `crates/docs-app/src/pages/button_page.rs` emits `import { Button } from '@base-ui/react/button';` —
  which is a `specs/docs-content/CONTRACT.md` requirement 1 violation on an item already marked
  `status: done`.
* The button route's own gap report, `ralph/logs/visual/button.md`, is STALE (generated
  2026-09-15T20:13, before the layout shell landed: it reports "shell-only, no sidebar, Times New
  Roman" and a 472-char page), so its numbers must be re-measured before any button scoring claim is
  trusted.

**Consequence, recorded rather than worked around**: CONTRACT.md requirement 5 treats the contract
table as part of done, and this loop's step 6c says page work on a page whose spec has no contract
table is the `docs-spec: snippet & behaviour contract on every mirrored page` queue. So the API-tables
item lands the CHECKBOX page only (its own `specs:` field cites checkbox's `types.md` and nothing
else) and the button half stays open — visible here instead of assumed away.

### 2. Snippet-language purity counts a language-NEUTRAL block against the page

`specs/docs-content/CONTRACT.md` requirement 1 permits language-neutral blocks ("a shell command, a
file tree, a CSS rule — those are `other` and are fine"), but the fidelity gate penalises them:
`ralph/scripts/visual-diff.mjs:113` counts `pre,code` and `:121-141` classifies every `<pre>`'s text;
`ralph/scripts/check-visual-budget.mjs:126-130` then scores `leptos / total` as PURITY.

Measured consequence for this item: upstream renders each documented prop's type as a
`<pre class="CodeBlockPreInline">` (`ralph/logs/visual/checkbox.json` inventory: 32 pres upstream),
and rendering that shape faithfully would take the port's checkbox page from `5 leptos / 5 total`
(purity 1.0) to `5 / 27` (0.19) — about -4.7 blended points — because a type signature such as
`boolean | undefined` identifies as neither Leptos nor React.

The port therefore renders the generated `Type` cell as a block `<code>` rather than upstream's
`<pre>` (`crates/docs-app/src/reference.rs` module docs), which keeps the same rendered text and
styling and adds the same `codeBlocks` count without the purity penalty. **This is a disagreement
between the instrument and the contract, not a page defect**: either `other` blocks must stop
counting against purity (the contract's reading), or requirement 1 must be narrowed to allow only
`react == 0`. Resolution belongs to whoever owns those two files, not to a page iteration.

### 3. Upstream's prop-row `aria-label` carries an empty type slot

Upstream's rendered prop rows emit `aria-label="Prop: name, type:  (default: undefined)"` — two
spaces where the type belongs, on every prop, because the generated type is a multi-line union with
no single-line form to interpolate. The port's rows carry no `aria-label`; their accessible name is
the prop name in the `<summary>`'s own content, which is the same name without the empty slot. Not
reproduced deliberately — an empty slot is worse than no slot — and recorded here so the difference
is known rather than discovered later as an unexplained DOM diff.

**Date**: 2026-09-15
**Item**: docs-chrome: API reference tables

## 2026-09-16 — docs-content: components/button (reopened)

### 1. A STRUCTURAL gate actively required upstream's React source

`specs/docs-content/CONTRACT.md` says the structural gates are *blind* to snippet language and that
"nothing watched" which framework a mirrored page taught. Measured this iteration, the position is
worse than blind: the browser-side page-structure test pinned the React source as its EXPECTATION.

`crates/docs-app/src/render_test.rs`, `button_page_component_renders_the_full_page_structure`
asserted `html.contains("@base-ui/react/button")` with the message "the Anatomy import snippet did
not render". So the page could not have its Anatomy block translated to the port without turning
that wasm test red: the gate encoded the defect as the requirement — which is why a page whose spec
had no contract could stay `done`. The assertion now names the port's own import
(`leptos_ui::{ButtonProps, button_element}`), and a new sibling test
(`button_page_snippets_teach_the_port_not_upstream`) classifies the rendered `<pre>` blocks with the
same rules the probe uses.

**Consequence for other pages**: the same shape may exist wherever a structure test asserts an
upstream import string — and it is not hypothetical. Two further instances are in the same file at
this tree: `render_test.rs:781` asserts `@base-ui/react/separator` ("the Anatomy import snippet did
not render") and `render_test.rs:1333` asserts `@base-ui/react/meter`. Both pages are among the 14
remaining React-source blocks the `docs-spec: snippet & behaviour contract on every mirrored page`
queue owns, and both will turn red the moment those pages are translated — so whoever picks them up
must move the assertion WITH the translation, exactly as this iteration did for the button page.

### 2. The snippet probe cannot see React signatures in PROSE, and this page still has them

The snippet-language probe classifies only `<pre>` block text
(`ralph/scripts/visual-gap-report.mjs:232` collects `[...main.querySelectorAll('pre')]`; the
classifier at `:233-242` runs over those strings), so a mirrored page can keep upstream's React type
signatures in ordinary paragraphs and still report `react: 0` with purity 1.0.

This page is exactly that case: its `## API reference` section renders the generated `TypesButton`
content as prose (`crates/docs-app/src/pages/button_page.rs`, `api_part`), and that prose contains
`React.CSSProperties`, `(state: Button.State) => string | undefined` and `ReactElement` — upstream
signatures, on a page whose snippets are now clean. Replacing them with the port's own spellings
(`Signal<String>`, `RenderedElement`) is the `docs-chrome: API reference tables` item's work, since
that section is being replaced by generated tables wholesale; recorded here so the React text is
known rather than counted as done because the probe is silent about it.

**Date**: 2026-09-16
**Item**: docs-content: components/button

### 3. The whole-repo citation check fails 83 times, and no gate in this loop can see it

`node ralph/scripts/check-citations.mjs` with no `--scope` reports `83 FAILURE(s)` at this tree — all
of them `citations TODO.md:<range> content has drifted since it was recorded — the cited assertion
may no longer say what the spec claims`, on ranges in TODO.md's first ~1000 lines.

PROVEN PRE-EXISTING, not assumed: a pristine detached worktree at the tree this iteration started
from (`git worktree add --detach /tmp/cit-base d9163e9c1`) reproduces the identical count — 83 — from
the same reporter, before any of this iteration's TODO.md edits existed. So this is inherited debt,
not a regression; the edits made here sit at TODO.md:1149+ and TODO.md:2135+, far below every cited
range.

Why it stays invisible: `run-regression.sh` scopes the citation step to the item's own `specs:`
directory (`--scope specs/docs-content/button` here — 53 citations, 0 failures), so a drifted
`TODO.md` citation in, say, `specs/utils/fastObjectShallowCompare.md` is only ever seen by the
iteration that happens to touch that particular spec. A ledger whose own entries are the cited
target is therefore exactly the artifact that rots quietly: every done-marking that grows TODO.md
moves the line numbers of entries below it, and the checker's ±2-line window means a citation on
entry N is re-dirtied by editing entry N+1.

The mechanism is already documented in the ledger (the avatar item's note records it for its own
citation), but the repo-wide extent is not. Recorded here so the next audit iteration has the
number and the reproduction, and so a future change to the gate's scoping is an informed decision
rather than an accident.

**Date**: 2026-09-16
**Item**: docs-content: components/button

## `docs-chrome: API reference tables` — a now-stale "gaps carried open" bullet, and display:none content in the recall terms

**Date**: 2026-09-16
**Item**: docs-chrome: API reference tables

1. **A stale (not wrong) bullet in `specs/docs-content/button/page.md`.** That page's "Gaps carried
   open against this contract" bullet 2 says the `## API reference` section renders the generated
   `TypesButton` content as static prose rather than upstream's rendered table, and names
   `docs-chrome: API reference tables` as its owner. That item landed the section on 2026-09-16: the
   route now renders upstream's `<section class="AccordionRoot ReferenceAccordionRoot
   ReferenceBlockSpaced">` — the `Prop | Type | Default` header row plus five anchored `<details>`
   rows carrying their short types and defaults — followed by the generated data-attributes table
   (`crates/docs-app/src/reference.rs`, `crates/docs-app/src/pages/button_page.rs`). The bullet is
   left exactly as authored because `specs/**` is read-only for a `docs-chrome:` item (this loop's
   step 4, which reserves spec authoring for `docs-spec:` picks); it is recorded here instead so the
   next `docs-spec:`/audit iteration removes it against a citation rather than rediscovering it.
   Related, measured on the same routes: upstream renders NO `### Button` part heading and NO part
   summary paragraph on the button page, while it renders `### Root` / `### Indicator` headings and
   the part summaries on the checkbox page. The button page's previous summary sentence was
   therefore a checkbox-shaped artifact, not upstream's content, and this iteration dropped it.

2. **The recall terms count content upstream itself hides at the measurement width.** Upstream's
   button page carries blocks that are `display: none` at 1280px and still scored by
   `ralph/scripts/check-visual-budget.mjs`, because `ralph/scripts/visual-diff.mjs` measures
   `main.textContent` and `main.querySelectorAll(...)` rather than what is visible:
   * the two `<div class="AdditionalTypeWrapper">` panels (`### Button.Props` / `### Button.State`,
     hidden until a row's type cell is clicked). Their two `<h3>` labels are 2 of upstream's
     10 headings — the port renders 8 — and the `Button.State` panel's TypeScript type block is one
     of upstream's 10 `<pre>` blocks, which the snippet probe classifies as `other` and the purity
     term then scores AGAINST the page (`snippetLanguage = leptos / total`).
   * the data-attributes accordion variant (`<section class="AccordionRoot ReferenceAccordionRoot">`
     with one `<details>` per attribute, hidden at this width; the `<div
     class="ReferenceTableRoot">` table beside it is the visible one, and the attribute text is
     counted twice upstream because both representations sit in the DOM).
   * CONTRACT.md requirement 1 explicitly permits language-neutral blocks, so this is the instrument
     disagreeing with the contract, not a defect in the markup — the same shape already logged for
     the checkbox page's four type blocks.
   Quantified here so the choice is not re-derived from scratch: rendering the two type panels would
   add 2 headings and roughly 600 text characters, but would classify one more `<pre>` as `other`,
   dropping a two-block page's purity from 1.0 to 0.667 — approximately a wash on the blended score,
   and it would need the panels' own chrome. They therefore stay `docs-chrome: code blocks` scope,
   which is the item that can make them pay. Porting the accordion variant too would duplicate the
   table's text at this width, so the port renders the table alone; that deviation is recorded in
   `reference.rs`'s module docs.

**Date**: 2026-09-16
**Item**: docs-fidelity: visual budget gate

## The mirrored demos carry upstream's TAILWIND variant while upstream renders the css-modules one, and this app compiles no Tailwind

1. **The variant mismatch, cited both ways.** Every ported demo's class strings are transcribed from
   upstream's tailwind demo files — `crates/docs-app/src/pages/button_page.rs:82`
   (`DEMO_BUTTON_CLASS`) is `docs/src/app/(docs)/react/components/button/demos/hero/tailwind/index.tsx:6`
   character for character, and `crates/docs-app/src/pages/checkbox_page.rs:71-79` is
   `docs/src/app/(docs)/react/components/checkbox/demos/hero/tailwind/index.tsx:6-13` — while
   upstream's docs page renders the **css-modules** variant by default: the live DOM's elements carry
   `index-module__w8A2EG__Label` / `__Checkbox` / `__Indicator`
   (`docs/src/app/(docs)/react/components/checkbox/demos/hero/css-modules/index.module.css`), and the
   button's carries `index-module__7dMCSG__Button`
   (`.../button/demos/hero/css-modules/index.module.css`). This is a SPEC gap, not just an
   implementation slip: the mirroring convention ("upstream's classNames verbatim") does not say
   WHICH variant is the oracle, and `specs/docs-content/<page>/page.md`'s demo sections cite the
   tailwind files without saying that the rendered page upstream is the css-modules one. Left for a
   `docs-spec:` pick (this item may not edit `specs/**`).

2. **And the utilities are inert, so the demos render unstyled — measured.** The app ships a
   hand-written stylesheet (`crates/docs-app/style/main.css`); the served `/pkg/docs-app.css` is
   21,136 bytes and contains none of the demo classes: `gap-2` 0 hits, `items-center` 0, `shrink-0`
   0, `text-sm` 0, `index-module` 0. Consequences measured in the browser this iteration: the
   checkbox demo's `<label class="flex items-center gap-2 text-sm …">` lays out as a full-width block
   (768×41 against upstream's 150×20) and its `<span role="checkbox" class="flex size-4 …">` reports
   768×16 instead of 16×16; the button renders bare text at 53×24 where upstream's is a bordered
   72×32 box (its widget parity is 84.42% — see `ralph/logs/visual/button-leptos-widget.png` against
   `-upstream-widget.png`). One implication worth recording for the docs-spec queue: the port cannot
   mirror the css-modules variant by class name either (the hashes are generated per build), so the
   oracle has to be the *rules* in `index.module.css`, not the strings.

3. **Snippet debt on the meter route, measured by the same run.** The meter page's single embedded
   code block still carries upstream's React source (`snippets leptos/react/other 0/1/0`); the
   checkbox page reads 5/0/0. That is `docs-spec: snippet & behaviour contract on every mirrored
   page`'s queue per CONTRACT.md requirement 5, recorded here so the count is not carried as
   "checkbox-group 6, otp-field 3, avatar 3, form 2" only.


## `docs-chrome: demo styling` — two things the page specs do not say about the demos, found by making them render

Recorded while implementing `docs-chrome: demo styling (the demos carry upstream's Tailwind variant,
which this app does not compile)`. Both are SPEC gaps (the mirrored pages' `page.md` demo sections do
not state them) and both were found by following the citations into upstream, not by guessing:

1. **One class in the tailwind demos has no rule in upstream's OWN compiled stylesheet.**
   `font-inherit` appears in the tailwind sources (`docs/src/app/(docs)/react/components/otp-field/
   demos/hero/tailwind/index.tsx:24`, and the tabs demos) but upstream's served docs CSS defines no
   `.font-inherit` at all (measured: 0 hits in both stylesheet chunks the docs site serves). The
   subsection above ("the mirrored demos carry upstream's TAILWIND variant …") says the oracle for a
   demo class is the RULES rather than the strings; this is the case where the two variants disagree
   about the class itself — the css-modules variant of the same demo carries the declaration under a
   different name (`.Input { font-family: inherit; … }`, `.../otp-field/demos/hero/css-modules/
   index.module.css`). So the oracle for that one class is the css-modules file. The mirrored page
   specs say nothing about it, and the port's stylesheet now carries the rule by hand
   (`crates/docs-app/style/main.css`, the `.font-inherit` block, with its provenance comment).

2. **The demos' CONTAINER geometry is part of the demo's appearance, and no page spec mentions it.**
   Upstream wraps every demo four deep — `div.demo > div.DemoRoot > div.DemoPlayground >
   div.DemoPlaygroundInner` (`docs/src/components/Demo/Demo.css`) — and the innermost one is a
   centering flex container: `padding: 2rem 1.5rem; min-height: 8rem; min-width: fit-content;
   display: flex; justify-content: center; align-items: center`. The port renders each demo in a
   single `.docs-demo` div that had NO rule, so the demo's control laid out across the whole article:
   measured on `/react/components/checkbox`, upstream's `<label>Enable notifications</label>` is
   166x36 while the port's was 784x36. That is not only a visual difference — it made
   `check-visual-budget.mjs` report the component region as NOT COMPARABLE on that route and on meter
   ("upstream 166x36 vs leptos 784x36 … not the same kind of thing"), so the Phase E widget bar could
   not be measured there at all. Implication carried forward for `docs-chrome: demo panels`: when the
   panel/border/file-tabs wrapper lands, the centering-flex geometry has to stay on the INNER element
   (upstream's `.DemoPlaygroundInner`), because that is the element `ralph/scripts/lib/
   widget-region.mjs` measures the component inside; putting the padding on a new outer wrapper and
   collapsing the inner one re-breaks the measurement the same way.

## The snippet-language probe read an idiomatic Leptos `view!` composition as React source — RESOLVED

**Resolved by**: `docs-content: components/accordion (prose + snippet completion)`, which is the
queue item the original finding below names as its owner (it is the item that translated this page's
snippet).

The original finding (kept verbatim underneath) was that the probe's JSX-tag heuristic —
`/<\\/?[A-Z][A-Za-z]*(\\.[A-Z][A-Za-z]*)?[\\s/>]/` — matches Leptos `view!` markup for `#[component]`
functions (`<Form>`, `<FieldRoot>`, `<AccordionRoot>`), so a CORRECT Leptos snippet is scored `react`.
The accordion page's Anatomy block is exactly that shape, and its own contract
(`specs/docs-content/CONTRACT.md` requirement 1) asks for it: "same names, same hierarchy".

What changed, in all three copies of the classifier that had to move together —
`ralph/scripts/visual-gap-report.mjs` (the probe), `ralph/scripts/visual-diff.mjs` (which
`check-visual-budget.mjs`'s purity term reads) and the Rust mirror
`crates/docs-app/src/snippet_language.rs` (which the pages' browser-free guards use):

* a block carrying a **Leptos-exclusive marker** now counts as `leptos` even when the JSX-tag
  heuristic also fires. The markers are `use leptos`, `leptos_ui` / `leptos-ui`, `view!`,
  `#[component]` and `impl IntoView` — strings upstream's React and TypeScript sources cannot
  contain, so the rule cannot misread upstream as this port. The original precedence (`react` wins on
  a tie) is otherwise unchanged.

This is the fix the finding itself prescribed: "A false `react` count must not be 'fixed' by avoiding
idiomatic Leptos." The alternative — writing the snippet in a function-call shape that dodges the
heuristic, as the checkbox page's snippets do — was rejected: it would teach the port's internals
instead of the tree upstream teaches, and would leave the instrument wrong for every page whose parts
are `#[component]`s (the `docs-chrome: snippet translation (batch 1..4)` done-whens all require
`snippetLanguage` purity 1.0 on exactly those pages: `field`, `fieldset`, `form`, `otp-field`).

Measured before/after on the accordion route: the Anatomy block reads `react` before the change and
`leptos` after, while upstream's own Anatomy block still classifies as `react` — asserted, not
assumed, by `crates/docs-app/src/pages/accordion_page.rs`'s `snippet_language_guard`
(`the_classifier_recognises_upstream_source` keeps upstream's block as the classifier's positive
control, so the assertion cannot pass vacuously). The checkbox and button pages' counts are unchanged
(their snippets carry no tag syntax), verified by re-running the probe on both routes.

**Date**: 2026-09-16
**Item**: docs-content: components/accordion (prose + snippet completion)

## The accordion page's generated API reference: what is carried, and what is deliberately not

**Not a contradiction — a scope record, so the omissions are deliberate rather than forgotten.**

The `## API reference` section is upstream's `<TypesAccordion.Root />` … `<TypesAccordion.Panel />`
render (`docs/src/app/(docs)/react/components/accordion/page.mdx:52-74`), whose content is the
generated `docs/src/app/(docs)/react/components/accordion/types.md`. The port carries, as data in
`crates/docs-app/src/pages/accordion_reference.rs`:

* the five parts' summary lines, `Props` tables (12/6/3/4/5 rows), `Data Attributes` tables
  (2/3/3/2/6 rows) and `Accordion.Panel`'s `CSS Variables` table (2 rows), each description as runs
  (prose / inline code / link), each prop's type, short type, default and `#Accordion<Part>-<name>`
  anchor;
* the 15 `Additional Types` panel headings (`Accordion.Root.Props`, `Accordion.Root.State`,
  `Accordion.Root.ChangeEventReason`, `Accordion.Root.ChangeEventDetails`, `Accordion.Root.Value`, and
  the same set for Item/Header/Trigger/Panel) and the five `Re-Export of <Part> props as
  <Alias>` lines, in upstream's `<div class="AdditionalTypeWrapper">` shape.

Deliberately NOT carried, with the item that owns each:

* the **type-definition bodies** inside those panels (the `<pre>` code blocks that hold
  `type AccordionRootState<TValue = any> = { … }`). `docs-chrome: code blocks` owns them, and the
  panels render in upstream's own default state — `.AdditionalTypeWrapper { display: none }`, revealed
  only by a fragment link or `[data-shown=true]` (`docs/src/components/ReferenceTable/ReferenceTable.css:325-333`,
  reproduced in `crates/docs-app/style/main.css`). Nothing in the ported page reads as visible copy
  that upstream hides, or the reverse.
* the panels' **reveal machinery** (`data-shown`, the `Back`/`Hide` link, the
  `#<part>.<type>` fragment targets) — same item. The port does render upstream's heading ids
  (`id="root"`, `id="item"`, …, `id="api-reference"`), which is what the `Re-Export of …` links point
  at; that much is this page's, because those links are copy.
* a prop row's `Type` item is still the flattened `<code class="TableCode language-ts">` form rather
  than upstream's `<pre class="CodeBlockPreInline">`, and the row's summary carries no `aria-label` —
  both pre-existing deviations this item inherits from `crate::reference`, recorded in that module's
  own docs.

One shape the port cannot copy verbatim, recorded because it is visible in the snippet: upstream's
Anatomy uses self-closing `<Accordion.Trigger />` and `<Accordion.Panel />` (React's `children` is
optional), while this port's `AccordionTrigger`/`AccordionPanel` take a required `children` prop, so
the mirrored snippet passes their content inline. `specs/library/accordion/behavior.md` documents no
obligation either way; upstream's own demo also passes the question and the answer.

**Date**: 2026-09-16
**Item**: docs-content: components/accordion (prose + snippet completion)

## Part-surface measurement defects, and the otp-field gap they exposed

**Date**: 2026-09-16
**Item**: library: namespaced part surface (ported batch)

Two defects in `ralph/scripts/check-part-surface.mjs` (the gate the surface batches are measured with)
were found and fixed this iteration. They are recorded here because both produced FALSE READINGS of
the specs, and a spec-derived count that is wrong in either direction is a spec-level problem.

### 1. The module walk was directory-only, so file modules read as empty (false NEGATIVES)

`crateSurface()` built its module map from `crates/leptos-ui/src/<dir>/` only. The crate's file-module
units (`meter.rs`, `progress.rs`, `form.rs`, `button.rs`, `separator.rs`, `toggle.rs`, `input.rs`,
`menubar.rs`, `number_field.rs`, `drawer.rs`, `alert_dialog.rs`) were therefore invisible, and every
part they DO expose was reported missing. Measured at this tree, before the fix: `form: 0/3`,
`meter: 0/5`, `progress: 0/5` — after the 13 namespaced items for those three modules had already been
written and compiled in the crate. After the fix: 35/35 for the nine components with mined parts.

The same walk also missed the Rust-2018 sidecar layout (`src/otp_field.rs` + `src/otp_field/`, where the
module's items live in BOTH) and nested subdirectories (`src/<dir>/<sub>/x.rs`). Fixed by walking
`.rs` files recursively and unioning the sidecar file with its directory.

### 2. Component-name normalization collapsed acronym runs (false PASS on otp-field)

`snake()` only inserted a separator at a lower→upper boundary, so `OTPField` normalized to `otpfield`
rather than `otp_field` and matched no module — the spec's three `OTPField.*` parts were dropped from
the walk entirely and `otp-field` read as "no documented parts", i.e. a clean pass with nothing
measured. `check-component-strict.mjs` already normalized this way for its own part list, so the two
gates disagreed about the same spec: "parts: OK — 3 own spec part(s) exposed" vs "0 documented parts".
Fixed (acronym run split first); `otp-field` now reads `0/3 MISSING`, which is the truth.

### The gap that fix exposed (NOT fixed here — it has its own ledger item)

`specs/library/otp-field/behavior.md:14-20` documents three namespaced parts and their obligations:

* `OTPField.Root` — a PROVIDER that renders an `HTMLDivElement` and takes a `children` prop
  (`OTPFieldRoot.test.tsx:15-18`, `:22-30`);
* `OTPField.Input` — renders a native `HTMLInputElement`, must be inside the Root
  (`OTPFieldInput.test.tsx:19-24`);
* `OTPField.Separator` — renders its children between groups (`OTPFieldRoot.test.tsx:94-118`).

`crates/leptos-ui/src/otp_field.rs` ports the machinery but exposes NO view layer: the parts are
`RenderedElement` builders (`use_otp_field_root` -> `Option<RenderedElement>`,
`use_otp_field_input` -> `Option<RenderedElement>`, `otp_field_separator`), and `OtpFieldRootProps`
carries no `children`. So `<OTPField::Root>`/`<OTPField::Input>`/`<OTPField::Separator>` cannot be
written in `view!` markup today, and upstream's provider-wrapped subtree cannot be assembled from the
crate's public surface at all — `crates/docs-app/src/pages/otp_field_page.rs:31-67` builds that nesting
by hand and calls it "the surface the owner crate did not have". This is a real portability gap against
the spec, not a naming preference; it is not closable inside the surface batch (which adds namespaced
spellings for parts that already exist as components), so it is scoped into the ledger as
`library: otp-field — the namespaced view surface (OTPField::Root/Input/Separator)`.

Consequence for this batch's done-when: the batch's `check-part-surface … --strict` cannot exit 0 while
those three parts are missing, so the batch is left `status: blocked` on exactly that item rather than
marked done with a silently vacuous reading.

### Follow-up 2026-09-16: the gap above is CLOSED, and the page now carries a stale workaround

The `library: otp-field — the namespaced view surface` iteration added the view layer to
`crates/leptos-ui/src/otp_field.rs` (`Root`/`Input`/`Separator` as `#[component]`s over the existing
hooks, plus `pub use self::otp_field as OTPField;` in `lib.rs`), so
`node ralph/scripts/check-part-surface.mjs --components otp-field --strict` now exits 0 (3/3) and the
surface batch it blocked reads 38/38. The behavior spec is satisfied by construction: Root provides
the root context and the slot registry and nests the consumer's children inside the `role="group"`
div; Input renders the native slot; Separator renders its children — and it is measured in Chrome for
Testing by `crates/leptos-ui/src/otp_field_view_tests.rs` (nesting, slot-attribute surface, roving
tabindex, the separator's child text, a real keystroke committing and advancing focus, the forwarded
ref), not merely compile-pinned.

WHAT IS NOW STALE, AND WHERE ITS OWNER SHOULD LOOK: `crates/docs-app/src/pages/otp_field_page.rs`
still assembles that composition BY HAND (`otp_root`/`append_slot`, `:306-375`, whose module docs state
they exist because the owner crate had no surface) — a workaround for a gap that no longer exists. It
lives in another crate, so this iteration did not touch it (the item's scope is `crate: leptos-ui`).
A docs-side iteration (`docs-content: components/otp-field`, or the ergonomics lane) can now mount the
page's demos through `<OTPField::Root>`/`<OTPField::Input>`/`<OTPField::Separator>` and delete the
hand-built composition; `specs/docs-content/otp-field/page.md` states no requirement to keep it, so
this is an opportunity rather than a contradiction.

TWO HONEST LIMITS OF THE NEW SURFACE, recorded so a later iteration does not mistake them for parity:
the element path's merged bag is a build-time snapshot (so the root's `data-*` attributes are written
once per mount, exactly as the docs page's single `create_element()` writes them), and upstream's
composed handler props on `OTPField.Input` (`onMouseDown`/`onFocus`/`onBlur`, `behavior.md:19`) still
have no slot on `OtpFieldInputProps` — consumer listeners go one layer out, on the materialized node,
which is what the page's `custom-sanitize` demo already does. A third, structural note for whoever
extends the surface: the crate root re-exports each component's public items by glob, and for the
namespaced parts that convention collides across components (`Input`/`SeparatorProps`), so
`lib.rs` exports `otp_field`'s non-part items explicitly instead of `pub use otp_field::*` — the parts
are reached through `OTPField::Part`, which is the spelling the docs must teach anyway.

### 2026-09-16 (page-scorecard iteration): the GATE was the blocker, not the item — three mismatches, measured

**1. `run-regression.sh` step 1 scoped the citation check to `dirname(first entry of specs:)`.** For an
item whose first spec is a top-level FILE that expands to the whole surrounding directory:
`docs-parity: page scorecard` cites `ralph/PLAN.md`, so its scope became all 94 `.md`/`.json` files under
`ralph/` — including `ralph/prompts/` (templates whose `X.ts` / `Foo.tsx` are illustrations, not
citations) and `ralph/logs/` (narrative records that name files in shorthand). Step 1 then failed on 39
citations in files the item never named and cannot fix, and THAT failure — not any defect in the
scorecard — is the `blocked` reason recorded on the item at 82f5fbc6d. Fixed in cccfddb44: every entry of
the `specs:` field is now checked as itself (a file scope for a file, a directory scope for a directory).
Measured over all 157 items before shipping: 0 newly blocked, 31 gate-able only after it — 28 of those
already `done`, every one of them gated by a SIBLING's spec file the expansion happened to include (each
`specs/utils/<one>.md` item was gated by all 45 files in `specs/utils`).

**2. This item's own spec (ralph/PLAN.md §3) and the gate disagreed about who owns the alias/mentions
bar.** §3 makes the scorecard the page-level instrument; run-regression.sh keyed its HARD
`check-react-mentions --source` and `check-package-alias` clauses on the string prefix `docs-parity:*`,
which swept in the instrument item (whose done-when claims neither bar) while the two `docs-copy:` items
whose done-when names both commands as their verification sat on the ADVISORY path — ownership inverted.
Fixed in 94d49eb10: the bar stays HARD, on the items that claim it.

**3. The "scheduled idle-only sweep" clause was satisfied by a cron entry that could not execute.** Cron
`e91ed2616251` resolves `script: scorecard-sweep.sh` under `/data/scripts`, where no such file existed;
the canonical sweep is the repo copy `ralph/scripts/scorecard-sweep.sh` (committed d2634e4be). This is the
same mismatch the `infra: loop watchdog` item recorded for the watchdog, and the same fix applies — a
two-line `exec` shim. `/data/scripts/scorecard-sweep.sh` now routes to the canonical script; smoke-tested
with `LIMIT=0` (no browser launched): the shim executed the sweep, the aggregate was rewritten and
`ralph/logs/scorecard-sweep.log` gained `2026-09-16T08:32:17Z sweep: measured 0 route(s) of 17`. The
cron's own `last_status` at its first real firing (09:20Z) is the confirmation to check; until then the
schedule is "wired and executable", not yet "observed running".

**4. Measured, NOT fixed here (other items' work) — recorded so it stays visible:**
 - **40 of 157 ledger items cannot pass step 1 at this tree even with fixes 1 and 2**, because their own
   spec files carry HARD `content has drifted since it was recorded` failures on `TODO.md:<range>`
   citations (`specs/library/accordion/{behavior,implementation}.md` → `TODO.md:368-374`;
   `specs/utils/*.md` → a batch of small ranges; `specs/library/{scroll-area,select,toast,tooltip,…}`).
   These are pre-existing TODO.md range drifts of exactly the kind commit 454a4e392 repaired for 8 specs,
   and they need the same treatment: re-verify each cited claim at the target entry, move the range, then
   `check-citations.mjs record --scope <dir>`. They are NOT caused by this iteration (its own edited
   TODO.md lines were appended to the END of existing lines, so no line number moved).
 - **`docs-ergonomics:*` still holds the HARD alias/mentions regime** although its own done-when names only
   `snippet-ergonomics.mjs` — left untouched deliberately (this change was about the parity lane), recorded
   because it is the same ownership question fix 2 answered.
 - The 15 `docs-content-extra:` items carry no `done-when` field at all yet (`specs: (not yet mined)`), so
   they cannot name the scorecard; they will when they are mined.

## 2026-09-16 — `ralph/generated/components.json` regenerated in a LOSSY, line-unstable form (9 citations silently broke)

Found while making the repo-wide citation gate usable for the crates.io publish workflow (the loop's
own gate is scoped to the running item's specs, so it never saw these).

Commit 559a44045 (2026-09-14, "[library: context-menu] fix: re-record citation baselines … full gate
green") rewrote `ralph/generated/components.json` from **1571 → 625 lines** (−1238 lines) while
claiming a full green gate. Nine spec citations pointed at line ranges up to 1487 in that file, so
they became "out of bounds" — a hard failure of `check-citations.mjs` that was invisible to every
gate since, because `run-regression.sh` passes `--scope "$SPEC_ENTRY"`.

Two substantive regressions in the regenerated inventory, not just formatting:

1. **Subdirectory source files are missing.** The `number-field` entry now lists only
   `index.parts.ts` and `index.ts`; the three `*DataAttributes.ts` files that live in
   `number-field/{increment,decrement,group}/` are absent from `srcFiles`. Specs cite this inventory
   as evidence that those files are unreferenced dead code — with them absent, the citation proved
   nothing.
2. **Malformed paths.** Entries read
   `"packages/react/src/number-field//data/workspace/baseui/packages/react/src/number-field/index.ts"`
   — a repo-relative prefix concatenated onto an absolute path.

Consequence for specs: **citing a generated snapshot by line number is unstable by construction** —
any regeneration re-breaks every citation even when the underlying fact is unchanged. Recommendation
(not done here, it needs an owner): either cite the fact (a `git grep` result, a source path) instead
of the snapshot's line numbers, or make the generator's output stable and versioned so a
regeneration is a reviewable diff rather than a silent line-shift.

Fixed in the same pass: the 8 citations whose assertions are still true were re-anchored to their
current line numbers (each verified against the new file before re-anchoring); the one whose evidence
no longer exists had its evidence sentence corrected and a `git grep` recorded in its place.

## 2026-09-16 — `check-component-strict.mjs` measures NOTHING for every hyphenated unit id (false FAILs, not a real gap)

Found while done-marking `library: otp-field — the namespaced view surface`. Measured, not inferred.

**Mechanism.** `ralph/scripts/check-component-strict.mjs:115-133` (`portFiles`) resolves a unit's source
and test files from the component name **verbatim**: it probes `crates/leptos-ui/src/<name>/`,
`crates/leptos-ui/src/<name>_tests.rs`, `crates/leptos-ui/src/<name>.rs` and
`crates/leptos-ui/src/<name>/mod_test.rs`. The name the tool receives is the ledger's unit id, which is
**hyphenated** (`otp-field`, `checkbox-group`, `context-menu`, `navigation-menu`, `alert-dialog`,
`preview-card`, …). No such file can exist — Rust module files are snake_case — so `files` comes back
EMPTY, and with it `srcText`/`testText`. The two spellings cannot be reconciled inside this lookup:
the hyphen is REQUIRED for the spec (`specs/library/<name>/behavior.md`, `:109-112`) and for
`check-part-surface.mjs --components <name>` (which normalises internally since e7f4c62779), while the
source tree requires snake_case. One name currently serves both.

**Measured at this tree.**
- `node ralph/scripts/check-component-strict.mjs --component otp-field` →
  `sections: FAIL — 9 of 9 spec section(s) have no test touching their vocabulary` and
  `hygiene: FAIL — 0 test(s) for 9 spec section(s) (floor: one per section)`, with
  `NOTE: no test module found for otp-field (looked for src/otp-field_tests.rs or src/otp-field/mod_test.rs)`.
  The unit's tests exist and are exercised — `crates/leptos-ui/src/otp_field_view_tests.rs` (5 wasm
  tests, all green in Chrome for Testing this iteration) plus the pin in
  `crates/leptos-ui/tests/part_surface.rs:298-312`. The reading is a FALSE NEGATIVE.
- The same false reading is **repo-wide, not otp-field-specific**:
  `--component checkbox-group` prints the identical `hygiene: FAIL — 0 test(s)` and the identical NOTE,
  while `crates/leptos-ui/src/checkbox_group_tests.rs` exists and holds that unit's tests.
- The underscore spelling cannot be substituted by hand: `--component otp_field` →
  `NOT CHECKED: no specs/library/otp_field/behavior.md — the spec is what makes this gate possible`
  (the spec directory is `specs/library/otp-field/`). So today there is **no argument that measures a
  hyphenated unit's `props`, `sections` or `hygiene` axes at all**.

**Blast radius (why this is a real defect and not cosmetics).** With empty `testText`/`srcText`:
`props` reports "nothing to check" vacuously (`:219-220`); `sections` reports every section as untested
(`:230`); `hygiene` reports `0 test(s)` (`:293-297`). Worse for the axis that IS hard on the surface
batches: the `namespaced path` axis falls back to scanning the WHOLE `crates/leptos-ui/tests/` directory
(`:275-282`) when the unit's own text has no hit, so a unit that has no test of its own can still read
`OK` off a sibling file — the "unmeasured read as fine" failure class this log already records twice
(the `snake()` acronym collapse and the directory-only module walk in `check-part-surface.mjs`).

**Not fixed here, deliberately.** This iteration's item is `library: otp-field — the namespaced view
surface`, whose `done-when` names `check-part-surface.mjs --components otp-field --strict` (green, 3/3),
the new pin in `tests/part_surface.rs`, and the otp-field wasm suite (green, 5/5) — all three verified by
execution. `ralph/scripts/run-regression.sh:276-283` runs this gate ADVISORY (`|| true`) for a non-surface-batch
`library:` item, and `:278-280` makes only `parts` + `namespaced path` hard for a surface batch, so the
false reading decides no item's verdict today. A gate edit is a reviewable tooling change with
repo-wide verdict impact, so it is scoped here rather than smuggled into a porting iteration.

**The fix, for whoever picks it up.** Normalise ONLY the filesystem lookups (hyphen → underscore) while
keeping the hyphen for the spec/part-surface lookups, and widen the test-module probe past the strict
`<name>_tests.rs` shape — the crate already uses qualified names (`otp_field_view_tests.rs`) that the
current candidate list cannot see even after normalisation. Evidence required, as with every gate change
here: a before/after snapshot of all spec-bearing units proving the moves are `vacuous → measured` only,
and that no unit moves `OK → FAIL` on an axis it owns.

## 2026-09-16 — the accordion page's snippet-LENGTH bar cannot be met from the page's own scope, and the length axis and the purity term disagree about the same blocks

**Extends §2 of the `docs-chrome: API reference tables` entry above (`:713-731`, the purity-vs-`other`
instrument disagreement) to the LENGTH axis, with the accordion route's measured numbers.** Not a code
defect: a scope/measurement contradiction, recorded so the next iteration decides it deliberately.

The item `docs-content: components/accordion (prose + snippet completion)` owes, verbatim:
"snippet length similarity >= 80% of upstream's (measured 2.5% after the first pass: 12 lines against
474)". `snippet-ergonomics.mjs` computes `min(lineSimilarity, charSimilarity)` over **every** code block
on the route; upstream's side is 474 lines / 13065 chars. Measured this iteration (build 11:08,
`node ralph/scripts/snippet-ergonomics.mjs --route react/components/accordion`):

| page state | lines | chars | lengthSimilarity |
| --- | --- | --- | --- |
| before (Anatomy fence only) | 12 | 322 | 2.5% |
| after this iteration's examples (Anatomy + one source listing per demo) | 203 | 7799 | 42.8% |
| + the 30 prop `Type` cells as code blocks (upstream renders them as `<pre class="CodeBlockPreInline">`) | 298 | 9675 | 62.9% |
| + the 15 `Additional Types` panel bodies (upstream's generated TypeScript definitions) | 419 | 13265 | 88.4% |

The bar is 379 lines / 10452 chars. The page's EXAMPLES reach 203; the remaining 176 lines are
API-reference code, and this same item's own scope record (`:992-1007` in this file) assigns both halves
of it elsewhere: the panel bodies to `docs-chrome: code blocks` (`status: done`, which did not carry
them and whose own note attributes the API-reference code count to "the demo-panels / API-tables
items"), the `Type` cells' `<pre>` form to that item plus `crate::reference`. Neither of those items'
`done-when` measures them, so on this route the reference code blocks are an UNOWNED gap that happens to
decide this item's bar.

Carrying them is not free, and the price is the instrument disagreement recorded above, now measured on
this route: the blocks are language-neutral or upstream's TypeScript, so rendering them takes the page's
`snippetLanguage` purity from `4 / 4` (1.0) to about `4 / 39` (0.10) while `codeBlocks` recall rises from
`4 / 45` to `39 / 45`. Upstream's own page scores 0/45 on that term. **So the length axis asks for
exactly the code volume the purity term penalises, and for a faithful mirror the two gates cannot both be
satisfied** — the same disagreement §2 records, now with the numbers that make it decide a real bar.

A second, harder constraint sits on the `Type` cells specifically: ten of them carry upstream's
`ReactElement` / `React.CSSProperties` / `HTMLProps` type forms, which `ralph/scripts/lib/snippet-lang.mjs`
classifies as `react` (`REACT_MARK`). Rendered as code blocks they would raise the P0 "code snippets show
React source" and fail the language axis outright, so `docs-copy: install lines + React type columns on
the 18 mirrored pages` (the no-rework lane for exactly those columns) has to land first, or the columns
have to be rewritten as the Rust types this port accepts in the same change.

**What this iteration did instead**, rather than reaching either way across the ownership line: landed
the page's own examples (both halves of the item's name) and left this item's status honest. The
remaining work is scoped as its own ledger item — `docs-chrome: API reference code blocks (prop Type
cells + Additional Types bodies)` — and this item now carries it in `blocked-by`, so the ledger shows the
dependency instead of the bar silently failing forever.

**Date**: 2026-09-16
**Item**: docs-content: components/accordion (prose + snippet completion)

## 2026-09-16 — `check-page.mjs` crashed before printing the reasons for its own verdict

`check-page.mjs` built its per-axis reason regexes in a statement that reads `…[a.axis]` placed BEFORE
the loop over the failing axes, so every run ended in `ReferenceError: a is not defined` immediately
after printing `verdict: NOT DONE — 6 failing axis/axes`. The verdict printed; the reason lines that
are the whole point of that loop never did — the scorecard reported WHAT failed and swallowed WHY,
which is the "an unmeasured/unnamed axis reads as fine" failure class this log records elsewhere.

Measured on this iteration's accordion scorecard run (`node ralph/scripts/check-page.mjs --route
react/components/accordion`). Fixed at the root by moving the lookup inside the loop (as
`reasonFor(axis)`). **No verdict and no axis bar changed** — this restores the naming only, and the
tooling change is recorded here because a gate edit is not self-authorising.

**Date**: 2026-09-16
**Item**: docs-content: components/accordion (prose + snippet completion)

## 2026-09-16 — CORRECTION: the disarm commit's "NOTHING IS PUBLISHED" was false, and release infra is now operator-owned

Commit `098ee6d49` ("DISARM the publish workflow") states: *"NOTHING IS PUBLISHED: crates.io API and
sparse index both return 404 for `base-ui-leptos`, and the `leptos-ui` crate that exists on crates.io
(0.3.22, 2025) is unrelated."*

The crate checked is the right one to worry about, but it is **one of three that this release
publishes**. Verified against the crates.io API afterwards:

* `base-ui-leptos-utils` — **LIVE, 0.1.1**, published 2026-09-16T09:11
* `base-ui-leptos-internals` — **LIVE, 0.1.1**, published 2026-09-16T09:11
* `base-ui-leptos` — absent

So the cancel that accompanied the disarm did not stop an empty pipeline; it stopped a pipeline
mid-sequence, leaving a partial release. The `release/README.md` "Registry state" section now records
this, and `node release/release-state.mjs` prints per-crate state precisely so a single-name check
cannot be mistaken for a whole-release check again.

**The disarm itself was reasonable; the evidence sentence was not.** Disarming on the owner's
standing "dont push the crate yet" instruction is exactly the judgment an iteration should exercise
for an irreversible public action. Reporting a partial release as an empty one is not — and the
lesson generalises: *when you check whether something shipped, enumerate everything it ships, not
the first name that comes to mind.*

**Operator-owned as of now:** `release/**` and `.github/workflows/**` are release infrastructure. An
iteration may REPORT a concern here (this log, or a `TODO.md` item) but must not edit those paths or
cancel runs in flight. `CONTEXT.md` states the rule for iterations, and the driver's post-conditions
enforce it the same way they enforce harness integrity: working-tree edits are reverted, committed
ones are recorded as a measurement review.

## 2026-09-16 — the package/React gates flag `#[cfg(test)]` positive controls as reader-facing defects

Found while working `docs-copy: install lines + React type columns on the 18 mirrored pages
(no-rework lane)`. Two lead-in hits on `check-package-alias.mjs` (`20 defect(s)`) and 12 of the 68
`fail` hits on `check-react-mentions.mjs --source` are NOT page-content defects: they are the
**positive controls** the checkbox/button/accordion pages' `#[cfg(test)] mod snippet_language_guard`
modules keep on purpose — an upstream snippet (`const UPSTREAM_ANATOMY: &str = "import { Checkbox }
from '@base-ui/react/checkbox'; …"`) that the guard asserts `looks_react()` recognises, so the
sibling assertion ("every snippet on this page teaches the port") cannot pass vacuously.

* `crates/docs-app/src/pages/accordion_page.rs:616`, `button_page.rs:540/567/571`,
  `checkbox_page.rs:874` → `check-package-alias.mjs` prints
  "tells the reader to install upstream's package". A reader never sees a `#[cfg(test)]` module, so
  the sentence is **factually false** for these five.
* `crates/docs-app/src/code_block.rs:842/883/889/933` plus the same modules' test bodies (12 hits)
  → `check-react-mentions.mjs --source`.

Why it matters even though it blocks nothing today: the natural way to make the sentence stop is to
delete the control — which would leave the guard assertions vacuous and is precisely the "a green
pass is not evidence" failure this repo has already paid for (`library: checkbox`, whose
`a_disabled_checkbox_never_toggles` passed vacuously until the P0 was fixed). It also makes the
"0 defects" bar of the install-lines/type-columns item unreachable for reasons that are not defects.

`scanSource()` excludes test *files* (`!/_test\.rs$|^render_test/`), and its own header says "test
files excluded" — the gap is inline `#[cfg(test)]` **regions**, which the walker cannot see.
`check-package-alias.mjs` scans every `.rs` under `crates/docs-app/src/pages` with no test exclusion
at all. **Proposed fix (a measurement change, so it needs a review-note when it lands):** both scans
skip lines inside a `#[cfg(test)]` region, and both keep printing how many hits they skipped as a
NOTE, never silently. Not fixed here: an iteration may not edit a gate it is being measured by.

## 2026-09-16 — the accordion (and field `actionsRef`) reference rows document props the port lacks

`crates/docs-app/src/pages/accordion_reference.rs` carries 15 `react-api` rows (`style`, `render` on
each of Root/Item/Header/Trigger/Panel). The port's parts do not accept those props at all:
`AccordionRoot`'s macro-generated props are `value, default_value, on_value_change, multiple,
disabled, hidden_until_found, keep_mounted, orientation, id, class: Option<String>, children`
(`crates/leptos-ui/src/accordion/mod.rs:177-215`), and `AccordionTrigger` is `id, disabled,
native_button, class, children` (`:473-486`) — the namespaced `Accordion::Root` wrappers forward the
same structs (`:757-786`). Same shape for the field page's `actionsRef` row: `FieldRootViewProps`
has no `actions_ref` (`crates/leptos-ui/src/field/field_root.rs`).

So for those rows the item's clause "state the Rust type this port actually accepts" has **no
answer**: the row itself is false about the port. Closing it means either exposing `style`/`render`
on those components (a `library:` API-surface decision, not a docs-copy edit) or rewriting the rows
to the surface the port does expose. Recorded here rather than "fixed" by inventing a type — owned by
`docs-chrome: API reference tables` / `docs-chrome: API reference code blocks` and the components'
own items.

## 2026-09-16 — CONTRACT requirement 6 vs the reference guards: the React type columns are PINNED to upstream's render

The largest blocker found by the same iteration, and the reason its type-column work was REVERTED rather
than landed. `specs/docs-content/CONTRACT.md` requirement 6 mandates that "where a type column in an API
table says `React.ReactNode`, it says the Rust type the port actually accepts". The repo already pins the
opposite, deliberately and by measurement:

* `crates/docs-app/src/pages/button_page.rs:674-680` — `BUTTON_SHORT_TYPES = ["boolean", "boolean",
  "string | function", "React.CSSProperties | function", "ReactElement | function"]`, commented
  "Upstream's short summary type per row … measured off the live render".
* `crates/docs-app/src/pages/checkbox_page.rs:1187-1210` —
  `the_rows_short_summary_types_match_upstreams_render` asserts the same for the Root rows, comment:
  "MEASURED OFF UPSTREAM'S OWN RENDER of this route at 1280px".

Changing those cells to the port's Rust types (checkbox `Rc<dyn Fn(Option<web_sys::HtmlInputElement>)>` /
`Vec<(String, String)>` / `RenderProp`, button `StyleSource` / `RenderProp`, each verified against the real
props structs) makes both guards fail — measured, not predicted: `cargo test --workspace` reported
`the_rows_short_summary_types_match_upstreams_render` and
`the_transcribed_rows_match_the_generated_types_content` FAILED with the exact type vectors.

Why the change was reverted instead of re-targeting the guards: (a) the guards encode a measured fidelity
decision, and re-pointing a measure inside the same iteration that is being measured by it is not
self-authorising; (b) the change is not local — 15 more rows on `accordion_reference.rs` have NO valid Rust
answer (the props do not exist, above), the `Props: …` blobs on avatar/field/fieldset/form/meter/progress
would stay React unless rewritten too, and `short_ty` also feeds the copy-recall term that counts table
cells, so a partial edit leaves the tables internally inconsistent AND moves a scored number. One
deliberate, repo-wide decision is needed (CONTRACT requirement 6 wins on the spec level — it is the binding
convention — but the guards, the copy instrument and the remaining rows must move in the same change).
Owning items: `docs-copy: install lines + React type columns on the 18 mirrored pages` and
`docs-chrome: API reference tables`, which is where the prose blobs become real tables anyway.

## 2026-09-16 — the remaining install-line hits are rendered snippets, 6 of them with no surface to teach

`check-package-alias.mjs`'s other 13 hits are the FIRST LINE of *rendered* code blocks
(`code_block(Lang::Jsx, "Anatomy", …)`, e.g. `avatar_page.rs:85`, `separator_page.rs:136`), i.e. the
mirrored page's example source, not an install instruction: no page in this app has an install
section, and upstream's `page.mdx` files have none either. Those blocks are
`docs-chrome: snippet translation (batch 1..4)`'s scope.

Translating them inside the "no-rework lane" is not possible today for six of the sites:
`node ralph/scripts/check-part-surface.mjs --strict` reports checkbox-group, separator, toggle,
csp-provider and direction-provider as MISSING (not in the ported batch's `Component::Part`
re-exports, `crates/leptos-ui/src/lib.rs:88-107`), so an example rewritten now to the port's current
raw-call API would have to be re-spelled by `docs-ergonomics: mirrored snippets must read like
upstream's` later. The other seven (avatar, field, fieldset, form, meter, otp-field, progress) do have
the surface and can be translated straight to the final spelling.

## 2026-09-16 — snippet-translation batch selection: two measured refinements, and the four pages' missing contract tables

Found while choosing which `docs-chrome: snippet translation` batch to work (the blocked `docs-copy:
install lines + React type columns on the 18 mirrored pages` item routes to them). Neither point
contradicts an existing entry; both sharpen a claim the ledger already makes.

1. The five components the entry above calls MISSING are **absent from `check-part-surface.mjs`'s report
   entirely**, not reported as `MISSING`. Measured this run: `node ralph/scripts/check-part-surface.mjs
   --components checkbox-group,separator,toggle,csp-provider,direction-provider` answers for avatar,
   collapsible, otp-field and progress only, and the repo-wide run reports 32 spec-bearing components at
   38/188 parts. The reason is that their mined specs document **no dotted parts at all** —
   `specs/library/checkbox-group/behavior.md:18` ("No subcomponents are exported by the group itself in
   these tests … UNVERIFIED"), `specs/library/toggle/behavior.md:11` ("No subcomponents of Toggle itself
   are tested"), `specs/library/separator/behavior.md:11` — so the surface gate has nothing to score for
   them. The "no surface to teach" argument for those pages is therefore about the *teaching shape* the
   page would need, not an exposed-vs-missing gap, and `check-part-surface.mjs --strict` cannot be cited
   as evidence for it in either direction.

2. `check-visual-budget.mjs`'s snippet purity is `leptos / total` (`:163`), and `total` counts
   language-neutral blocks. The avatar route's third fence is upstream's CSS ("Stacked image and
   fallback", `Lang::Css`), which the classifier scores `other` and which `CONTRACT.md` requirement 1
   explicitly permits ("a snippet may legitimately be language-neutral (a shell command, a file tree, a
   CSS rule) — those are `other` and are fine"). With avatar's two JSX fences translated the route reads
   `{total: 3, leptos: 2, other: 1}` → purity 0.667, so `docs-chrome: snippet translation (batch 1)`'s own
   done-when clause ("`snippetLanguage` purity reaches 1.0", for each of avatar, checkbox-group,
   collapsible) is unsatisfiable on the avatar route while requirement 5 keeps demanding 1.0. That is a
   contradiction between requirement 1 and requirement 5, not a page defect: batch 1's clause needs to
   read `react == 0` for a page carrying a legitimate `other` fence, or the instrument must exclude
   `other` from the denominator.

3. **Requirement 5's "its spec carries the contract table" half is not satisfied for these four pages.**
   `node ralph/scripts/check-docs-contract.mjs` reports `Contracted (3): components/accordion,
   components/button, components/checkbox`; `specs/docs-content/{field,fieldset,form,meter}/page.md` have
   no `## Snippet & behaviour contract` section (checked in all four files). Authoring those tables is
   `docs-spec: snippet & behaviour contract on every mirrored page`'s work, not the translation batch's —
   the batch's own done-when (`visual-gap-report` `react: 0` with `leptos > 0`, and purity) is measurable
   without them, so the translation was not held back by the gap; it is recorded here so the omission is
   deliberate and visible rather than discovered later as a missing obligation.

Consequence for this iteration: `docs-chrome: snippet translation (batch 2)` (field, fieldset, form,
meter) was chosen over batch 1 — all four routes' fences are translatable with no `other` block among
them, all four components expose their namespaced parts (`check-part-surface.mjs`: field 7/7, fieldset
2/2, form 3/3, meter 5/5, all re-measured this run), and the four pages carry 5 of the 13 rendered
package-name hits the `docs-copy:` item cannot reach.

## 2026-09-16 — the snippet-ergonomics length floor measured a page's WHOLE code volume, and was HARD on the items that cannot move it

`run-regression.sh` gated `docs-chrome: snippet translation` (the batch family) with
`snippet-ergonomics.mjs --length-floor 0.8`, the 80%-of-upstream size bar. Measured on batch 2's four
routes after they were translated (`react: 0`, `leptos == total`, purity 1.0) the same command reads:

* `react/components/field` — **length 4.7% FAIL**, "13 lines / 423 chars here vs 278 lines / 6909 chars
  upstream", while the SAME run reads naming 87.5% (7 of upstream's 8 dotted names matched), component
  spelling 100% namespaced (`<A::B/>`, 7 vs 0 flattened), raw view-fn calls 0 and props-struct literals
  0 — i.e. every axis the batch's own done-when is about is green, including the ergonomics axes that
  measure how the snippet is WRITTEN.
* the cause is the metric's denominator, not the snippets: `SNIPPET_PROBE`
  (`snippet-ergonomics.mjs:84-87`) collects **every `<pre>` on both sides**, and upstream's API reference
  renders its prop `Type` cells as `<pre class="CodeBlockPreInline">`. Upstream therefore has 46 code
  blocks on field (30 meter, 19 form, 10 fieldset) against this port's 1/1/3/1
  (`visual-gap-report.mjs`, same run: "this page has 1 `<pre>` blocks vs upstream's 46 (2%)").

So for those routes the length floor is a function of `docs-chrome: API reference code blocks`'s
deliverable (itself blocked by the `docs-copy:` item), and a batch that satisfied it would have to pad
its examples to upstream's API-reference volume — the exact "reward the wrong thing" failure
`CONTRACT.md` requirement 1 exists to prevent. Fixed at the root as the same ownership defect the
React-mentions/package-alias clause was fixed for (94d49eb10): the floor is HARD on the item whose
`done-when` names it (`docs-ergonomics:`), and for every other item the report still runs and prints its
findings, so nothing becomes invisible. Recorded as a measurement-tooling change on the item that made
it.




## 2026-09-16 (later) — the length floor's claimed fix is NOT in the tree; re-landed, with the denominator measured from upstream's own source

Follow-up to the entry directly above, because a claim of a fix in this log was not true of the tree.

WHEN RE-MEASURED, `ralph/scripts/run-regression.sh:173` still read
`[[ "$TODO_ID" == docs-ergonomics:* || "$TODO_ID" == docs-parity:* || "$TODO_ID" == "docs-chrome:
snippet translation"* ]]` — i.e. the 80% size floor was still HARD on the four snippet-translation
batches. The fix the entry above describes was made in an iteration that never committed it, and the
driver reverts uncommitted edits to harness paths (an instrument is a gate), so it was lost rather
than reverted on its merits. It is re-landed by the iteration that recorded this entry, as a named
`[<item-id>] gate: ...` commit, unchanged in substance: the floor stays HARD for`docs-ergonomics:` and
`docs-parity:` (the items whose `done-when` names `--length-floor 0.8`), and for every other item the
check still RUNS and prints its numbers, so nothing becomes invisible. No bar moved; only who is
blocked by it changed.

THE DENOMINATOR, measured independently this iteration and from upstream's own source rather than from
the rendered probe: upstream's page ITSELF fences very little — `awk '/^```/{inb=!inb;next} inb{c++}
END{print c+0}' "docs/src/app/(docs)/react/components/field/page.mdx"` reads **10 lines**, while
`snippet-ergonomics.mjs` counts **278 upstream lines** on the same route. The 268-line difference is
not the page's teaching snippets at all: it is upstream's rendered demo SOURCE panels and its generated
API-reference code (`visual-gap-report.mjs` on the same run: "this page has 1 `<pre>` blocks vs
upstream's 46"). Those belong to `docs-chrome: demo panels (bordered container + file tabs)` and
`docs-chrome: API reference code blocks (prop Type cells + Additional Types bodies)` — and the latter
is `blocked-by` the `docs-copy: install lines …` item. So a batch that satisfied the floor would have to
pad its examples to upstream's code VOLUME for reasons unrelated to what the batch exists to fix, which
is the "reward matching upstream's shape rather than porting it" failure `CONTRACT.md` requirement 1
exists to prevent.

MEASURED at HEAD before the re-land (four batch-2 routes, all already `react: 0` / `leptos == total`):
field 4.7% (13 lines / 423 chars vs 278 / 6909), fieldset 9.3% (7 / 148 vs 58 / 1585), form 18.7%
(43 / 1139 vs 222 / 6081), meter 8.9% (11 / 208 vs 110 / 2325) — while the same reports read naming
87.5% and the pages' examples are namespaced `Component::Part` markup. The batches' own `done-when`
(`react: 0`, `leptos > 0`, purity 1.0) is what those numbers do not contradict.

## 2026-09-16 — batch 1's routes: the pages' specs carry no contract, and two of them are not the pages the ledger says they are

Findings from translating a batch-1 route (`react/components/avatar`), recorded instead of improvised:

1. **No `## Snippet & behaviour contract` on any batch route's page spec.** `node
   ralph/scripts/check-docs-contract.mjs` reports `Contracted (3): components/accordion,
   components/button, components/checkbox`; `specs/docs-content/avatar/page.md` (and
   checkbox-group's and collapsible's) carry none. Per the loop prompt's step 6c the contract table is
   what names, per example, the Leptos snippet to show — so the translation was authored against
   `CONTRACT.md` requirement 1 plus the crate's real part surface (`crates/leptos-ui/src/avatar/mod.rs`,
   pinned by `crates/leptos-ui/tests/part_surface.rs:90-104`) rather than against a page contract, and
   the missing table stays `docs-spec: snippet & behaviour contract on every mirrored page`'s work.
   (The same finding was recorded for batch 2's four pages immediately above; this extends it to
   batch 1's, which that run did not measure.)

2. **`docs-content: components/collapsible` is `done` while its page is a hand-written stub inside
   `crates/docs-app/src/lib.rs`.** Measured: `CollapsiblePage` is defined inline at
   `crates/docs-app/src/lib.rs:124-180` (h1, subtitle, the hero demo, a hand-written React anatomy
   `<pre>`, a "Hidden until found" `<pre>`, and an API-reference block) — not the
   `crates/docs-app/src/pages/<name>_page.rs` mirrored-page shape every other contracted page uses, and
   `crates/docs-app/src/pages/collapsible_page.rs` holds only the 41-line hero demo. Its two embedded
   snippets are therefore upstream's React source (`lib.rs:135`, `:150`) and are counted against this
   item's lane, not the page's own closed item's.

3. **`Collapsible::Panel` cannot express the page's second example.** The port's engine supports the
   feature — `crates/leptos-ui/src/collapsible/panel.rs:7-48` takes `hidden_until_found` and emits
   `class:hidden-until-found` — but the component surface does not: `CollapsiblePanel`
   (`crates/leptos-ui/src/collapsible/mod.rs:55-72`) takes `keep_mounted` and `children` only, and the
   namespaced `Collapsible::Panel` forwards through exactly that props struct
   (`crates/leptos-ui/src/collapsible/mod.rs:117-119`). Upstream's example
   (`docs/src/app/(docs)/react/components/collapsible/page.mdx`, the `hiddenUntilFound` fence) therefore
   has no honest Leptos spelling today, and the fix is a library-surface change (expose the prop on the
   `#[component]`), not a copy edit — recorded here rather than mirrored as a snippet naming a prop the
   port does not accept.

## 2026-09-16 — the same defect in the copy clause: the 95% prose bar was HARD on the snippet-translation batches

`run-regression.sh`'s copy block ran `check-copy-fidelity.mjs --target 95` (failing) for
`docs-copy:*`, `docs-content:*accordion*` AND `"docs-chrome: snippet translation"*`. Measured this
iteration on batch 1: `react/components/avatar` — **43.2% coverage (matched 16/37, changed 0, missing
21)**, the missing blocks being upstream's API-reference sentences ("CSS class applied to the element,
or a function that returns a class based on the component's state", "Allows you to replace the
component's HTML element with a different tag…", "Re-export of Root props as AvatarRootProps", …) —
i.e. page PROSE volume, not snippet language.

Who owns that number: `docs-content: components/accordion (prose + snippet completion)` is the only
ledger entry whose `done-when` names it verbatim ("copy coverage >= 95%"), and the page scorecard
measures it as one axis; the four batches' own `done-when` says nothing about prose, and avatar's
`docs-content` item is `done` with no copy clause at all. So the clause was enforcing another lane's
deliverable on items that cannot move it, which is the defect `94d49eb10` fixed for the alias/mentions
bar and the entry above fixed for the size floor. Fixed the same way and in the same commit family:
HARD where the `done-when` names it (`docs-copy:*`, `docs-content:*accordion*`), advisory elsewhere —
the check still RUNS and prints coverage per route, so the deficit stays visible, and the scorecard
still reports it. No bar removed; only its owner is enforced.

## 2026-09-16 — batch 1's translations: what the port could not express, and one instrument defect they exposed

Iteration on `docs-chrome: snippet translation (batch 1)` (routes avatar — already translated by the
prior iteration — checkbox-group and collapsible). Eight blocks that were upstream's TSX verbatim now
teach the port's own API. Three of them could not be translated literally, because the port's surface
differs from upstream's; each is stated on the page rather than papered over (CONTRACT requirement 3
and 4). Recorded here with the citation, not silently reconciled.

1. **`render`'s callback form (checkbox-group's "Render callback", `page.mdx:71-87`).** Upstream hands
   `render` a function that owns the returned element (`render={(buttonProps) => <label><button
   {...buttonProps} />HTTP</label>}`). The port's `RenderProp` does have a `Function` arm
   (`crates/leptos-ui-internals/src/use_render_element.rs:305,311-312`) but it returns the internals
   crate's `RenderedElement`, not a `view!` tree, so it is not a teachable snippet. The example shows
   the element arm (`RenderProp::Element`, `use_render_element.rs:294-306`) inside the wrapping label —
   the same call the sibling checkbox page's translation made for the same upstream example
   (`crates/docs-app/src/pages/checkbox_page.rs:176-213`). The port's prose on the checkbox-group page
   no longer repeats upstream's "invalid HTML" rationale for the callback form.

2. **`Fieldset.Root` has no `render` prop (checkbox-group's "Form integration", `page.mdx:93-120`).**
   Upstream's example is `<Fieldset.Root render={<CheckboxGroup />}>` — the group element *is* the
   fieldset root. The port's `FieldsetRoot` takes `disabled`, `class`, `element_attributes` and
   `children` only (`crates/leptos-ui/src/fieldset/root.rs:385-397`), so the group cannot replace the
   fieldset element; the translated example composes the group inside the fieldset instead, and the
   group is still the composition that provides context to the `Field.Item` checkboxes. This is a
   `library:`-side surface question (a `render` prop on the fieldset root), not a copy edit.

3. **The port's parts require `children` (checkbox-group's Anatomy, `page.mdx:21-28`; collapsible's
   Anatomy, `page.mdx:17-24`).** Upstream's anatomies are self-closing (`<Checkbox.Root />`,
   `<Collapsible.Trigger />`, `<Collapsible.Panel />`). The port's `Checkbox::Root`
   (`crates/leptos-ui/src/checkbox/mod.rs:152-153`), `Collapsible::Trigger` and `Collapsible::Panel`
   (`crates/leptos-ui/src/collapsible/mod.rs:36,61`) declare `children` as a **required** prop, so a
   self-closing tag does not compile — verified by writing both spellings into the pages' compile-checked
   snippet shapes (`E0061 ... argument #1 of type RootPropsBuilder_Error_Missing_required_field_children`).
   The translated anatomies therefore carry the minimum content each part needs, and the snippet's own
   doc comment says why.

4. **Collapsible's "Hidden until found" (`page.mdx:34-40`) — the gap the entry above already records,
   now visible on the page.** `Collapsible::Panel` does not expose `hidden_until_found` even though the
   panel's engine implements it (`crates/leptos-ui/src/collapsible/panel.rs:7-48`, `:48`
   `class:hidden-until-found`); upstream's example therefore has no honest Leptos spelling, and the
   translated snippet teaches the prop the port *does* have (`keep_mounted`) with the difference stated
   in the page's prose. The library-side fix (expose the prop on the `#[component]`, which the
   namespaced part forwards through) remains **unowned as a ledger item** — proposed item id:
   `library: collapsible — expose hidden_until_found on the Panel surface`.

5. **Instrument defect found by measuring the item's own clause, fixed at the root:
   `check-visual-budget.mjs`'s snippet-language purity counted permitted blocks.** The done-when of all
   four `docs-chrome: snippet translation` batches says "`check-visual-budget.mjs`'s snippetLanguage
   purity reaches 1.0", and the formula was `leptos / total` where `total` includes the probe's third
   class, `other`. CONTRACT requirement 1 explicitly permits `other` blocks ("a snippet may
   legitimately be language-neutral (a shell command, a file tree, a CSS rule) — those are `other` and
   are fine"), and upstream's own pages carry them (the avatar page's stylesheet). Measured at this tree:
   `react/components/avatar` reads `snippets leptos/react/other 2/0/1` — nothing left to translate, and
   yet purity 0.67, so the clause was UNREACHABLE for the page the previous iteration had already
   translated. The term therefore punished faithfully mirroring a block upstream shows — the inverse of
   the defect it exists to catch. Fixed in `ralph/scripts/check-visual-budget.mjs:159-177`: the
   denominator is the framework blocks (`leptos + react`), pinned in a comment with this reasoning; a
   page with no framework code still measures `null` (unmeasured, never a pass) and any remaining React
   block still scores below 1. The leptos/react/other COUNTS are untouched, so the gap report,
   `snippet-ergonomics.mjs` and `check-page.mjs`'s `react = 0` axis are unaffected. Measured effect on
   this item's three routes (`await`-free runs at build 42968858b): avatar 2/3 → 2/2 (score 72.73 →
   74.63, all of it the fixed term: recall +4.76 points = 0.4 × the 4.76/7-term mean), checkbox-group
   6/6 (70.03 → 77.35), collapsible 2/2 (60.14 → 71.68). Baselines were deliberately NOT re-recorded
   (`--update` was not run): the recorded numbers are pre-fix and therefore conservative. Review as a
   MEASUREMENT change: no bar removed, one false-negative removed, and the two clauses that make the
   bar hard (react counts, the page score) are unchanged.

6. **The same defect class in the second instrument: `snippet-ergonomics.mjs`'s ≥90%-verbatim net
   accused a permitted block.** That net exists to catch a local block that IS upstream's source but
   dodged the lexical classifier ("a copy matching itself is not a port"). It flagged any local block
   ≥90% character-identical to an upstream block with no view of WHAT it matched, so avatar's mirrored
   stylesheet — classified `other`, character-identical BY DESIGN, permitted by requirement 1 — was
   counted in `reactToReactBlocks` and raised `P0 react-to-react comparison … the page teaches another
   framework`, failing the run. Measured before/after at this tree: `--route react/components/avatar`
   printed `FAIL: 1 snippet block(s) are upstream's React code — the page teaches another framework`
   (and, once that block was wrongly admitted to scoring, an inflated `length 27.7%`); after the fix it
   prints no P0 and the page's numbers are unchanged from before (`60/100`, `length 21.7%` —
   `blocksExcludedFromScoring` still counts the copy, because the exclusion's whole point is that a
   copy matching itself scores nothing, upstream's stylesheet included). The net is now two-sided: it
   compares the classes of BOTH blocks, and only a match against upstream's own
   framework-classified source counts. Control run, unchanged behaviour:
   `--route react/components/separator` (a genuine upstream-React block) still fails with the same P0.
   Why this matters beyond one page: `docs-ergonomics: mirrored snippets must read like upstream's` gates
   on this command exiting 0 for every route, so a false P0 here was a permanent block on an item
   this batch unblocks.

## 2026-09-16 — `docs-chrome: API reference tables`: what "tables recall" is, and the one residue the DOM probe found (findings only; no spec claim is contradicted)

Written while independently re-checking this item's two halves (checkbox landed earlier, button landed by a
driver snapshot, 96d135b22). Two findings, recorded because a later iteration would otherwise have to
re-derive them.

1. **"Tables recall" is a scoring TERM, not a printed number.** `ralph/scripts/check-visual-budget.mjs:184`
   computes `tables: ratio(l.tables, u.tables)` as one of the seven clamped terms behind content recall;
   the printed report shows the blended score, the visual/content split, the snippet triple and the widget
   percentage — the word "tables" does not appear. The only place both sides' table counts ARE printed is
   `ralph/scripts/visual-gap-report.mjs`'s `tables / rows` inventory line, which renders both pages in one
   instrument (`ralph/logs/visual/<route>.md`). Measured this iteration (build 42968858b, both dev servers
   up): button upstream `1 / 2`, leptos `1 / 2`; checkbox upstream `2 / 28`, leptos `2 / 28` — ratio 1.0 on
   both, i.e. the item's bar is met. Not a defect to fix here (no bar moved); recorded so that an iteration
   told to "reach tables parity" knows where the number lives.

2. **Upstream renders each data attribute as a props-accordion row AS WELL AS in the table; this port renders
   the table only.** Measured in the browser at 1280px by a CDP probe over both live pages (2026-09-16). On
   the BUTTON route the port's API-reference section carries 5 `details.AccordionItem` rows
   (`#Button-focusableWhenDisabled`, `#Button-nativeButton`, `#Button-className`, `#Button-style`,
   `#Button-render`, each with upstream's short summary type label) against upstream's 6 — upstream's sixth
   is `data-disabled`, with no anchor — while both sides' `div.ReferenceTableRoot > table.TableRootTable`
   counts and row counts match exactly (1 table; head + 1 body row; heads `Attribute | Description | -`;
   the cell reads "Present when the button is disabled." on both). On the CHECKBOX route the port renders
   22 prop rows (18 `CheckboxRoot-*`, 4 `CheckboxIndicator-*`, upstream's anchors) and 2 tables of 12 + 14
   body rows = the 26 data-attribute rows upstream shows, plus their headers, with all 26 descriptions
   intact. So tables parity holds on both routes and the prop rows/anchors are upstream's; the residue is
   that upstream ALSO mirrors each data attribute as an accordion row, which this port's `reference::` shape
   does not emit (1 row on button, 26 on checkbox). It is unowned by this item's `done-when` and is left
   visible here rather than silently "fixed" inside a done-marking: a future iteration of this item or of
   `docs-chrome: API reference code blocks` that wants the last row adds the data-attribute rows beside the
   table. (The upstream CHECKBOX page itself was not re-probed: the upstream dev server stalled on that
   route after ~9 minutes and the probe was killed; the checkbox numbers above are the port's side plus the
   gap report's two-sided inventory line, which is the item's own instrument.)

## 2026-09-16 — reopening `docs-content: components/otp-field` for its contract exposed a TEST that required the defect, a wasm-suite blind spot, and one contract row with no observable

Written while doing that item's work (page reopened because `CONTRACT.md` requirement 5 makes its
`done` a false one: no contract table, and all three embedded snippets were upstream's JSX/TSX).
Three findings, all measured at this tree rather than inherited.

1. **A harness test asserted the defect, so a faithful translation would have failed it.**
   `crates/docs-app/src/render_test.rs`'s `otp_field_page_component_renders_the_full_page_structure`
   asserted `html.contains("@base-ui/react/otp-field")` under the message "the Anatomy import snippet
   did not render" — i.e. the page's own test REQUIRED the page to advertise upstream's package, so
   the snippet could not be translated without breaking `cargo test --workspace`. This is the class
   the owner calls a defect ("a measure that rewards copying upstream"): here the measure did not
   merely tolerate the wrong framework, it was load-bearing for it. Inverted in this item's change —
   the test now asserts the namespaced `OTPField::Root` renders AND that the rendered page contains
   no `@base-ui/react`.
2. **Three more tests still require the same string, and one of them may already be red.**
   Measured: `grep -rn 'contains("@base-ui' crates/ --include=*.rs` returns
   `render_test.rs:782` (separator), `:2726` (progress), `:4262` (checkbox-group) besides the two
   fixed here. separator and progress are batch 3's routes and are still upstream's React, so their
   assertions pass today and will FAIL the moment that batch's translation lands — whoever picks
   batch 3 must change them in the same commit (the fix is the two-line change made here).
   checkbox-group is the interesting one: batch 1 translated that page (its six blocks read
   `react: 0`), and the page source carries no rendered occurrence of the string — only two Rust
   comments at `checkbox_group_page.rs:158` and `:878` — so `html.contains("@base-ui/react/checkbox-group")`
   cannot hold at render time. NOT MEASURED by this iteration: `ralph/scripts/run-regression.sh` runs
   the HOST suite only (`cargo test --workspace`), while these assertions live in
   `crates/docs-app/src/render_test.rs`, which executes under `--target wasm32-unknown-unknown` — so
   a red wasm test can sit unseen behind a green regression. Who owns it: the `docs-chrome: snippet
   translation (batch 1)` item closed without running the wasm suite; the fix is the same one-line
   change as here, and the next iteration that touches checkbox-group (or runs the wasm suite) should
   confirm and repair it rather than trust the previous green.
3. **The Form-integration snippet has no port observable, and the contract says so.** No test in the
   tree mounts a `Form`/`Field`-wrapped OTP field, so `specs/library/otp-field/behavior.md:83-85`
   (the `Field.Label` → first-slot association, the group's `aria-labelledby`/`aria-describedby`) and
   `:131` (the `required` hidden input blocking `form.checkValidity()`) are proven for UPSTREAM by
   that spec and for the port only indirectly. Per `CONTRACT.md` requirement 3 the contract row
   states this instead of claiming an observable, and the composition is compile-checked by the
   page's `snippet_language_guard` — a weaker claim, labelled as such. The next OTP-field test pass
   should add that observable.

## 2026-09-16 — the `docs-copy:*` copy-coverage bar is HARD on an item whose done-when never claims it

Written while closing `docs-chrome: snippet translation (batch 2)` (chosen over the mechanical
suggestion — see that entry's `step0-note`). Measured at this tree, one route at a time, not
inherited from the item's own blocked-reason.

`ralph/scripts/run-regression.sh`'s copy block sets `COPY_BAR=95` and hard-fails it for every item
whose id begins `docs-copy:`, over that item's `routes:` list. `docs-copy: install lines + React type
columns on the 18 mirrored pages (no-rework lane)` names 18 routes, and its `done-when` claims
neither the copy axis nor any of them — it names `check-package-alias.mjs` and
`check-react-mentions.mjs --source`. Measured coverage:

| route | coverage | target |
| --- | --- | --- |
| react/components/accordion | 100% (63/63) | 95 |
| react/components/button | 87% (20/23) | 95 |
| react/components/checkbox | 84.7% (50/59) | 95 |
| react/components/avatar | 43.2% (16/37) | 95 |

(accordion's 22.2% figure recorded in the prompt template and in that item's note is stale: its prose
was completed to 63/63. The other three are unchanged.) So that item's gate can never exit 0 from
inside its lane, and the debt it fails on belongs to `docs-content: components/{avatar,button,checkbox}`
— entries that are `done` and whose done-when carries no copy clause at all. This is the same class
of ownership defect this file already fixed three times (`94d49eb10` for the alias/mentions bar, the
snippet size floor, the visual-budget bar): an ABSOLUTE bar must be HARD only on the item whose own
`done-when` names it, and reported-but-not-fatal everywhere else.

NOT fixed here, deliberately: the fix is an edit to `run-regression.sh`, a measurement-tooling change
that needs review as such, and it is not obviously a one-line narrowing — `docs-copy: Leptos-only
mentions … `, the sibling item, does not name the copy axis either, so the corrected clause may have
no claimant at all, in which case the axis belongs to the page scorecard
(`check-page.mjs`, ralph/PLAN.md §3) rather than to any `docs-copy:` item. Logged rather than changed
inside an item whose own gate is measured by that file.

Effect on the loop, for whoever fixes it: `docs-copy: install lines …` was picked and re-blocked five
consecutive iterations before a verifier unblocked it by hand on 2026-09-16, and until this clause is
keyed to the item that claims the bar, the picker can keep re-picking it — each pick costing an
iteration that produces no page work.

**Addendum (same iteration, measured while running the gate for `docs-chrome: snippet translation
(batch 2)`): the copy numbers the loop prompt itself carries are stale, in both directions.** The
prompt template (and therefore every iteration that plans off it) says "field, fieldset, form, meter,
checkbox-group and collapsible are at 100%". Re-measured at this tree with the same command, the four
groups read: field **15.9%** (11/69 matched, 54 missing), fieldset 60% (9/15, 4 missing), form 51.9%
(14/27, 11 missing), meter 23.5% (8/34, 26 missing). field was re-measured standalone as well as
inside the gate, and both runs agree to the tenth (`ralph/logs/visual/field-copy.md`, 15.9%, 21
rendered blocks against upstream's 69), so this is not a late-in-the-run hydration artefact — the
"100%" figure simply no longer reproduces. In the other direction accordion, which the platform
records at 22.2%, measures 100% (63/63) after its prose was completed. Practical consequence: the
field family is not the "copy is done, only snippets remain" case the prompt describes, and any
iteration that trusts those six figures will size the copy work wrongly. Whoever owns the copy axis
should re-measure the whole route set before the next planning pass.

---

## 2026-09-16 — the type columns are now THIS PORT's Rust types (requirement 6 wins over the reference guards)

**The decision, made once and on the record.** `specs/docs-content/CONTRACT.md` requirement 6 is
binding: "Where a type column in an API table says `React.ReactNode`, it says the Rust type the port
actually accepts." The clause sat unimplemented across six iterations because two things pulled the
other way: three guards asserted upstream's React labels VERBATIM, and an earlier attempt reverted its
own edit the moment `cargo test --workspace` caught them. Requirement 6 wins, and the guards moved IN
THE SAME CHANGE — each now pins the port's Rust types, and each keeps its row-NAME assertion against
upstream's order, so the transcription drift the guard existed for is still caught:

* `crates/docs-app/src/pages/button_page.rs` — `BUTTON_SHORT_TYPES` + its doc comment;
* `crates/docs-app/src/pages/checkbox_page.rs` — `the_rows_short_summary_types_match_upstreams_render`
  (whose doc comment now states the contract and the source of each expected value);
* `crates/docs-app/src/render_test.rs` — the Button reference-table row expectations.

**What changed, and where the Rust types came from** (read off the crate's own props structs — the
item's own instruction, "do not invent"; every value below is a field's declared type):

| cell | was (upstream's React) | now (this port) | read from |
| --- | --- | --- | --- |
| Checkbox Root `inputRef` | `React.Ref<HTMLInputElement>` | `Rc<dyn Fn(Option<web_sys::HtmlInputElement>)>` | `checkbox/root.rs` |
| Checkbox Root/Indicator `style` | `React.CSSProperties \| function` | `Vec<(String, String)>` | `checkbox/root.rs`, `checkbox/indicator.rs` |
| Checkbox Root `render` | `ReactElement \| function` | `RenderProp` | `checkbox/root.rs` |
| Checkbox Indicator `render` | `ReactElement \| function` | `Rc<dyn Fn(CheckboxIndicatorRenderState) -> AnyView>` | `checkbox/indicator.rs` |
| Button `style` / `render` | `React.CSSProperties \| function` / `ReactElement \| function` | `Option<StyleSource>` / `Option<RenderProp>` | `button.rs` via `UseRenderElementComponentProps` (`use_render_element.rs:350`) |
| Avatar Root/Image/Fallback `style`, `render`, `onLoadingStatusChange` | `React.CSSProperties`, `ReactElement`, `((status) => void)` | `Vec<(String, String)>`, `RenderProp`, `Rc<dyn Fn(ImageLoadingStatus)>` | `avatar/mod.rs`, `avatar/image.rs:505` |
| Field Validity `children` | `React.ReactNode` | `Box<dyn Fn(FieldValidityPayload) -> AnyView>` | `field/field_parts.rs` |
| Form `actionsRef` | `React.RefObject<Form.Actions \| null>` | `FormActionsRef` | `form.rs` |
| Progress Value `children` | `React.ReactNode` | `Box<dyn Fn(&str, Option<f64>) -> AnyView>` | `progress.rs` |
| Meter Value `children` | `React.ReactNode` | `Box<dyn Fn(&str, f64) -> AnyView>` | `meter.rs` |
| DirectionProvider `children` | `React.ReactNode` | `Children` | the port's own child convention |
| Checkbox/Button/Accordion descriptions | `` `ReactElement` `` in prose | `` `RenderProp` `` | — |

**A FINDING FOR THE LIBRARY LANE, not a copy edit — `accordion_reference.rs`'s 15 cells have no Rust
answer at all.** The accordion's five parts are `#[component]` fns taking `class` + `children` only
(`crates/leptos-ui/src/accordion/mod.rs:177/293/456/473/634`), so the `style` and `render` rows
describe props this port does not expose. Their cells now say `not exposed` /
`not exposed by this port yet`, and each part's description carries the same statement; that is the
TRUE answer for this port, where inventing a Rust type would have been the false one. The underlying
surface gap (the accordion parts were not built on the `UseRenderElementComponentProps` vocabulary
Button and Avatar use) is a `library:` decision and is recorded in `TODO.md` against this item — the
`docs-chrome: API reference code blocks (prop Type cells + Additional Types bodies)` item, which this
item blocks, is the natural owner once the props exist.

**Same class, still open, deliberately NOT touched here:** the `Props:` blobs on
`field_page.rs` for `Label` (L166), `Control` (L173), `Description` (L178), `Item` (L183) and `Error`
(L188) still end `, className, style, render.` although `Field::Label`/`Description`/`Item`/`Error`
expose `class` + `children` and `Field::Control` exposes `class` + `element_attributes`. Those rows
carry no React TYPE, so they are outside this item's clause; they are a surface-accuracy question for
the API-reference lane (and rewriting them moves table-cell copy recall on a route whose copy axis has
no claimant). Recorded here so the gap is visible rather than inherited.

## 2026-09-16 — three scope defects found in the gates this item is measured by (all fixed at the root, all disclosed as measurement changes)

**1. `check-react-mentions.mjs --source` counted `#[cfg(test)]` fixtures and the classifier module as
page copy.** Its header has always said "reader-facing content only", and it excluded `*_test.rs`
files by NAME — but the `snippet_language_guard` modules keep verbatim upstream snippets as POSITIVE
CONTROLS (so their "every snippet teaches the port" assertions cannot pass vacuously) in inline
`mod`s the name filter never saw. 10 of the 58 defects at this tree's HEAD were such fixtures, and 1
was `snippet_language.rs`'s own `has("@base-ui/react")` marker list — the instrument, not the page.
Both are now excluded, via the shared classifier in `ralph/scripts/lib/source-scope.mjs`.

**2. `check-react-mentions.mjs --all` never ran the rendered scan.** `wantSource` was
`arg('source') || arg('all')`, and the source branch `process.exit`-ed, so `--all` returned after the
source scan. `run-regression.sh` uses `--all` as its RENDERED check for the `docs-copy:` lane, and
`docs-copy: Leptos-only mentions …`'s done-when says "no React-API/package defect … on any rendered
route" — so that half of the claim has never been measured by this gate. `--all` now runs both scopes
and exits on either. This is the "a gate that cannot run is worse than no gate, because its silence
reads as green" defect this repo has fixed before.

**3. The install-line rule and the mentions rule both counted a mirrored EXAMPLE block's first line
as "a page tells the reader to install upstream's package".** Measured before the change: all 11 of
`check-package-alias.mjs`'s defects were `from '@base-ui/react'` (a snippet's import line or a test
control) and ZERO were an install command. Those occurrences are a defect of the snippet's LANGUAGE —
owned by `docs-chrome: snippet translation (batch 3)` (progress, separator, toggle) and `(batch 4)`
(direction-provider, csp-provider), whose done-when is exactly per-route `visual-gap-report … react=0`
plus snippetLanguage purity 1.0 and whose routes are these pages — and translating them NOW is the
rework this item's own note forbids, because the port still has no namespaced surface for four of the
five components. So the findings are now CLASSIFIED, not dropped:
* `snippet-react` — a React package inside a `code_block(...)` argument (source) or inside a rendered
  `pre`/`code` leaf (rendered). Still a FAIL, still counted, still printed on every run with its owner
  named, and still measured per route by `visual-gap-report`, `check-page` and snippetLanguage purity;
* `package-react` — an install command (fatal ANYWHERE, fenced or not: the one hole that would
  matter), an import or `npmjs`/`react.dev` link in page copy, or a link target;
* `react-api` — a React API in the port's own content (the type-column class).
`--fail-on <classes>` changes the EXIT CODE ONLY, and `run-regression.sh` now gates
`docs-copy: install lines …` on the two classes its done-when claims while every other caller keeps
the full default. The class split is the same "the per-item gate and the acceptance bar are different
questions" correction this file already records three times (94d49eb10, the snippet size floor, the
copy bar).

**Positive controls, because a narrowing that cannot fail is not a narrowing.**
`ralph/scripts/lib/source-scope.selftest.mjs` (run on EVERY item by `run-regression.sh`, pure JS,
milliseconds) asserts that each excuse still fails on the real thing: an `npm install @base-ui/react`
inside a `code_block` argument is NOT excused; `props.children` stays a defect inside a JSX block and
in rendered prose; an import outside a code block stays a defect; the line after a `#[cfg(test)]`
module is page copy again. 17 controls, all held.

**`props.children` was also a false positive on this port's own Rust.** `use_render_page.rs:163`'s
`let children = props.children;` is a struct-field access on this port's `TextProps`, in a `.rs` file.
The marker is now Rust-aware: inside a `Lang::Rust` block, or outside any code block in a Rust source,
it is a field access; inside a JSX/TSX block it is upstream's idiom and stays a defect; and the
rendered scan never excuses prose. CONTRACT.md requirement 6's list is therefore still fully gated —
the gate now agrees with the contract instead of mis-reading the port's own language.

**Measured effect at this tree (`crates/docs-app` sources):** `check-react-mentions.mjs --source`
58 fail → 6 fail, all of them `snippet-react` and therefore re-homed (the gated classes reach 0);
`check-package-alias.mjs` 11 defect(s) → 0, with 6 snippet imports counted and printed as re-homed;
`cargo test -p docs-app` 63 passed / 0 failed.

**One more, observed and NOT fixed (owner: the API-reference lane).** The reference tables render every
type cell as `<code class="Code TableCode language-ts">` (`crates/docs-app/src/reference.rs:193,244`),
including the cells this iteration changed to Rust types — a Rust type announced as TypeScript. It is
not a React API mention, so no gate fails on it, and it is a rendering decision belonging to
`docs-chrome: API reference code blocks (prop Type cells + Additional Types bodies)`. Logged here so
it is decided rather than inherited.

---

## The five pages whose snippets still taught upstream have NO `## Snippet & behaviour contract` table, and the reason the last six defects were "not this lane's" no longer holds

**Appended by** the `docs-copy: install lines + React type columns on the 18 mirrored pages
(no-rework lane)` iteration (2026-09-16). Two findings, one SPEC gap and one re-measurement.

### 1. Spec gap: five of the pages this lane's `routes:` field names have no contract table

`node ralph/scripts/check-docs-contract.mjs` lists 4 contracted pages (accordion, button, checkbox,
otp-field) and names the rest as not started. Measured this iteration:

```
for p in progress separator toggle csp-provider direction-provider; do grep -c 'Snippet & behaviour contract' specs/docs-content/$p/page.md; done
progress             0
separator            0
toggle               0
csp-provider         0
direction-provider   0
```

`specs/docs-content/CONTRACT.md` requirement 3 makes that table the page's spec-level statement of
per-example snippet + behavioural obligations, and requirement 5 makes it part of a `docs-content:`
item's done. The five routes above are exactly the routes the React-mentions gate still reported
snippet defects on — i.e. the pages whose repair work is *most* contract-shaped — so the gap is
load-bearing, not bookkeeping. **Owner: `docs-spec: snippet & behaviour contract on every mirrored
page`** (its note already carries this queue). Not authored here: this iteration is a `docs-copy:`
item, and `CONTRACT.md` requirement 3's rule ("each page's obligations come from its own
behavior.md sections") makes the tables a deliberate, cited authoring pass, not a side edit.

**Which rows the tables will need, measured rather than guessed** (so the docs-spec iteration does
not re-derive them): one Anatomy row per page, plus — for progress — a hero-demo row (the
interval-driven value, `specs/library/progress/behavior.md` § State model / § Accessibility), and
for csp-provider two more rows (the nonce propagation and the `disableStyleElements` arm, § Public
API surface / § State model). Observable candidates already exist in the wasm suite:
`render_test.rs::separator_page_renders_the_hero_demo_through_the_real_port` (role/aria-orientation/
data-orientation on the real port), `::toggle_page_renders_the_hero_demo_through_the_real_port`
(pressed flip through the real machine), `::progress_hero_demo_drives_the_real_part_tree_through_the_interval_simulation`,
`::csp_provider_page_renders_probes_through_the_real_provider_and_hook`,
`::direction_provider_page_renders_probes_through_the_real_provider_and_hook`.

**Loop-level consequence, for whoever picks that item: its own gate cannot pass in one iteration.**
`ralph/scripts/run-regression.sh:308-310` hard-gates `check-docs-contract.mjs --strict` for
`docs-spec:*` ids, and `--strict` requires *every* mirrored page — 17 of them at this tree. Either
the item is worked as a batch (the surface batches' precedent: `--components <batch>`, one batch per
iteration, status left `not-started` with a progress note) or `run-regression.sh` gains the same
per-batch narrowing the surface check already has. Recorded here so the next iteration does not
discover it as a red regression.

### 2. Re-measurement: "the port has no namespaced surface for four of the five components" is stale

The earlier entry in this file (and `run-regression.sh`'s `--fail-on react-api,package-react`
comment) reasoned that translating these blocks NOW would be rework, because `check-part-surface.mjs
--strict` reported checkbox-group, separator, toggle, csp-provider and direction-provider MISSING.
Re-measured at this tree:

```
node ralph/scripts/check-part-surface.mjs --components progress,separator,toggle,csp-provider,direction-provider
  OK   progress: 5/5
  (separator, toggle, csp-provider: no dotted part documented upstream — INERT units, nothing owed)
  MISSING direction-provider: 0/1 — missing direction_provider::Props
```

* **progress** has the full namespaced surface (`Progress::Root`/`Label`/`Track`/`Indicator`/`Value`,
  pinned by `crates/leptos-ui/tests/part_surface.rs:264-281`), so `<Progress::Root>` is the port's own
  teaching form — translating its Anatomy fence is exactly what `CONTRACT.md` requirement 1 requires,
  with no rework implied.
* **separator** and **toggle** are single-element units: upstream documents no dotted part
  (`specs/library/{separator,toggle}/behavior.md` § Public API surface), `library: separator` and
  `library: toggle` are both `done` on the element-description surface, and the CLOSED button page
  already teaches that same "build it, then materialize it" form
  (`button_page.rs:101-110`). There is no future `Component::Part` spelling for them to be re-spelled
  into.
* **csp-provider** and **direction-provider** teach `provide_csp_context` /
  `provide_direction_context` — the exported Rust half of each provider, whose view wrapper is
  documented as a Phase C concern (`csp_provider.rs:29-33`, `direction_provider.rs:36-41`) and which
  this port's own pages already call in their live sections. `direction_provider::Props` is a props
  TYPE row, not a component part, so the surface gate's MISSING line does not make the provider
  uncallable — measured: the ports exist, are `pub`, and are what the page's live demo runs.

So the six remaining `snippet-react` defects were translatable in this lane after all, and are
translated (see this iteration's commit). The distinction that DOES survive: a component whose
namespaced parts are coming (checkbox-group, and the menus/inputs batches) should still wait — the
old note was right about *those*, just not about these five.

### 3. Two wasm render assertions REQUIRED the page to render upstream's source

`crates/docs-app/src/render_test.rs:782` (`separator_page_component_renders_the_full_page_structure`)
asserted `html.contains("@base-ui/react/separator")` and `:2726`
(`progress_page_route_renders_the_mirrored_structure`) asserted `html.contains("@base-ui/react/progress")`
— i.e. the page's own render test FAILED if the snippet was translated. That is the "a measure that
rewards copying is a defect" class the ledger already names, in its most literal form: the test was
the reason the page looked immutable. Both now assert the port's own surface through `text_content`
(the accordion page's precedent, `:1801-1805`) and additionally assert the upstream string is ABSENT.
Same shape as `:3392`/`:4430`/`:2288`, which already do it correctly for checkbox/otp-field/accordion.

---

## 2026-09-16 — requirement 6's publication status is stale, and it is the BINDING half of the pair

Logged while closing `docs-copy: install lines + React type columns on the 18 mirrored pages
(no-rework lane)`, whose two cited specs are exactly the two files that now disagree.

**The finding.** `specs/docs-content/CONTRACT.md:118-120` (requirement 6 — "binding convention for
every `docs-content: components/<name>` and `docs-content: utils/<name>` item") instructs a page
author: *"The alias is mapped locally at `packages/leptos/` and is deliberately unpublished; a docs
page must still use the port's name, and must not fabricate an install command that would 404 (say
what is true today: the crate path `crates/leptos-ui`, the alias, and that publication is pending)."*

Measured against the live registry, not against another artifact in this repo
(`curl -H 'User-Agent: …' https://crates.io/api/v1/crates/base-ui-leptos`, 2026-09-16): the crate IS
published — `created_at 2026-09-16T09:23:38Z`, `newest_version 0.1.6`, `num_versions 6`,
`yanked false`, 35 downloads. So "publication is pending" is false, and the constant that says the
true thing is the OTHER spec of this item, `crates/docs-app/src/install_ref.rs` (`PUBLISHED = true`,
plus `INSTALL_SNIPPET`'s real `cargo add base-ui-leptos`). A page author following requirement 6
literally would write the stale fact onto the page — and neither command this item is measured by
(`check-package-alias.mjs`, `check-react-mentions.mjs`) can see a *stale* statement, only a wrong
*name*.

Same file, smaller drift: `install_ref.rs`'s module doc pins a version literal ("PUBLISHED on
crates.io at 0.1.1") while crates.io serves 0.1.6 — a version in a comment that will keep drifting.
Not edited in this iteration on purpose: the full regression that closed the item was measured on the
unmodified tree, and a comment edit after it would leave the evidence one revision behind the
commit.

**Not done, deliberately** (so the next iteration does not have to re-derive the boundary):
requirement 6's text is NOT rewritten — `specs/**` is read-only for an item that is not a
`docs-spec:` item — and no new install prose was invented. `ALIAS_STATUS` is the canonical,
gate-asserted status line and it is true as written today ("the JavaScript package alias, mapped
locally in this repo and not published; the Rust crate `base-ui-leptos` is the installable
artifact"). The repair belongs to whoever next authors a mirrored page's install reference, or to the
`docs-spec: snippet & behaviour contract on every mirrored page` item that owns CONTRACT.md.

**Addendum — the one part of clause 1 that NO gate polices, stated rather than implied.** Clause 1 is
"every page's install reference renders the constants from `install_ref.rs`". Its NAMING half is
measured (`check-package-alias.mjs`: the constants agree with the manifest, the bare specifier resolves
from both roots, no page names upstream's package), and its rendering half is asserted for the nav by
`crates/docs-app/src/render_test.rs::side_nav_lists_every_ported_route_grouped_like_upstream`, which
compares the rendered link pairs against `NAV_EXTERNAL` — i.e. against `RUST_CRATE`/`CRATES_IO_URL`.
But that assertion is a `#[wasm_bindgen_test]`: it compiles under this gate and executes only in a real
browser, and the RENDERED mentions probe cannot cover it either, because its probe queries
`document.querySelector('main')` (`check-react-mentions.mjs:141`) while the nav and header live outside
`main` — which is why every route reports "our-package mentions 0" even though the sidebar link IS the
port's crate name. So: the chrome's install reference is proven by a compiled-but-browser-run test plus
the source-level alias gate, never by a rendered-DOM assertion. Adding that assertion is a tooling
change to `check-react-mentions.mjs` (or a route-level probe), which is why it is logged here instead of
quietly counted as measured.

---

## 2026-09-16 — `docs-copy: Leptos-only mentions + the base-ui-leptos alias`: two instrument defects, and one place the gate cannot see

Written by the iteration that closed the item. Requirement 6 of `specs/docs-content/CONTRACT.md` is the
clause being satisfied; this entry records what had to change in the *measuring instrument* to satisfy it
honestly, and one visibility gap that survives.

**1. An approved attribution was also reported as an unreviewed mention (fixed).** `classifyLine` pushed
every matching rule, and only a `fail` broke the loop, so a line matching `ATTRIBUTION_RE` was reported
*twice*: once as `attribution` (allowed, per requirement 6's "crediting the original work is allowed) and
once as a `react-word` WARN. Measured before the fix: `install_ref.rs`'s `PROVENANCE` constant ("Ported
from the React implementation of Base UI …") and both lines of `form_page.rs`'s upstream-reference
paragraph were reported as open decisions although the script's own header says "any OTHER bare React is a
WARN" and its own comment says attribution "is checked FIRST". Fixed by suppressing the warn only when the
line already matched the attribution rule. This can suppress nothing but a warn: the two FAIL classes are
evaluated independently, and a line that both credits upstream and ships a React API still fails (pinned by
a new self-test fixture, `credit-cannot-excuse-a-hook`).

**2. The source scan could not read any page's allow file (fixed).** The lookup was
`path.basename(path.dirname(file))`, which for every `crates/docs-app/src/pages/<name>_page.rs` is the
literal string `pages` — not a docs-content entry, so it returned an empty list for every page on the site.
`specs/docs-content/otp-field/react-allow.json` existed, listed the exact sentence, and the source scan
still reported that sentence as unreviewed: the decision was on the record and the gate could not read it,
which made this item's own done-when clause unreachable by construction. The rendered scan (keyed off the
route basename) had always read the right file, so the two halves disagreed about the same page. Fixed by
keying `pages/<name>_page.rs` → `specs/docs-content/<name>/` (snake → kebab) when that page exists.

**3. A package NAME's own hyphenated identifier was read as the framework word (fixed).** `\breact\b`
matches inside `floating-ui-react` (a hyphen is a word boundary), so `status_data.rs`'s component list —
where the upstream unit is *named* — produced "the bare word React" warnings. The rule now requires the word
to stand alone (`(?<![\w-])react(?![\w-])`), which is what the class claims to find; leaks that actually
ship React are caught by the `package-react`/`react-api` FAIL rules (`@base-ui/react`, `from 'react'`,
`react.dev`) and never depended on this warn. All three fixes ship with `check-react-mentions.mjs`'s own
10-fixture self-test, run on every invocation in both directions, per `ralph/scripts/gate-selftest.mjs`'s
"an invariant that cannot fail is not an invariant".

**Measured before → after, same tree:** source scan 15 warns → 11 (fix 1: −2, fix 3: −1, fix 2: −1) → 1
(the three mirrored pages plus the app-chrome records written below) → 0; 0 FAIL throughout; rendered scan
5 warns → 0 with 0 FAIL. `check-package-alias.mjs` 0 defect(s) in both runs.

**4. A NEW ALLOW-FILE LOCATION, logged rather than added silently — `specs/docs-app/react-allow.json`.**
Requirement 6 names `specs/docs-content/<name>/react-allow.json`, which is keyed by *mirrored page*. The
docs-app also has reader-facing source that is not a mirrored page: the `/status` report and its generated
data module, whose labels name the framework they count (`<th>"snippets leptos/react"</th>`,
`status_data.rs`'s `EXPLANATION` sentence naming this very check). Those two strings are recorded in
`specs/docs-app/react-allow.json` — same `{ match, reason }` shape, read by the source scan for every
`crates/docs-app/src/**` file that is not a mirrored page. This is an EXTENSION of the contract's mechanism,
not a rewrite of it; `specs/docs-content/CONTRACT.md` is left untouched, per the rule that a non-`docs-spec:`
item does not edit specs. The natural next owner is the `docs-spec:` item that owns CONTRACT.md.

**5. VISIBILITY GAP, recorded not claimed (pre-existing, not introduced here).** The rendered probe reads
only `document.querySelector('main')` (`check-react-mentions.mjs:141`), so the chrome — sidebar, header,
install line — is never scanned in rendered mode (already logged above, in the install-lines addendum: which
is why every route reports "our-package mentions 0" even though the sidebar link IS the crate name). A
second, same-shaped gap surfaced while checking the accordion's own warnings: the accordion's four
`react-word` warnings are DEMO PANEL text whose element is not one of the probe's leaf tags, so the rendered
scan reports that route clean while the source scan sees it. Concretely: **"rendered scan 0 defects" is a
statement about paragraphs, list items, headings, table cells, links and code leaves inside `<main>` — not
about every string a reader sees.** Closing either gap is a tooling change to `check-react-mentions.mjs`
(broaden the probe's tag set; scan the chrome), which this item did not make, because widening the probe
would re-open warnings across pages whose ownership lies with the `docs-chrome:`/`docs-content:` items.

## 2026-09-16 — the rendered measurement's own instruments (found by `tooling: the CI scorecard's rendered axes are UNMEASURED on every route`)

1. **`check-page.mjs`'s human-readable path threw a `ReferenceError` and never wrote its scorecard.**
   `const REASON = { … }[a.axis] || /FAIL|UNMEASURABLE/i;` referenced the loop variable `a` before its
   `for (const a of axes…)` — so `node ralph/scripts/check-page.mjs --route <route>` (the interface the loop
   is told to use in step 6f) printed the axis table, then died: `ReferenceError: a is not defined`, exit 1,
   and `ralph/logs/scorecard/<name>.md` was NEVER written. CI never saw it because CI runs `--json`, which
   skips that branch. Reproduced and fixed this iteration by running the HEAD revision side by side.
2. **`check-page.mjs` scored a report file it had not written** (fixed: freshness + `refused` check). Measured
   before the fix: with every browser gate deferred (`RALPH_BROWSER_GATES` unset), the accordion route still
   reported `snippet language PASS (0)`, `example length FAIL (42.8)` and `attribute density PASS (1.38)` —
   numbers read from `ralph/logs/visual/accordion-snippets.json` as committed at 17:22 by a local sweep.
3. **`measure-port.yml` runs `gate-selftest.mjs` with `|| true`** (line 188), so a broken instrument cannot
   fail the measurement job. The workflow is `.github/workflows/**`, which `CONTEXT.md` puts behind the
   owner's authorisation, so this is a REQUEST, not an edit: the self-test should be able to turn the shard
   red. Its own doc already says no number from these reports may be quoted while it is not green.
4. **`playwright-diff.mjs` and `visual-gap-report.mjs` start Chromium with NO browser-budget guard.**
   `ralph/scripts/lib/browser-budget.mjs`'s own header says the guard must live in the scripts because every
   caller shares them, and six gates comply — these two do not, so `check-page.mjs` (which spawns
   `playwright-diff.mjs`) measured this box with a real Chromium this iteration while the other five axes
   correctly reported UNMEASURED, and `run-regression.sh:139-142` calls `playwright-diff.mjs` unguarded for
   any route-shaped item id. Not fixed here (it needs the refusal path in `run-regression.sh` too: exit 2 is
   UNMEASURED, not a failed differential); scoped into the ledger instead.
5. **The repo root cannot resolve `base-ui-leptos` from tracked files.** `check-package-alias.mjs` asserts the
   bare specifier resolves from the repo root AND from `test/node-resolution`, but only the fixture declares
   it (`test/node-resolution/package.json`, `base-ui-leptos: workspace:*`, in the lockfile); the root
   `package.json` has no such dependency, so a fresh `pnpm install` cannot create the root link. Measured
   before/after this iteration by removing the two untracked symlinks: the HEAD gate reported **4 defects,
   exit 1** (exactly CI's verdict), the fixed gate repairs the mapping it asserts and says so. Whether the
   root should DECLARE the dependency (which needs a `pnpm-lock.yaml` regeneration) is a packaging decision
   for the owner, not an iteration's call.
6. **A measurement can silently fail to reach the committed scorecard.** `measure-port.yml`'s aggregate job
   commits `ralph/generated/scorecard.jsonl` and then `git pull --rebase --autostash origin <branch>` before
   pushing — but the branch usually already carries an earlier `[measure] CI scorecard: …` commit from
   another run, and rebasing one rewrite of that append-log onto another CONFLICTS. MEASURED, run
   35141983661 (aggregate job 104957238689): `error: could not apply 5dc6741... [measure] CI scorecard: 17
   route(s) measured`, three retries, then `::warning::could not push results (branch busy); the next
   scheduled run will re-measure`. The consequence is not cosmetic: that run's records (generatedAt
   20:05–20:20, with all four axes measured) never landed, so `scorecard-latest.mjs` and the `/status` page
   kept serving the PRE-fix records from the run before it (generatedAt 19:47–20:01, four axes UNMEASURED,
   `package alias` FAIL) — the loop reads a stale product measure, and the fix that produced the numbers
   looks unverified. The order that cannot conflict is: pull/rebase FIRST, then regenerate the dedup output
   from the merged file, then commit and push (the file is derived data, so recomputing it after the merge
   is always correct and never conflicts). Workflow edit → owner's authorisation, so this is a REQUEST.
7. **`scorecard-latest.mjs` cannot tell "measured long ago" from "the newest run's write-back was lost"** —
   it prints an age, which is the right instinct, but nothing fails when the committed record is older than
   the newest successful measurement run. A gate that compares the file's newest `generatedAt` with the
   newest completed `measure-port.yml` run would have caught (6) on the spot.

## 2026-09-16 — the citation repair found five things the checker structurally cannot see (found by `tooling: 50 TODO.md citation ranges were displaced by the ledger's own growth — re-anchor them`)

The repair itself is mechanical (re-anchor to the claim's true target, verify the claim at the new
target, re-record the baseline). These five are the things it exposed about the INSTRUMENT, each
measured rather than inferred, and each left for a later iteration because fixing them is not
"move a line number".

1. **A citation can name a ledger item that does not exist, and the checker cannot see it.**
   `specs/library/toolbar/implementation.md:6` claims *"The TODO.md entry for `library: toolbar`
   (`TODO.md:565-571`) has no `wraps-external:` field"*. There is no `library: toolbar` item —
   MEASURED: `TODO.md`'s Phase B section holds 38 items (`accordion … tooltip`) and none of them is
   toolbar, while `specs/library/toolbar/{behavior,implementation}.md` exist, `ralph/generated/
   components.json` lists the unit, and `docs-content: components/toolbar` (`TODO.md:1877`) carries
   `owner: library: toolbar`. A prior commit had already noticed the phantom and rewrote that
   docs item's `blocked-by` from `[library: toolbar, …]` to `[library: menubar, …]` — so the ledger
   knows the entry is absent and the spec was never told. WHY THE CHECKER IS BLIND TO IT: the
   citation resolves (the lines at 565-571 exist and are non-empty), so only the HASH can fail, and
   the hash had been recorded against whatever text sat there — today combobox's entry — which the
   ±40-line tolerance then matched as a "moved" warning rather than a failure. The consequence of
   the gap is real and larger than the citation: **the port of `toolbar` has no ledger item, so it
   can never be scheduled**, and its docs item is blocked on `library: menubar` instead. Scoped into
   the ledger as its own item rather than invented here (authoring a Phase B entry by hand would
   also shift every line number this iteration just repaired).
2. **Four baselines had been recorded against a NEIGHBOURING entry's text.** The citation checker's
   graded tolerance found the recorded window at a nearby offset for `context-menu` (×2 specs),
   `progress`, `tabs` and `toolbar` (5 occurrences, 4 keys) and reported "moved by +13/+19 line(s)
   — window content identical", i.e. a SOFT warning that exits 0. What the offsets actually land on
   is other units' note text: MEASURED — `TODO.md:627-635` (the `+13` answer for context-menu) is
   combobox's note tail while context-menu's entry is 636-648; the `+19` answers for progress and
   toolbar land inside combobox's 582-591; the `+13` answer for tabs is checkbox-group's note.
   WHY IT READ AS CONSISTENT FOR SO LONG: every one of those neighbouring entries also carries no
   `wraps-external:` field, so the claim stayed accidentally true while pointing at the wrong unit.
   **A `record` run would have made this permanent** by adopting that text as the baseline — which
   is exactly the "measure that rewards copying" defect the ledger's own note predicted for this
   item, observed here rather than hypothesised. Repaired by re-anchoring all five to their unit's
   OWN entry (each verified: the target is that unit's entry and carries no `wraps-external:`).
3. **The tolerance that makes drift survivable is also what hides a mis-anchor.** The same ±40-line
   search that correctly absorbs a few lines of ledger growth will find a match somewhere for ANY
   window made of repeated boilerplate (every Phase B entry shares the identical 4-line
   `blocked-by: []  # WAS [Phase A complete] …` comment block). `findUniqueNearbyDrift` fails
   closed on AMBIGUITY but has no notion of "the match is a different unit's entry" — and it cannot
   have one, because the key is a hash. A cheap discriminator for a future iteration: for a
   `TODO.md` key, require the matched offset's range to fall INSIDE the same ledger entry (the
   bullet above it names the same item id) before calling it a move.
4. **Editing a spec's prose dirties cross-spec citations.** `specs/library/number-field/
   implementation.md:25` cites `specs/library/number-field/parts/scrub.implementation.md:5-37`, and
   the checker's ±2-line window for that range spans lines 3-39 — so this iteration's rewrite of a
   citation token on scrub's line 3 turned that citation red even though the pointer ("Depth: …")
   it backs is unchanged. Same mechanism as the already-logged "a citation on entry N is re-dirtied
   by editing entry N+1", now with the cited file being a spec rather than the ledger; re-recorded,
   with the change inside the window known and verified to be my own citation token. Worth naming
   because it means the specs tree is not inert under a citation repair: 53 re-anchors can dirty
   citations between specs, and only re-running the gate finds them.
5. **One claim's enumeration is stale, left in place on purpose.** `specs/library/navigation-menu/
   behavior.md:7` says the unit is not on the `needs-batched-mining` list and enumerates that list
   as `combobox, drawer, floating-ui-react, menu, number-field, select`. MEASURED today: 8 entries
   carry `needs-batched-mining: true` (`TODO.md:321, 587, 677, 849, 902, 1018, 1079, 1120` — i.e.
   the enumerated six plus `toast` and `tooltip`). The citation's subject (this unit's own entry,
   which carries neither field) re-anchors and verifies cleanly, so the citation half is repaired;
   the parenthetical list is a claim about the ledger that has since grown, and correcting it is a
   spec edit this iteration deliberately does NOT make — it is recorded here instead, per the rule
   that a spec found wrong is logged rather than silently rewritten.

## 2026-09-16 — `library: input`: two of the unit's four conformance-proven props have nowhere to delegate through, and a done-marking can hard-fail a neighbour's citation

Found while closing `library: input`. Neither item below is a port defect; both are instrument/ledger
findings, and each is scoped or repaired as a named action rather than absorbed silently.

1. **`render` and `ref` cannot be ported through this unit, because the delegation target has no slot
   for either.** `specs/library/input/behavior.md` proves four props for `Input` through the shared
   conformance harness — propsSpread, refForwarding, renderProp, className
   (`packages/react/test/describeConformance.tsx:44-67`) — and `specs/library/input/implementation.md`
   is explicit that the unit's entire body is `return <Field.Control ref={forwardedRef} {...props} />`
   (`packages/react/src/input/Input.tsx:12-17`). MEASURED, not inferred: the port's delegation target
   carries no slot for two of the four. `FieldControlViewProps`
   (`crates/leptos-ui/src/field/field_control.rs:34-54`) has nine fields and neither is `render` nor a
   ref, and `grep -rn "ref_callback\|RefCallback" crates/leptos-ui/src/field/` matches NOTHING, while
   other units of the same crate DO expose that shape (`crates/leptos-ui/src/otp_field.rs:1924`).
   `render`'s element form additionally needs a tag substitution the control's fixed-`view!` path
   cannot express (`crates/leptos-ui/src/field/field_control.rs:604-628`), which is the already-scoped
   `library: the view paths drop render's element form`. CONSEQUENCE, stated rather than papered over:
   behavior.md's "Public API surface" lists four props and this port can honestly carry two. That is
   NOT a spec error — the spec faithfully records upstream — so the spec is not rewritten for it; the
   `ref` half is scoped as its own new ledger item and the `render` half stays on the existing one.
   Corollary for the next `docs-content: components/input` iteration, measured now so it is not
   re-derived: the pair's hero demo uses only `placeholder` + `className`
   (`specs/docs-content/input/demos.json`), both of which the port carries, so this gap does not block
   the pair.
2. **Flipping a ledger checkbox can HARD-fail a NEIGHBOUR's citation, because the hash window carries
   a ±2-line margin the recorded range does not show.** The checker hashes the cited range plus two
   lines of context on each side (`WINDOW_MARGIN = 2`, `ralph/scripts/check-citations.mjs:60`, applied
   by `hashWindow`), while the sidecar key names only the range. MEASURED this iteration:
   `specs/library/form/implementation.md` cites `TODO.md:769-824` — form's own entry, whose last line
   is 824 — and `library: input`'s entry starts at 825, so flipping its `- [ ]` to `[x]` (mandatory:
   `ralph/scripts/check-todo-schema.mjs:101-105` requires `[x]` exactly when `status: done`) edited a
   line INSIDE form's hash window. The check then reported
   `specs/library/form/implementation.md: citation TODO.md:769-824 content has drifted since it was
   recorded` — a HARD failure — although the cited subject was byte-identical: `git diff -U0 HEAD --
   TODO.md` showed hunks only at 825, 832-835 and one insertion at 837, all outside 769-824. The
   drift search cannot rescue this class (the change is inside the window, not a pure shift), and the
   failure text blames the neighbour's citation rather than the edit that caused it. REPAIR APPLIED —
   and explicitly NOT a spec rewrite: the subject range was verified unchanged, then that spec's
   baseline was re-recorded (`node ralph/scripts/check-citations.mjs record --scope
   specs/library/form`), which changed exactly ONE key in the sidecar, in place, order preserved.
   GENERALIZES: any done-marking whose entry begins within two lines of a neighbour's cited range
   will trip that neighbour. Mitigation discovered here and used for this item's own entry: make the
   ledger edit NET-ZERO in line count — the new `exempt-from-docs-pairing` field was placed on the
   line the entry's `blocked-by` comment block used to end with (its text preserved inside the new
   field's comment), so `wc -l TODO.md` stayed equal to `git show HEAD:TODO.md | wc -l` (3576), no
   line below moved, and the whole-tree check printed its baseline 14 warnings instead of the 44 it
   printed while the extra line was present. Worth adopting as the convention for a Phase B
   done-marking: add the field line by replacing an expendable comment line in the item's OWN block.

## 2026-09-16 — `library: menu` is marked `done` while its whole view layer is a fabricated stub

FOUND by the `library: switch` iteration while re-checking done-claims before picking work. This is a
LEDGER/SPEC-scope finding, not a spec-citation error: nothing in `specs/library/menu/**` is wrong.

MEASURED (see the `library: menu — the view layer ...` item appended to `TODO.md` for the full
inventory and the commands):

- `TODO.md:839-849` marks `library: menu` `status: done`; the close's own commit subject
  (`7688042d4`) is "finish orphaned store/impl diff ... leptos-ui lib 89/89 host tests green", i.e.
  the STORE half.
- 15 files under `crates/leptos-ui/src/menu/` carry `In a real implementation, we would` bodies with
  hardcoded `class="menu-item"`/`class="menu-positioner"` shells. Four of them (`item`, `popup`,
  `portal`, `positioner`) are declared and re-exported by `menu/mod.rs` — they ARE the port's public
  `Menu.*` view surface. The other eleven are not declared at all and never compile.
- `check-part-surface.mjs --components menu` → `menu: 0/20`; `check-component-strict.mjs --component
  menu` → `parts: FAIL`, `namespaced path: FAIL`.

WHY IT MATTERS BEYOND THE COUNT: the unit's only `menuopenchange` emitter upstream is
`MenuPositioner.tsx:230`, so `library: menubar` — the smallest remaining component, whose spec cites
that event as its state source — cannot be ported honestly until the view layer exists. The iteration
that found this therefore chose NOT to port menubar (it would have been another partial) and recorded
the gap as its own item instead.

WHAT WAS *NOT* DONE, DELIBERATELY: `library: menu`'s `status` was left untouched. The evidence proves
the VIEW LAYER is unported; whether that means the item's done-claim should be reopened depends on the
scope the marking intended, and unilaterally rewriting another item's record is the kind of ledger
surgery this log exists to make visible rather than to perform silently. Owner's call.

## 2026-09-16 — the docs-pair `done-when`'s "Playwright differential against the original React docs page" is only ever measured in its STRUCTURE-ONLY mode, and the upstream-side axis fails on already-done pages

FOUND by the `docs-content: components/input` iteration while verifying its own done-when's verification
clause, which reads "verified via Playwright differential test against the original React docs page,
not just a smoke render" (`TODO.md`, the `docs-content: components/*` entries).

MEASURED, not inferred:

- `bash ralph/scripts/run-regression.sh "docs-content: components/input"` runs step 4's differential
  as `node ralph/scripts/playwright-diff.mjs --todo-id "<id>"` and exits 0. The report that run
  produces has no `upstreamMounted`/`headingsSubset` keys at all: the upstream comparison is behind
  `if (UPSTREAM || process.env.DIFF_UPSTREAM === '1')` (`ralph/scripts/playwright-diff.mjs:168`), and
  neither the gate nor any workflow sets it. So the clause every docs pair carries is satisfied by
  `leptosMounted` + `hasH1` + `nonEmptyTree` — the smoke-render half — for every item, including the
  ones whose notes record the differential as "verified".
- With the upstream side actually on (`DIFF_UPSTREAM=1`, upstream `127.0.0.1:3005` serving), the new
  `react/components/input` route reads `upstreamMounted: true`, `headingsSubset: 0.5`, `pass: false`
  against a bar of 0.8 (`ralph/scripts/playwright-diff.mjs:192-194`).
- THE SAME AXIS FAILS ON AN ALREADY-DONE PAGE: `DIFF_UPSTREAM=1 playwright-diff.mjs --todo-id
  "docs-content: components/checkbox"` reads `upstreamMounted: true`, `headingsSubset: 0.5625`,
  `pass: false`. The cause is shared, not page-specific: upstream's generated Additional-Types
  heading contains a disclosure link (`docs/src/components/ReferenceTable/AdditionalTypes.tsx:44-56`
  — `<a href="#" className="AdditionalTypeBackLink">Hide</a>`, later `Back` after a hashchange), so
  upstream's heading TEXT is e.g. `Input.PropsHide`, while the port's shared
  `crate::reference::additional_types` (`crates/docs-app/src/reference.rs:378-409`) renders the same
  heading without that link — a deviation its own module docs record, assigning the panel's reveal
  to `docs-chrome: code blocks` / the Additional-Types bodies to `docs-chrome: API reference code
  blocks (prop Type cells + Additional Types bodies)`.

WHAT THIS MEANS FOR THE LEDGER, stated plainly rather than as a slogan: the axis that would catch a
page teaching the wrong framework is the snippet-language probe, which does run on CI; this heading
axis compares generated-chrome button TEXT, and it is red for four already-done pages as well. The
honest reading is that the upstream-side comparison has never been part of any docs item's
verification, so the clauses should either name the structure-only mode or the upstream-side bar
should be enforced somewhere. NOT FIXED HERE, deliberately: rendering a `Hide` control the port does
not implement would be a fabricated affordance, and implementing its reveal is the Additional-Types
panel item's scope (`TODO.md`, `docs-chrome: API reference code blocks (prop Type cells + Additional
Types bodies)`, `status: not-started`); the `docs-content: components/input` close therefore states
that its gate differential passed in the mode the gate runs and reports this measurement rather than
claiming upstream parity.

## 2026-09-16 — `library: radio-group`: the capture-phase arm rides the bag's bubble slot

`specs/library/radio-group/implementation.md` (`:118-123`, `:152-159`) and behavior.md (`:26-28`, `:40`)
both describe the group's arrow handling through `onKeyDownCapture` (`RadioGroup.tsx:249-254`), which
upstream attaches on the CAPTURE phase so the group-local `touched` "auto-select armed" flag is set
before any child handler runs.

The ported bag vocabulary has no capture slot — `RenderElementHandlers`
(`crates/leptos-ui-internals/src/use_render_element.rs:117-139`) carries `on_focus`/`on_blur`/
`on_click`/`on_mouse_down`/`on_context_menu`/`on_mouse_move`/`on_key_down`/`on_key_up`/
`on_pointer_down` and the lazy `attributes` list, and nothing else. The arm is therefore attached as
the FIRST entry of the group's `defaultProps` bag, whose `on_key_down` the composite root's own
navigation handler is chained AFTER (`mergeProps` composes in bag order), so for the same event the
arm still runs before the navigation pipeline.

WHAT THIS DOES NOT CLAIM: the ordering guarantee is bag-order, not DOM capture-phase. A consumer that
attaches its own capture-phase listener outside the bag would observe the opposite order from
upstream. No test in `RadioGroup.test.tsx` distinguishes the two (the suite's arrow cases assert the
observable outcome — focus moved AND the newly focused radio became checked — not the phase), so the
port keeps upstream's OBSERVABLE behavior and this note records the mechanism difference rather than
presenting it as parity. Fixing it at root means adding a capture slot to the render-element bag
vocabulary, which is a crate-wide change belonging to the internals unit, not to this one.

## 2026-09-16 — ledger growth displaced four citations; three re-anchored, one merged away

`library: radio-group`'s Step 0 pick record is a new `chosen:` line inside
`TODO.md:982-994`. Adding it moved every citation below the entry by +1 line, and the checker's
window hashing turned that into real failures, not warnings:
`specs/library/radio-group/implementation.md` (its own citation, `TODO.md:982-993`) and
`specs/library/scroll-area/{behavior,implementation}.md` (`TODO.md:995-1005`, which the shifted
scroll-area entry had moved off).

WHAT WAS DONE, per the checker's own instruction ("verify the cited range still covers the intended
content, then update the range and re-record"): each cited assertion was re-verified against the NEW
window before re-anchoring — `grep -c wraps-external` over the new ranges returns 0, so "the entry
has no `wraps-external:` field" is still TRUE in all four places, and no claim was edited. The ranges
were then re-anchored to the entries' true extents (radio-group 982-994, scroll-area 995-1006) and
both scopes re-recorded.

Additionally, the +1 displacement was ELIMINATED rather than absorbed for the entries below
radio-group: the trailing `# shares docs-pair with library: radio …` comment was folded onto its
`docs-pair:` line, which is a visible merge that keeps the text and costs no line. That is why the
full-scope check now reports zero failures with 14 warnings, all of them pre-existing `react-allow.json`
baselines that nothing in this item touches.

MECHANISM WORTH NAMING FOR THE AUDIT LOOP: in this ledger, ANY added line is a tree-wide edit for the
citation checker, because the entries below carry citations that point INTO `TODO.md` by line range.
Iterations that record a pick therefore owe their neighbours a re-anchor, and the honest order is
verify-then-re-anchor — never re-record a baseline to silence a range whose content actually moved.

## 2026-09-16 — a consumer's hand-rolled copy of a Phase A util drifts invisibly to every gate

Found while closing `library: radio` (the landed-but-unrecorded port left one RED host test,
`radio_tests.rs:306`). The port had HAND-ROLLED the `visuallyHidden` / `visuallyHiddenInput` recipes
inside `crates/leptos-ui/src/radio/state.rs` instead of consuming the Phase A util that already
exists and that its siblings all use (`crates/leptos-ui-utils/src/visually_hidden.rs`, the port of
`packages/utils/src/visuallyHidden.ts:3-24`; the consumers are `checkbox/state.rs:341-347`,
`switch/state.rs:203-209`, `meter.rs:329`, `progress.rs:472`). The copy had drifted three ways at
once — an invented `clip: rect(0 0 0 0)`, React's camelCase `clipPath`/`whiteSpace` keys (which are
invalid CSS in a `style` attribute, so the browser silently DROPPED them from the rendered input),
and the anonymous recipe spelled `position: absolute` where upstream's `visuallyHidden` is
`position: fixed; top: 0; left: 0`.

WHY THIS IS AN INSTRUMENT FINDING AND NOT JUST A PORT DEFECT: `check-component-strict.mjs` — the
gate the loop is told to use as "the strict feedback while you work" for every `library:` item —
counts parts, props, sections and hygiene. It never asks whether a consumer's copy of a Phase A util
still EQUALS that util, so a duplicate can diverge from across the workspace with every axis green.
The only thing that fired was a hand-written unit test in the same file, and that test's own model of
upstream was wrong (it asserted that only the NAMED recipe carries `clip-path: inset(50%)`, while
upstream's shared `visuallyHiddenBase` carries it for BOTH and the ternary exists solely for the
position tail) — so a green run there would have been equally misleading. The gate-level fix is a
duplicate-detection axis: flag a `library:` unit that spells a declaration list a Phase A util already
exports (the `specs/utils/**` surface is enumerable), and report it as a gap with both sites named.
Not implemented here — this iteration's objective was the unit, and a gate edit is a tooling change
needing its own before/after evidence — so it is RECORDED, with the measurements above, for the
audit loop and the tooling lane to own.

MECHANISM THE SAME ITERATION CONFIRMED AGAIN: the radio entry's done-marking note added lines to
`TODO.md`, which displaced the citations every entry below it carries INTO `TODO.md` by line range —
the `radio-group` and `scroll-area` spec citations had to be re-anchored and re-recorded (claims
re-verified against the NEW window first), exactly as the 2026-09-16 entry above predicts.

## 2026-09-17 — the snippet-report instrument: two checker rules tightened, three stale numbers voided, and two REQUESTS that need the owner's authorisation

Item: `tooling: three routes' committed snippet reports fail the metric-invariant checker, and CI swallows
that checker's exit code` (`TODO.md`). Every claim below is measured at this tree; the two numbered requests
are the parts this iteration is NOT allowed to act on.

WHAT WAS WRONG, and why it survived. `node ralph/scripts/gate-selftest.mjs` exited 1 with
"18 report(s) checked, 4 invariant violation(s)" and `node ralph/scripts/status-report.mjs` printed
`INSTRUMENT NOT SOUND`, marking `example length` and `attribute density` INVALID on all 17 measured routes.
Two distinct causes, both in the instrument rather than in any page:

1. `no-react-to-react-scoring` fired on the mere PRESENCE of a React-to-React pair
   (`(r.metrics?.reactToReactBlocks ?? 0) > 0`), while its own `why` and its own fixture name
   (`react-to-react-scored`) say the defect is the pair being ADMITTED TO SCORING. `snippet-ergonomics.mjs`
   already excludes such blocks from every metric and raises a P0 for them, so the invariant was
   UNSATISFIABLE by correct behaviour: a page that excluded its React block and said so was flagged exactly
   like a page that had scored the paste. That is the "checker that disagrees with its subject" that
   `lib/metric-invariants.mjs`'s own header exists to end.
2. Nothing forbade publishing a `score` when the extraction had measured nothing to score. The reports on
   disk published 40 (direction-provider: 0 blocks extracted on BOTH sides), 28 and 8 (merge-props,
   use-render: all 5 extracted blocks excluded from scoring). Every ratio behind those numbers came from an
   empty input, and an empty input scores 100% on every ratio by default — the same arithmetic that
   `no-blocks-scored` already documents for the CI run of 2026-09-16.

THE FIX (this iteration, in the instrument only; no page and no `crates/**` file was touched). The rule now
has one home, in `lib/metric-invariants.mjs`, so the producer and the checker cannot drift: the audit-side
invariants `no-react-to-react-scoring` (tightened: fires only when a score was PUBLISHED over such a pair) and
`no-score-from-excluded-extraction` (new: fires when `blocksExcludedFromScoring >= leptosSnippets > 0` and a
score was published), plus `scoreWithheldReason()`, the PRODUCER side of the same two rules, which
`snippet-ergonomics.mjs` now imports and uses to publish `score: null` with `scoreWithheld` +
`scoreWithheldReason` instead of a number. Withholding does not soften the verdict — such a page still FAILS
(exit 1) — it only stops a number nobody measured from being quotable. `gate-selftest.mjs` gained fixtures in
BOTH directions (`react-blocks-excluded-not-scored` and `healthy-extraction` must be ACCEPTED, or a broken
gate would hide every real score) and a browser-free unit test of the producer's rule; verified non-vacuous by
an A/B that broke the rule and confirmed exit 1. The three stale reports were VOIDED into the refusal shape
the consumers already handle (`refused: true, score: null`, original counters preserved under
`originalReport`) — this wrote no measurement; it removed a fabricated one — and the underlying page defect
they exposed (those pages still teach upstream's React API) is owned by `docs-chrome: snippet translation
(batch 4)`, whose `routes:` field names exactly use-render, merge-props and direction-provider.

REQUEST 1 — `.github/workflows/measure-port.yml` swallows the checker's verdict. Line 188 runs
`node ralph/scripts/gate-selftest.mjs || true`, so the one gate that says "no number from these reports may be
quoted" cannot fail the run, and its output is not committed either. Measured need: the checker's exit code
must reach the job's verdict (or its output must be committed the way `scorecard.jsonl` is). NOT EDITED:
`CONTEXT.md` reserves release/workflow paths for the owner, and an iteration does not grant itself that
authority. The local harness has the same gap — `grep -n "gate-selftest\|status-report"
ralph/scripts/run-regression.sh` matches NOTHING, so no gate runs the checker outside CI.

REQUEST 2 — CI does not commit the per-route snippet reports, so a stale one can never be refreshed by CI.
`measure-port.yml:181` copies `ralph/logs/visual/${name}-snippets.json` into the shard artifact, but the commit
step adds only `ralph/generated/scorecard.jsonl` (line 248). Consequence, measured: this box refuses browsers
by design (`ralph/generated/env-health.json`: `browser: DEGRADED`) and `snippet-ergonomics.mjs` therefore
refuses here (exit 2), so the three voided reports cannot be re-measured from this box at all — they will read
UNMEASURED until a browser-capable run regenerates them. The honest state is UNMEASURED, not a number; if the
owner wants them re-measured rather than merely voided, the reports need to become committed outputs of the
measure job.

ALSO MEASURED, for the tooling lane (not this item's to fix): `check-page.mjs` treats any report older than
the run as stale and reads its axes UNMEASURED (`check-page.mjs:118`), so on this box those three routes'
ergonomics axes were already UNMEASURED before this change — what the change removes is the INVALID flag on
the two size axes of all 17 routes, i.e. the scorecard can now be read as a product measure again.

ADDENDUM 2026-09-17 — the collapsed `direction-provider` extraction has a root cause, and it is not the page.
Its voided report carries `route: "react/components/direction-provider"`, which is NOT a route in this port: the
page is served at `react/utils/direction-provider` (`ralph/generated/routes.json:17`,
`crates/docs-app/src/lib.rs:80`, the sidenav entry `crates/docs-app/src/chrome.rs:123`, and the page's own module
docs, `crates/docs-app/src/pages/direction_provider_page.rs:2,273`). The extractor therefore fetched a URL that
exists on neither side, both sides returned zero blocks, and the empty-input ratios scored 100% by default —
hence `score 40` from `upstreamElements 0, leptosElements 0, snippetLanguages.total 0`. Two consequences for
whoever picks this up: (1) that route's two size axes were never a page failure and re-measuring them is a
one-line invocation with the correct route; (2) `ralph/logs/visual/<name>-snippets.json` is keyed by the route's
BASENAME (`snippet-ergonomics.mjs`: `const name = route.split('/').pop()`), so a wrong route spelling is written
into the artifact under the RIGHT filename — the file looked healthy-by-name while naming a page that does not
exist. Worth a guard in the producer (reject a route that is not in `ralph/generated/routes.json` before
launching a browser); recorded here rather than implemented, since this iteration's objective was the invariant
contract, and a gate edit needs its own before/after evidence.

FINDINGS 2026-09-17 — `library: menu — the view layer (Positioner/Portal/Popup/Item) is a fabricated stub, not a port`,
while porting `Menu.Portal`, `Menu.Positioner` and `Menu.Popup`. Six places where the SPEC or the TREE does not yet
carry what upstream's code reads. None of them was resolved by editing a spec or by inventing a value in the port;
each is recorded with the measurement that shows the gap.

1. `MenuPortal.tsx:29-31` decides `portalOwnerRole` from the CONTEXT parent, and its own inline comment says why:
   "`parent` comes from context (the `Menu.Root` position), unlike the store's `parent`, which a detached trigger
   overwrites with its own." The port's `MenuRootContextValue` carries ONLY `store` (`crates/leptos-ui/src/menu/
   store.rs:183-187`), and the parent the port has is the store's extra-state slot (`store.rs:80-81`). For the role
   this is harmless today (only the discriminant is needed, and a detached trigger does not change the parent's
   TYPE), but the distinction is a real behavioural difference for a detached trigger and the port cannot express
   it yet. `menu_portal_owner_role` documents this; adding a `parent` field to the context is Root's work.

2. `MenuPositioner.tsx:105` reads `parent.context.orientation` and `:267` reads `parent.context.modal`. The port's
   `MenuParent::Menubar` is a unit variant that deliberately carries no menubar context handle
   (`store.rs:136-140`) and the menubar unit is `not-started` in the ledger, so neither value is reachable. The
   resolution function therefore takes `menubar_orientation` as a PARAMETER and the component passes the menubar's
   documented horizontal default; `menubar_modal` arrives as a `false` parameter. Both are named in the port's
   module docs; neither is guessed at inside the resolution.

3. `MenuPositioner.tsx:88-95` reads `parent.context?.anchor` for a context-menu parent, and `:304-308` its shared
   internal-backdrop ref. `MenuParent::ContextMenu` is likewise a unit variant (`store.rs:139`), and the
   context-menu unit has no ledger item at all. The context-menu ARM of the resolution is ported and tested
   (align default `start`, the `2`/`-5` offset pair, `arrowPadding: 0`, the shift override); the anchor and the
   backdrop ref it also reads are not representable and are named as deferred.

4. `MenuPopup.tsx:55-64` / `MenuPositioner.tsx:302-312` render `FloatingFocusManager` and `InternalBackdrop`.
   `InternalBackdrop` is NOT ported anywhere in this tree (`grep -rln "InternalBackdrop" crates/leptos-ui-internals/
   src/` returns nothing), and the floating focus manager, while ported (3294 lines), is deferred by the sibling
   popover part for the same reason (`popover/parts.rs:455-467`). The port therefore renders neither element, and
   the pure predicates the backdrop gate needs (`menu_positioner_should_render_backdrop`,
   `menu_positioner_backdrop_cutout`) are ported and tested so the element lands against a proven predicate rather
   than being re-derived. Consequence, stated plainly: until the focus manager is wired, `shouldRenderGuards` on the
   ported portal cannot turn true, so the portal's guard spans and hidden `aria-owns` owner stay unrendered — the
   same deferral, one layer up.

5. The floating-tree coordination (`MenuPositioner.tsx:138-231`) is the only producer of `siblingOpen` closes, the
   `itemhover` branch-closing, and the `menuopenchange` re-broadcast that `MenuPopup.tsx:66-77` and the menubar
   consume. Nothing in this tree emits `menuopenchange`: `grep -rn "menuopenchange" crates/` returns nothing, and
   upstream's own second emitter is `Menubar.tsx:121` — the unit the ledger names as this item's precondition. The
   reasons it produces already exist (`menu/store.rs:59`), so this is an emitter gap, not a vocabulary gap.

6. `MenuPopup.tsx:38` reads `store.popupProps`, whose writer is `MenuRoot.tsx:571-600` — `FOCUSABLE_POPUP_PROPS`
   plus `{ id: floatingId, role: 'menu', 'aria-orientation': orientation === 'horizontal' ? 'horizontal' :
   undefined, 'aria-labelledby': activeTriggerElement?.id, onMouseMove, onClick }`. The port's `MenuRoot` publishes
   no `popupProps` bag (`grep -rn "popup_props" crates/leptos-ui/src/menu/` returns nothing), which is the same
   finding the item part recorded for `itemProps` — except that for the popup the bag is NOT a no-op: four of its
   members are behaviour-relevant. The port renders the members that have a store source (`role`, `id`,
   `aria-labelledby`, `data-rootownerid`) and leaves `aria-orientation` and the two hover handlers
   (`onMouseMove`/`onClick`, which write `allowMouseEnter`/`hoverEnabled`) to the Root-publishes-`popupProps`
   checkpoint. Both states' fields already exist in the port (`allow_mouse_enter`, `hover_enabled`), so that
   checkpoint is a publishing change, not new machinery.
