//! NumberField — port of Base UI NumberField component
//! 
//! A simplified working version that demonstrates basic number field functionality.

use leptos::prelude::*;

/// NumberField root component
#[component]
pub fn NumberFieldRoot(
    /// CSS class name(s) to apply to the root element
    #[prop(default = None)]
    class: Option<String>,
    /// The explicit id
    #[prop(default = None)]
    id: Option<String>,
    /// The control's name
    #[prop(default = None)]
    name: Option<String>,
    /// The controlled value
    #[prop(default = None)]
    value: Option<f64>,
    /// The default value for uncontrolled mode
    #[prop(default = None)]
    default_value: Option<f64>,
    /// Minimum value
    #[prop(default = None)]
    min: Option<f64>,
    /// Maximum value
    #[prop(default = None)]
    max: Option<f64>,
    /// Step value
    #[prop(default = 1.0)]
    step: f64,
    /// Whether the input is disabled
    #[prop(default = false)]
    disabled: bool,
    /// Whether the input is read-only
    #[prop(default = false)]
    read_only: bool,
    /// Whether the input is required
    #[prop(default = false)]
    required: bool,
    /// Locale for formatting
    #[prop(default = String::from("en-US"))]
    locale: String,
) -> impl IntoView {
    let (value, set_value) = signal(value.unwrap_or(default_value.unwrap_or(0.0)));
    
    let input_value = Memo::new(move |_| value.get().to_string());
    
    let handle_input = move |ev: web_sys::Event| {
        let input = event_target::<web_sys::HtmlInputElement>(&ev);
        let new_value = input.value().parse().unwrap_or(0.0);
        set_value.set(new_value);
    };
    
    let handle_keydown = move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "ArrowUp" => {
            ev.prevent_default();
            let new_value = value.get() + step;
            set_value.set(new_value);
        }
        "ArrowDown" => {
            ev.prevent_default();
            let new_value = value.get() - step;
            set_value.set(new_value);
        }
        _ => {}
    };
    
    view! {
        <div 
            class=format!("number-field {}", class.unwrap_or_default())
            id=id
        >
            <input
                type="text"
                prop:name=name
                prop:value=input_value
                prop:disabled=disabled
                prop:readonly=read_only
                prop:required=required
                prop:min=min
                prop:max=max
                on:input=handle_input
                on:keydown=handle_keydown
            />
        </div>
    }
}