#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Theme {
    Weakest,
    Weak,
    Base,
    Strong,
    Strongest,
    Accent,
    Primary,
    Secondary,
    Success,
    Warning,
    Danger,
    Ghost,
}

impl Theme {
    pub const fn as_str(self) -> &'static str {
        match self {
            Theme::Weakest => "weakest",
            Theme::Weak => "weak",
            Theme::Base => "base",
            Theme::Strong => "strong",
            Theme::Strongest => "strongest",
            Theme::Accent => "accent",
            Theme::Primary => "primary",
            Theme::Secondary => "secondary",
            Theme::Success => "success",
            Theme::Warning => "warning",
            Theme::Danger => "danger",
            Theme::Ghost => "ghost",
        }
    }
}

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
