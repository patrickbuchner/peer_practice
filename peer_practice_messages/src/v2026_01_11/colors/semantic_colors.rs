use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PaletteColor {
    Text,
    Subtext1,
    Subtext0,
    Overlay2,
    Overlay1,
    Overlay0,
    Surface2,
    Surface1,
    Surface0,
    Base,
    Mantle,
    Crust,
}

impl PaletteColor {
    pub const fn css_var(self) -> &'static str {
        match self {
            PaletteColor::Text => "var(--text)",
            PaletteColor::Subtext1 => "var(--subtext1)",
            PaletteColor::Subtext0 => "var(--subtext0)",
            PaletteColor::Overlay2 => "var(--overlay2)",
            PaletteColor::Overlay1 => "var(--overlay1)",
            PaletteColor::Overlay0 => "var(--overlay0)",
            PaletteColor::Surface2 => "var(--surface2)",
            PaletteColor::Surface1 => "var(--surface1)",
            PaletteColor::Surface0 => "var(--surface0)",
            PaletteColor::Base => "var(--base)",
            PaletteColor::Mantle => "var(--mantle)",
            PaletteColor::Crust => "var(--crust)",
        }
    }
}

/// “Intent” colors for UI components (background/fill).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticColor {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

impl SemanticColor {
    pub const fn css_var(self) -> &'static str {
        match self {
            SemanticColor::Primary => "var(--primary-color)",
            SemanticColor::Secondary => "var(--secondary-color)",
            SemanticColor::Success => "var(--success-color)",
            SemanticColor::Warning => "var(--warning-color)",
            SemanticColor::Danger => "var(--danger-color)",
        }
    }
}

/// Text colors meant to be paired with [`SemanticColor`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SemanticTextColor {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

impl SemanticTextColor {
    pub const fn css_var(self) -> &'static str {
        match self {
            SemanticTextColor::Primary => "var(--primary-text-color)",
            SemanticTextColor::Secondary => "var(--secondary-text-color)",
            SemanticTextColor::Success => "var(--success-text-color)",
            SemanticTextColor::Warning => "var(--warning-text-color)",
            SemanticTextColor::Danger => "var(--danger-text-color)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackgroundColor {
    Base,
    Weakest,
    Weak,
    Strong,
    Strongest,
}

impl BackgroundColor {
    pub const fn css_var(self) -> &'static str {
        match self {
            BackgroundColor::Base => "var(--bg-base-color)",
            BackgroundColor::Weakest => "var(--bg-weakest-color)",
            BackgroundColor::Weak => "var(--bg-weak-color)",
            BackgroundColor::Strong => "var(--bg-strong-color)",
            BackgroundColor::Strongest => "var(--bg-strongest-color)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BackgroundTextColor {
    Base,
    Weakest,
    Weak,
    Strong,
    Strongest,
}

impl BackgroundTextColor {
    pub const fn css_var(self) -> &'static str {
        match self {
            BackgroundTextColor::Base => "var(--bg-base-text)",
            BackgroundTextColor::Weakest => "var(--bg-weakest-text)",
            BackgroundTextColor::Weak => "var(--bg-weak-text)",
            BackgroundTextColor::Strong => "var(--bg-strong-text)",
            BackgroundTextColor::Strongest => "var(--bg-strongest-text)",
        }
    }
}
