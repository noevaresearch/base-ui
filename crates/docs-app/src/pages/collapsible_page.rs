use leptos::prelude::*;
use leptos_ui::{CollapsiblePanel, CollapsibleRoot, CollapsibleTrigger};

/// The `docs/src/app/(docs)/react/components/collapsible/demos/hero/tailwind/index.tsx`
/// demo, ported to Leptos on the real `leptos_ui` Collapsible component.
/// The upstream demo passes Tailwind classes as component props; the ported
/// components do not yet forward arbitrary props, so the same classes ride on
/// wrapper elements with identical layout/visual semantics.
#[component]
pub fn CollapsibleHeroDemo() -> impl IntoView {
    view! {
        <div class="flex min-h-36 w-48 flex-col justify-center text-neutral-950 dark:text-white">
            <CollapsibleRoot>
                <CollapsibleTrigger>
                    <span class="flex h-8 w-full items-center justify-between gap-2 rounded-none border border-neutral-950 bg-white pl-3 pr-2 text-sm leading-none whitespace-nowrap font-normal text-neutral-950 select-none">
                        "Recovery keys"
                        <svg
                            class="transition-transform duration-100"
                            width="16"
                            height="16"
                            viewBox="0 0 16 16"
                            fill="currentColor"
                            style="display: block"
                        >
                            <path d="M6 12V4l4.5 4z" />
                        </svg>
                    </span>
                </CollapsibleTrigger>
                <CollapsiblePanel>
                    <div class="flex h-9 flex-col justify-end overflow-hidden text-sm">
                        <div class="flex flex-col gap-2 px-3.5 py-2">
                            <div>"alien-bean-pasta"</div>
                            <div>"wild-irish-burrito"</div>
                            <div>"horse-battery-staple"</div>
                        </div>
                    </div>
                </CollapsiblePanel>
            </CollapsibleRoot>
        </div>
    }
}
