use peer_practice_shared::accent_colors::AccentColor;

#[derive(Clone, Copy)]
pub enum AccentName {
    Rosewater,
}

impl AccentName {
    pub const fn as_str(self) -> &'static str {
        match self {
            AccentName::Rosewater => "rosewater",
        }
    }
}

#[derive(Clone, Copy)]
pub enum PaletteStyle {
    AccentSwatch,
}

impl PaletteStyle {
    pub fn with_accent(self, accent_var: &str) -> String {
        match self {
            PaletteStyle::AccentSwatch => {
                format!("--accent: {}; width: 100%;", accent_var)
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
            StatusColor::Connected => "var(--success-color)",
            StatusColor::Disconnected => "var(--danger-color)",
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

#[derive(Clone, Copy)]
pub enum CssVar {
    BgStrong,
    BgStrongest,
    Teal,
}

impl CssVar {
    pub const fn as_str(self) -> &'static str {
        match self {
            CssVar::BgStrongest => "var(--bg-strongest-color)",
            CssVar::Teal => "var(--teal)",
            CssVar::BgStrong => "var(--bg-strong-color)",
        }
    }
}

pub fn chat_accent_style(color: AccentColor) -> String {
    format!("--accent: {};", color.css_var())
}

pub fn chat_name_style(color: AccentColor) -> String {
    format!("color: {};", color.css_var())
}

pub fn chat_border_style(color: AccentColor) -> String {
    format!("border-color: {};", color.css_var())
}