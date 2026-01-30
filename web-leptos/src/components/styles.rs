use crate::components::theme::Theme;
pub mod button_class;
pub mod card;
pub mod chat;
pub mod cluster;
pub mod color;
pub mod event_card;
pub mod form_class;
pub mod ideas;
pub mod layout;
pub mod nav_menu;
pub mod navbar;
pub mod shadow;
pub mod stack;
pub mod status;
pub mod text_class;
pub mod toast;

pub fn pin_theme(is_complete: bool) -> &'static str {
    if is_complete {
        Theme::Primary.as_str()
    } else {
        Theme::Secondary.as_str()
    }
}
