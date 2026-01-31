use peer_practice_shared::colors::Color;
use peer_practice_shared::colors::semantic_colors::{BackgroundColor, SemanticColor};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    /// Background/surface strength ramp (what you currently call weakest..strongest).
    Surface(SurfaceTheme),

    /// Semantic “intent” theme (primary/secondary/success/warning/danger).
    Intent(IntentTheme),

    /// Uses the per-component `--accent` (typically set via inline style).
    Accent,

    /// Low-emphasis/transparent-ish styling.
    Ghost,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceTheme {
    Weakest,
    Weak,
    Base,
    Strong,
    Strongest,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntentTheme {
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
}

impl Theme {
    /// What goes into `data-theme="..."` (keeps CSS compatibility).
    pub const fn as_str(self) -> &'static str {
        match self {
            Theme::Surface(SurfaceTheme::Weakest) => "weakest",
            Theme::Surface(SurfaceTheme::Weak) => "weak",
            Theme::Surface(SurfaceTheme::Base) => "base",
            Theme::Surface(SurfaceTheme::Strong) => "strong",
            Theme::Surface(SurfaceTheme::Strongest) => "strongest",

            Theme::Intent(IntentTheme::Primary) => "primary",
            Theme::Intent(IntentTheme::Secondary) => "secondary",
            Theme::Intent(IntentTheme::Success) => "success",
            Theme::Intent(IntentTheme::Warning) => "warning",
            Theme::Intent(IntentTheme::Danger) => "danger",

            Theme::Accent => "accent",
            Theme::Ghost => "ghost",
        }
    }

    #[allow(dead_code)]
    /// Optional mapping to the shared `Color` token that “drives” the theme.
    /// (Useful when you want to set CSS vars from Rust, or document intent.)
    pub const fn color_token(self) -> Option<Color> {
        match self {
            Theme::Surface(SurfaceTheme::Weakest) => {
                Some(Color::Background(BackgroundColor::Weakest))
            }
            Theme::Surface(SurfaceTheme::Weak) => Some(Color::Background(BackgroundColor::Weak)),
            Theme::Surface(SurfaceTheme::Base) => Some(Color::Background(BackgroundColor::Base)),
            Theme::Surface(SurfaceTheme::Strong) => {
                Some(Color::Background(BackgroundColor::Strong))
            }
            Theme::Surface(SurfaceTheme::Strongest) => {
                Some(Color::Background(BackgroundColor::Strongest))
            }

            Theme::Intent(IntentTheme::Primary) => Some(Color::Semantic(SemanticColor::Primary)),
            Theme::Intent(IntentTheme::Secondary) => {
                Some(Color::Semantic(SemanticColor::Secondary))
            }
            Theme::Intent(IntentTheme::Success) => Some(Color::Semantic(SemanticColor::Success)),
            Theme::Intent(IntentTheme::Warning) => Some(Color::Semantic(SemanticColor::Warning)),
            Theme::Intent(IntentTheme::Danger) => Some(Color::Semantic(SemanticColor::Danger)),

            Theme::Accent | Theme::Ghost => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AccentStrength {
    Weakest,
    Weak,
    Base,
    Strong,
    Strongest,
}

impl AccentStrength {
    pub const fn as_str(self) -> &'static str {
        match self {
            AccentStrength::Weakest => "weakest",
            AccentStrength::Weak => "weak",
            AccentStrength::Base => "base",
            AccentStrength::Strong => "strong",
            AccentStrength::Strongest => "strongest",
        }
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CardShadow {
    None,
    Weakest,
    Weak,
    Base,
    Strong,
    Strongest,
}

impl CardShadow {
    pub const fn as_str(self) -> &'static str {
        match self {
            CardShadow::None => "none",
            CardShadow::Weakest => "weakest",
            CardShadow::Weak => "weak",
            CardShadow::Base => "base",
            CardShadow::Strong => "strong",
            CardShadow::Strongest => "strongest",
        }
    }
}
