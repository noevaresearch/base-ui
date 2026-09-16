import re

path = 'crates/docs-app/src/render_test.rs'
src = open(path).read()
marker = '/// The ported code-block chrome (`crate::code_block`)'
idx = src.index(marker)
head = src[:idx]
addition = '''/// The ported code-block chrome (`crate::code_block`), measured on the route that carries the most
/// snippets. `visual-gap-report.mjs` raised this item as the first P0 on every recorded route —
/// "upstream's code carries 41 coloured tokens; this page has 0 — the code is unstyled monochrome"
/// — so the assertion is on the DOM the real `CheckboxPage` mount produces, not on the constants:
/// each of the page's five embedded snippets renders inside upstream's `.CodeBlockRoot` (panel with
/// the fence's own `.mdx` title, a copy control), its code text is preserved character for
/// character (every fidelity probe and the pages' own snippet guards read `pre.textContent`), and
/// its tokens carry the prettylights classes the ported stylesheet colours.
#[wasm_bindgen_test]
fn checkbox_page_code_blocks_render_through_the_ported_chrome() {
    use crate::pages::checkbox_page::CheckboxPage;
    use leptos::mount::mount_to;
    use leptos::prelude::*;

    let container = leptos::prelude::document()
        .create_element("div")
        .expect("create container")
        .dyn_into::<web_sys::HtmlElement>()
        .expect("div as HtmlElement");
    container.set_id("test-mount-root-code-blocks");
    leptos::prelude::document()
        .body()
        .expect("body")
        .append_child(&container)
        .expect("append container");

    let _ = any_spawner::Executor::init_futures_executor();
    std::mem::forget(mount_to(container.clone(), || view! { <CheckboxPage /> }));

    let roots = els(&container, ".CodeBlockRoot");
    assert_eq!(
        roots.len(),
        5,
        "the page's five embedded snippets render as code blocks"
    );

    // The titles are the page's own `.mdx` fence titles (`page.mdx:21-88`), in page order.
    let expected_titles = [
        "Anatomy",
        "Wrapping a label around a checkbox",
        "Sibling label pattern with a native button",
        "Render callback",
        "Using Checkbox in a form",
    ];

    let mut classified_total = 0usize;
    for (index, title) in expected_titles.iter().enumerate() {
        let root = &roots[index];
        let title_el = els(root, ".CodeBlockPanelTitle");
        assert_eq!(
            title_el.len(),
            1,
            "block {index} renders exactly one panel title"
        );
        assert_eq!(
            title_el[0].text_content().unwrap_or_default(),
            *title,
            "block {index}'s panel title must be its fence's title"
        );

        let copy = els(root, "button[aria-label='Copy code']");
        assert_eq!(
            copy.len(),
            1,
            "block {index} renders exactly one copy control"
        );
        assert_eq!(
            els(&copy[0], "svg").len(),
            1,
            "block {index}'s copy control carries the icon"
        );

        let code = els(root, "pre code");
        assert_eq!(code.len(), 1, "block {index} renders one `pre > code`");
        let code = &code[0];
        let total: usize = code
            .get_attribute("data-total-lines")
            .expect("data-total-lines")
            .parse()
            .expect("a line count");
        assert_eq!(
            els(code, ".line").len(),
            total,
            "block {index} declares {total} lines and renders a different number of line spans"
        );
        assert!(
            code.class_list().contains("language-rust"),
            "the port's own snippet is fenced as rust, not as upstream's jsx"
        );
        let text = code.text_content().unwrap_or_default();
        assert!(
            text.starts_with("use leptos::prelude::*;"),
            "block {index}'s text is the port's snippet; it read: {text:?}"
        );

        classified_total += els(
            code,
            ".pl-k, .pl-c1, .pl-s, .pl-en, .pl-ent, .pl-smi, .pl-c, .di-bool, .di-n",
        )
        .len();
    }
    assert!(
        classified_total >= 40,
        "the five blocks carry the prettylights token hooks the stylesheet colours; found {classified_total}"
    );
}
'''
open(path, 'w').write(head + addition)
print('rewrote; lines now', (head + addition).count('\\n'))
