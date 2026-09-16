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
mod menu;
mod menubar;
pub mod meter;
mod navigation_menu;
mod number_field;
mod otp_field;
mod popover;
mod popover_tests;
mod preview_card;
pub mod progress;
mod separator;
mod toggle;

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
pub use otp_field::*;
pub use popover::*;
pub use preview_card::*;
pub use progress::*;
pub use separator::*;
pub use toggle::*;

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
mod menubar_tests;
#[cfg(test)]
mod meter_tests;
#[cfg(test)]
mod navigation_menu_tests;
#[cfg(test)]
mod preview_card_tests;
#[cfg(test)]
mod progress_tests;
#[cfg(test)]
mod separator_tests;
#[cfg(test)]
mod toggle_tests;
