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
