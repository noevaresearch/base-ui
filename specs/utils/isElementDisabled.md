# `isElementDisabled` — behavior spec

Unit: `packages/utils/src/isElementDisabled` (single-file util → crate `leptos-ui-utils`, per its
`TODO.md` entry, which has no `wraps-external:` field, so no external-package delegation applies
`TODO.md:79-83`).
Source of truth: **none of its own** — the unit has no dedicated test suite; the generated manifest
records `testFiles: []` for it (`ralph/generated/utils.json:132-140`). The unit's name appears in
exactly one definition file plus three source imports; no test file imports it
(`packages/utils/src/isElementDisabled.ts:1-7`,
`packages/react/src/select/root/SelectRoot.tsx:8`,
`packages/react/src/internals/composite/root/useCompositeRoot.ts:3`,
`packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.tsx:3`).
Scope note: with no test files to mine, every behavioral claim below is marked
`UNVERIFIED — inferred from ...` with citations into the unit's own source (or, where noted, a
consumer's source), per the template's UNVERIFIED rule. Consumer test suites (Select, Menu,
composite internals) belong to those units and were not opened.

## Public API surface (props, parts, subcomponents)

- Single module `packages/utils/src/isElementDisabled.ts`; no components, props, parts, or
  subcomponents — this is a pure DOM predicate.
- Single export: `isElementDisabled(element: HTMLElement | null): boolean`
  (`packages/utils/src/isElementDisabled.ts:1-7`).
- The declared parameter type is `HTMLElement | null`, but the implementation tests with loose
  equality (`element == null`), so a runtime `undefined` is also accepted and treated the same as
  `null` (`packages/utils/src/isElementDisabled.ts:3`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:3`, no test asserts this.
- Library consumers (usage pattern only; none are tests):
  - Select root uses it as the per-index `disabledIndices` predicate over typeahead item elements
    (`packages/react/src/select/root/SelectRoot.tsx:384`), deliberately relying on its
    attribute-only semantics so hidden force-mounted items used for closed-trigger typeahead are
    not dropped by the visibility filter that `disabledIndices` sidesteps
    (`packages/react/src/select/root/SelectRoot.tsx:379-383`). UNVERIFIED — inferred from source,
    no test asserts this.
  - Composite root consults it when handling arrow keys on a native input: the composite only
    intercepts the keys when the target input is NOT disabled by this predicate, otherwise the
    event falls through to native textbox behavior
    (`packages/react/src/internals/composite/root/useCompositeRoot.ts:229`). UNVERIFIED — inferred
    from source, no test asserts this.
  - Menu submenu-trigger uses it inside a development-only effect that warns when the rendered
    trigger element is disabled by attributes while the component's own `disabled` state is not
    (`packages/react/src/menu/submenu-trigger/MenuSubmenuTrigger.tsx:104-111`). UNVERIFIED —
    inferred from source, no test asserts this.

## State model (controlled/uncontrolled, defaults, transitions)

- No state of any kind: no controlled/uncontrolled concept, no module-level cache, no
  memoization. The function is a pure predicate that re-reads the passed element's attributes on
  every call (`packages/utils/src/isElementDisabled.ts:1-7`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:1-7`, no test asserts this.
- The result is a pure function of the element's current attributes: `true` when the element is
  null-ish, carries a `disabled` attribute (any value), or carries `aria-disabled` equal to the
  exact string `'true'`; `false` otherwise (`packages/utils/src/isElementDisabled.ts:2-6`).
  UNVERIFIED — inferred from `packages/utils/src/isElementDisabled.ts:2-6`, no test asserts this.
- The null-ish case short-circuits to `true`, i.e. "no element" is treated as "disabled" by this
  predicate (`packages/utils/src/isElementDisabled.ts:3`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:3`, no test asserts this.

## Keyboard interactions

N/A — the unit binds no event listeners and contains no keyboard handling
(`packages/utils/src/isElementDisabled.ts:1-7`). Its only keyboard relevance is indirect: the
composite root's arrow-key handler uses it as a gate
(`packages/react/src/internals/composite/root/useCompositeRoot.ts:229`).

## Focus management

N/A — no focus behavior exists in the unit (`packages/utils/src/isElementDisabled.ts:1-7`).

## Accessibility (roles, aria-*, id linking)

- The unit is itself an accessibility-semantics helper: it encodes the convention that a disabled
  control is conveyed either by the native `disabled` attribute or by `aria-disabled="true"`
  (`packages/utils/src/isElementDisabled.ts:4-5`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:4-5`, no test asserts this.
- `aria-disabled` is compared strictly to the string `'true'`; other values (e.g. `"false"`) do
  not count as disabled (`packages/utils/src/isElementDisabled.ts:5`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:5`, no test asserts this.
- The unit writes nothing: no roles, no aria attributes set, no id linking — it only reads
  attributes (`packages/utils/src/isElementDisabled.ts:1-7`).

## DOM structure & portal behavior

N/A — the unit renders nothing and has no portal behavior; it performs attribute reads on a node
it is handed (`packages/utils/src/isElementDisabled.ts:1-7`).

## Events (names, payload shape, bubbling, preventDefault semantics)

N/A — the unit emits no events, observes none, and has no bubbling or preventDefault semantics
(`packages/utils/src/isElementDisabled.ts:1-7`).

## Edge cases (rapid interactions, unmount, nesting)

- Null-ish element: returns `true` (treated as disabled) via the leading `element == null`
  short-circuit (`packages/utils/src/isElementDisabled.ts:3`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:3`, no test asserts this.
- Attribute presence, not value: `hasAttribute('disabled')` is `true` for any attribute value, so
  even the invalid `disabled="false"` counts as disabled
  (`packages/utils/src/isElementDisabled.ts:4`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:4`, no test asserts this.
- Property-blind ("attribute-only"): only attributes are consulted; an element whose DOM
  `disabled` property is `true` without the attribute set (or that is natively disabled through an
  ancestor, e.g. inside a disabled `<fieldset>`) is reported as enabled. The Select root comment
  explicitly relies on this attribute-only behavior
  (`packages/utils/src/isElementDisabled.ts:4`,
  `packages/react/src/select/root/SelectRoot.tsx:381`). UNVERIFIED — inferred from source, no test
  asserts this.
- No ancestor walk: both checks inspect only the passed element; an enabled element inside a
  disabled container is reported enabled (`packages/utils/src/isElementDisabled.ts:4-5`).
  UNVERIFIED — inferred from `packages/utils/src/isElementDisabled.ts:4-5`, no test asserts this.
- No transient state or lifecycle: nothing to race across rapid repeated calls, no subscriptions,
  no cleanup paths; a detached element is still evaluated from its retained attributes
  (`packages/utils/src/isElementDisabled.ts:1-7`). UNVERIFIED — inferred from
  `packages/utils/src/isElementDisabled.ts:1-7`, no test asserts this.
- Nesting/reentrancy: N/A beyond the above — a pure function with no shared mutable state
  (`packages/utils/src/isElementDisabled.ts:1-7`).

## Shared harness dependencies

None of its own — the unit has no test files (`ralph/generated/utils.json:132-140`), so there is
no harness to read. Its consumers' suites (Select, Menu, composite internals) belong to those
units' spec scopes.
