use leptos::*;
use leptos_ui::*;
use leptos_testing::TestView;

#[test]
fn test_collapsible_root() {
    let view = view! {
        <CollapsibleRoot default_open=true>
            <CollapsibleTrigger>"Click me"</CollapsibleTrigger>
            <CollapsiblePanel>
                <p>"Content"</p>
            </CollapsiblePanel>
        </CollapsibleRoot>
    };
    
    let test_view = TestView::new(view);
    assert!(test_view.contains("Click me"));
    assert!(test_view.contains("Content"));
}

#[test]
fn test_collapsible_trigger_interaction() {
    let view = view! {
        <CollapsibleRoot default_open=false>
            <CollapsibleTrigger>"Toggle"</CollapsibleTrigger>
            <CollapsiblePanel>
                <p>"Hidden content"</p>
            </CollapsiblePanel>
        </CollapsibleRoot>
    };
    
    let test_view = TestView::new(view);
    assert!(test_view.contains("Toggle"));
    // Initially content should not be visible when closed
    assert!(!test_view.contains("Hidden content"));
}

#[test]
fn test_collapsible_disabled() {
    let view = view! {
        <CollapsibleRoot disabled=true>
            <CollapsibleTrigger>"Disabled trigger"</CollapsibleTrigger>
            <CollapsiblePanel>
                <p>"Content"</p>
            </CollapsiblePanel>
        </CollapsibleRoot>
    };
    
    let test_view = TestView::new(view);
    assert!(test_view.contains("Disabled trigger"));
    assert!(test_view.contains("Content"));
}