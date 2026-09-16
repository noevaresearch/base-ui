#!/usr/bin/env node
// source-scope.selftest.mjs — POSITIVE CONTROLS for the shared classifier that four gates now share
// (`check-react-mentions.mjs` source + rendered modes, `check-package-alias.mjs` rule 4, and
// `check-visual-budget`'s snippet-language consumers via snippet-lang).
//
// WHY THIS EXISTS
// ---------------
// The classifier decides which findings are snippet LANGUAGE (re-homed to the `docs-chrome: snippet
// translation` items) and which are page copy (fatal here). Narrowing a gate is only honest if the
// narrowing cannot also swallow a REAL defect — so each excuse has a control that must STILL fail:
//   * an install COMMAND is fatal even inside a `code_block` argument (the one hole that would matter);
//   * `props.children` stays a defect inside a JSX/TSX code block, and in rendered prose;
//   * an import outside a code block stays a defect;
//   * a `#[cfg(test)]` fixture is NOT page copy, but the line right AFTER the module is.
// A bare assertion passing because the classifier stopped matching anything is exactly the failure
// mode this file exists to make impossible.
//
// USAGE: node ralph/scripts/lib/source-scope.selftest.mjs   (exit 0 = all controls held)

import {
  cfgTestRanges, inRanges, codeBlockRanges, codeBlockAt,
  isSnippetLanguageHit, propsChildrenIsRustFieldAccess, INSTALL_COMMAND_RE,
} from './source-scope.mjs';

let failed = 0;
const check = (name, actual, expected) => {
  const ok = actual === expected;
  if (!ok) failed++;
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${name}${ok ? '' : ` — expected ${expected}, got ${actual}`}`);
};

// ---- a fixture with one of every shape the classifier has to tell apart -------------------------
const FIXTURE = `use crate::code_block;
// prose: install line (page copy)
pub fn page() {
    view! {
        {code_block(
            Lang::Jsx,
            "Anatomy",
            "import { Checkbox } from '@base-ui/react/checkbox';

<Checkbox.Root>
  <Checkbox.Indicator />
</Checkbox.Root>;",
        )}
        {code_block(
            Lang::Rust,
            "Port",
            "pub fn text_element(props: TextProps) -> RawElementView {
    let children = props.children;
}",
        )}
        {code_block(
            Lang::Jsx,
            "Children",
            "function App(props) {
  return <div>{props.children}</div>;
}",
        )}
    }
}

#[cfg(test)]
mod tests {
    const UPSTREAM_ANATOMY: &str = "import { Checkbox } from '@base-ui/react/checkbox';";
}
`;

const ranges = codeBlockRanges(FIXTURE);
const tests = cfgTestRanges(FIXTURE);
const lineOf = (needle) => FIXTURE.split('\n').findIndex((l) => l.includes(needle)) + 1;

// 1. the scanner finds exactly the three code_block calls, with their declared Lang
check('three code_block ranges', ranges.length, 3);
check('block 1 lang jsx', ranges[0].lang, 'jsx');
check('block 2 lang rust', ranges[1].lang, 'rust');
check('block 3 lang jsx', ranges[2].lang, 'jsx');

// 2. a #[cfg(test)] module is not page copy; the line after it is
const testLine = lineOf('UPSTREAM_ANATOMY');
check('fixture line is inside #[cfg(test)]', inRanges(tests, testLine), true);
check('the import above the fixture is NOT in a test', inRanges(tests, lineOf('use crate::code_block')), false);

// 3. an install COMMAND outranks the snippet classification — the one hole that would matter
const installLine = lineOf('@base-ui/react/checkbox', 1);
check('import inside a Jsx block IS snippet language', isSnippetLanguageHit('    "import { X } from \'@base-ui/react/x\';",', installLine, ranges), true);
check('an npm install line is NOT excused as snippet language',
  isSnippetLanguageHit('"npm install @base-ui/react"', installLine, ranges), false);
check('INSTALL_COMMAND_RE matches pnpm add', INSTALL_COMMAND_RE.test('pnpm add @base-ui/react'), true);
check('INSTALL_COMMAND_RE matches yarn add', INSTALL_COMMAND_RE.test('yarn add @base-ui/react'), true);

// 4. props.children: Rust field access excused, upstream idiom still a defect
check('Rust block: props.children is a field access',
  propsChildrenIsRustFieldAccess('let children = props.children;', { block: codeBlockAt(ranges, lineOf('let children = props.children;')) }), true);
check('Jsx block: props.children is a defect',
  propsChildrenIsRustFieldAccess('  return <div>{props.children}</div>;', { block: codeBlockAt(ranges, lineOf('return <div>{props.children}')) }), false);
check('prose outside any code block reads as Rust source',
  propsChildrenIsRustFieldAccess('let children = props.children;', { block: null }), true);
check('rendered prose is never excused',
  propsChildrenIsRustFieldAccess('a function that accepts props.children', { renderedCodeText: 'a function that accepts props.children' }), false);
check('rendered Rust snippet is excused',
  propsChildrenIsRustFieldAccess('let children = props.children;', { renderedCodeText: 'fn text_element(props: TextProps) -> RawElementView { let children = props.children; }' }), true);
check('an unambiguous React API always wins',
  propsChildrenIsRustFieldAccess('const x = props.children; const [a] = useState(0);', { block: null }), false);

// 5. a real install target written as prose import stays page copy (not a code block)
check('prose import outside a code block is not snippet language',
  isSnippetLanguageHit('// see from \'@base-ui/react\'', 1, ranges), false);

console.log(`\n${failed === 0 ? 'ALL CONTROLS HELD' : `${failed} CONTROL(S) FAILED`}`);
process.exit(failed === 0 ? 0 : 1);
