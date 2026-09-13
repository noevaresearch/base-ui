# Spec Discrepancies

## 2026-09-13 - Context Menu Citation Drift

**Issue**: Citation drift in context-menu specs

**Files affected**:
- specs/library/context-menu/behavior.md
- specs/library/context-menu/implementation.md

**Problem**: Both spec files cite `TODO.md:371-377` but the cited content has drifted. The current content at lines 371-377 of TODO.md contains Phase B library content, not context-menu specific content.

**Cited content (lines 371-377)**:
```
## Phase B — Library components (blocked-by: all Phase A items; crate leptos-ui)

- [x] library: accordion
      crate: leptos-ui
      specs: specs/library/accordion/behavior.md, specs/library/accordion/implementation.md, specs/library/accordion/fixtures.json
      blocked-by: [Phase A complete, library: collapsible]
      status: done
```

**Expected content**: Should cite context-menu specific content or the actual TODO.md entry for library: context-menu.

**Resolution**: This appears to be a citation error in the spec files. The citation should point to the actual context-menu TODO entry (around line 459) or be updated to reflect the correct content.

**Impact**: The citation check fails, but this is a documentation issue, not an implementation issue. The implementation is complete and functional.