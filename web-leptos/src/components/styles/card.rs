#[derive(Clone, Copy)]
pub enum CardClass {
    Base,
    AuthElevated,
}

impl CardClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            CardClass::Base => "card",
            CardClass::AuthElevated => "card card--elevated auth-card",
        }
    }
}

#[derive(Clone, Copy)]
pub enum CardShellClass {
    Base,
}

impl CardShellClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            CardShellClass::Base => "card-shell",
        }
    }
}