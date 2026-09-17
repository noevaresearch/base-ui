pub mod accordion;
mod alert_dialog;
mod autocomplete;
pub mod avatar;
mod button;
pub mod checkbox;
mod checkbox_group;
pub mod collapsible;
mod combobox;
mod context_menu;
mod context_menu_tests;
mod dialog;
mod drawer;
pub mod field;
pub mod fieldset;
pub mod form;
mod input;
pub mod menu;
mod menubar;
pub mod meter;
mod navigation_menu;
mod number_field;
pub mod otp_field;
mod popover;
mod popover_tests;
mod preview_card;
pub mod progress;
mod separator;
pub mod radio;
pub mod radio_group;
pub mod switch;
mod toggle;
pub mod toggle_group;

pub use accordion::*;
pub use alert_dialog::*;
pub use autocomplete::*;
pub use avatar::*;
pub use button::*;
pub use checkbox::*;
pub use checkbox_group::*;
pub use collapsible::*;
pub use combobox::*;
pub use context_menu::*;
pub use dialog::*;
pub use drawer::*;
pub use field::*;
pub use fieldset::*;
pub use form::*;
pub use input::*;
pub use menu::*;
pub use menubar::*;
pub use meter::*;
pub use navigation_menu::*;
pub use number_field::*;
pub use otp_field::{
    NormalizeValueFn, OtpChangeEventDetails, OtpChangeHandler, OtpCharset, OtpFieldInputProps,
    OtpFieldInputState, OtpFieldRootContextValue, OtpFieldRootProps, OtpFieldRootState,
    OtpGenericEventDetails, OtpGenericHandler, OtpValidationConfig, OtpValidationType,
    REASON_INPUT_CHANGE, REASON_INPUT_CLEAR, REASON_INPUT_PASTE, REASON_KEYBOARD,
    get_otp_validation_config, normalize_otp_value, normalize_otp_value_with_details,
    otp_field_separator, provide_otp_composite_list, remove_otp_character, replace_otp_value,
    strip_otp_whitespace, use_otp_field_input, use_otp_field_root, use_otp_field_root_context,
};
// NOT `pub use otp_field::*` — the three namespaced parts (`OTPField::Root`/`Input`/`Separator`)
// deliberately stay inside their module (and the `*Props` structs the `#[component]` macro
// generates beside them, which would otherwise collide with `input::InputProps` and
// `separator::SeparatorProps` at this root). The crate root flattens each component's public
// surface by convention, and for the parts that convention is exactly wrong: upstream teaches
// `<OTPField.Input>`, so the port teaches `<OTPField::Input>` — reached through the namespace, not
// through a bare `Input` that means a different component. The alias below is that namespace.
pub use popover::*;
pub use preview_card::*;
pub use progress::*;
pub use radio::*;
pub use radio_group::*;
pub use separator::*;
pub use switch::*;
pub use toggle::*;
pub use toggle_group::*;

// ---------------------------------------------------------------------------
// The capitalised component aliases — the path a consumer writes in `view!`
// ---------------------------------------------------------------------------
//
// Upstream's docs spell a component's parts with a dot (`<Accordion.Root>`); this port's
// spelling is the same tree with Rust's path separator (`<Accordion::Root>`), which needs a
// *capitalised module* to hang the parts off (CONTRACT.md requirement 1; the macro-level pin
// is `crates/leptos-ui/tests/ns_component_path.rs`). These aliases are that module: the same
// snake_case module under upstream's own name, so `use leptos_ui::Accordion;` +
// `<Accordion::Root>` resolves. Nothing is renamed or removed — the snake_case path
// (`leptos_ui::accordion::Root`) and every pre-existing `*_view` helper and `Accordion*`
// wrapper keep working.
//
// Scoped to the components whose namespaced surface is built today (the `library: namespaced
// part surface (ported batch)` item); the menus/inputs batches add theirs when they land.
#[allow(non_snake_case)]
pub use self::accordion as Accordion;
#[allow(non_snake_case)]
pub use self::avatar as Avatar;
#[allow(non_snake_case)]
pub use self::checkbox as Checkbox;
#[allow(non_snake_case)]
pub use self::collapsible as Collapsible;
#[allow(non_snake_case)]
pub use self::field as Field;
#[allow(non_snake_case)]
pub use self::fieldset as Fieldset;
#[allow(non_snake_case)]
pub use self::form as Form;
#[allow(non_snake_case)]
pub use self::meter as Meter;
#[allow(non_snake_case)]
pub use self::progress as Progress;
// otp-field's three parts (`OTPField::Root`/`Input`/`Separator`) — the
// `library: otp-field — the namespaced view surface` item. The camel-case form
// `OTPField` is what upstream's docs teach, and the gate normalizes an acronym
// run the same way (`OTPField` -> `otp_field`, `check-part-surface.mjs`'s `snake`).
#[allow(non_snake_case)]
pub use self::otp_field as OTPField;
// The `switch` lane's namespaced surface (`Switch::Root` / `Switch::Thumb`) — the
// `library: switch` item; upstream teaches `<Switch.Root><Switch.Thumb /></Switch.Root>`,
// so the port's spelling is the same tree with Rust's path separator.
#[allow(non_snake_case)]
pub use self::switch as Switch;
// The `radio` lane's namespaced surface (`Radio::Root` / `Radio::Indicator`) — the
// `library: radio` item; upstream teaches `<Radio.Root><Radio.Indicator /></Radio.Root>`,
// so the port's spelling is the same tree with Rust's path separator. Both parts the unit
// documents (`index.parts.ts` re-exports exactly those two) are exposed on the namespace.
#[allow(non_snake_case)]
pub use self::radio as Radio;
// The `menu` lane's namespaced surface — `Menu::Item` today, from the
// `library: menu — the view layer (Positioner/Portal/Popup/Item) is a fabricated stub`
// item. Upstream teaches `<Menu.Item />`, so the port's spelling is the same tree with
// Rust's path separator. The alias is exposed as soon as the first part's body is a real
// port rather than a placeholder: at the time of writing `Menu::Item` is a translated
// counterpart of `MenuItem.tsx`, while `Menu.Popup`/`Menu.Portal`/`Menu.Positioner`
// still carry placeholder bodies and are therefore NOT aliased here — aliasing them would
// be the "ergonomics over a fabricated body" defect the menus-batch item's note names.
#[allow(non_snake_case)]
pub use self::menu as Menu;

#[cfg(test)]
mod accordion_tests;
#[cfg(test)]
#[cfg(test)]
mod alert_dialog_tests;
#[cfg(test)]
mod autocomplete_tests;
#[cfg(test)]
mod avatar_tests;
#[cfg(test)]
mod button_tests;
#[cfg(test)]
mod checkbox_group_tests;
#[cfg(test)]
mod checkbox_tests;
#[cfg(test)]
mod collapsible_tests;
#[cfg(test)]
mod combobox_tests;
#[cfg(test)]
mod dialog_tests;
#[cfg(test)]
mod drawer_tests;
#[cfg(test)]
mod field_tests;
#[cfg(test)]
mod form_tests;

#[cfg(test)]
mod input_tests;
#[cfg(test)]
mod menu_tests;
#[cfg(test)]
mod menu_view_tests;
#[cfg(test)]
mod menubar_tests;
#[cfg(test)]
mod meter_tests;
#[cfg(test)]
mod navigation_menu_tests;
#[cfg(test)]
mod otp_field_view_tests;
#[cfg(test)]
mod preview_card_tests;
#[cfg(test)]
mod progress_tests;
#[cfg(test)]
mod radio_group_tests;
#[cfg(test)]
mod radio_tests;
#[cfg(test)]
mod separator_tests;
#[cfg(test)]
mod switch_tests;
#[cfg(test)]
mod toggle_group_tests;
#[cfg(test)]
mod toggle_tests;
