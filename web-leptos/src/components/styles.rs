use crate::components::theme::{IntentTheme, Theme};
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
pub mod stack;
pub mod status;
pub mod text_class;
pub mod toast;

pub fn pin_theme(is_complete: bool) -> Theme {
    if is_complete {
        Theme::Intent(IntentTheme::Primary)
    } else {
        Theme::Intent(IntentTheme::Secondary)
    }
}
