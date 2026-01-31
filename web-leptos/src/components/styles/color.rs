use peer_practice_shared::colors::accent_colors::AccentColor;
use peer_practice_shared::colors::semantic_colors::{BackgroundColor, SemanticColor};

#[derive(Clone, Copy)]
pub enum PaletteStyle {
    AccentSwatch,
}

impl PaletteStyle {
    pub fn with_accent(self, accent_var: &AccentColor) -> String {
        match self {
            PaletteStyle::AccentSwatch => {
                format!("--accent: {}; width: 100%;", accent_var.css_var())
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum StatusColor {
    Connected,
    Disconnected,
}

impl StatusColor {
    pub const fn as_str(self) -> &'static str {
        match self {
            StatusColor::Connected => SemanticColor::Success.css_var(),
            StatusColor::Disconnected => SemanticColor::Danger.css_var(),
        }
    }

    pub const fn from_connected(connected: bool) -> StatusColor {
        if connected {
            StatusColor::Connected
        } else {
            StatusColor::Disconnected
        }
    }
}

#[derive(Clone, Copy)]
pub enum SvgStrokeColor {
    StatusOutline,
}

impl SvgStrokeColor {
    pub const fn as_str(self) -> &'static str {
        match self {
            SvgStrokeColor::StatusOutline => "#111827",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum CssVar {
    BgStrong,
    BgStrongest,
    Teal,
}

impl CssVar {
    pub const fn as_str(self) -> &'static str {
        match self {
            CssVar::BgStrongest => BackgroundColor::Strongest.css_var(),
            CssVar::BgStrong => BackgroundColor::Strong.css_var(),
            CssVar::Teal => AccentColor::Teal.css_var(),
        }
    }
}

pub fn chat_name_style(color: AccentColor) -> String {
    format!("color: {};", color.css_var())
}

pub fn chat_border_style(color: AccentColor) -> String {
    format!("border-color: {};", color.css_var())
}

#[derive(Clone, Copy)]
pub enum ShadowColor {
    Base,
    Green,
    Teal,
    Sky,
    Mauve,
    Lavender,
}

impl ShadowColor {
    pub const fn as_str(self) -> &'static str {
        match self {
            ShadowColor::Base => "base",
            ShadowColor::Green => "green",
            ShadowColor::Teal => "teal",
            ShadowColor::Sky => "sky",
            ShadowColor::Mauve => "mauve",
            ShadowColor::Lavender => "lavender",
        }
    }
}
