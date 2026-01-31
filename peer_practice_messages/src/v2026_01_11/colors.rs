pub mod accent_colors;
pub mod semantic_colors;

use accent_colors::AccentColor;
use semantic_colors::{
    BackgroundColor, BackgroundTextColor, PaletteColor, SemanticColor, SemanticTextColor,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Color {
    Palette(PaletteColor),
    Semantic(SemanticColor),
    SemanticText(SemanticTextColor),
    Background(BackgroundColor),
    BackgroundText(BackgroundTextColor),
    Accent(AccentColor),
}

impl Color {
    pub const fn css_var(self) -> &'static str {
        match self {
            Color::Palette(c) => c.css_var(),
            Color::Semantic(c) => c.css_var(),
            Color::SemanticText(c) => c.css_var(),
            Color::Background(c) => c.css_var(),
            Color::BackgroundText(c) => c.css_var(),
            Color::Accent(c) => c.css_var(),
        }
    }
}
