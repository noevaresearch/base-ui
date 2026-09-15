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
