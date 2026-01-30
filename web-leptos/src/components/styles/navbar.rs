#[derive(Clone, Copy)]
pub enum NavbarClass {
    Navbar,
    Section,
    SectionCenter,
    SectionEnd,
    Title,
    TitleAccent,
    IconBar,
}

impl NavbarClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            NavbarClass::Navbar => "navbar",
            NavbarClass::Section => "navbar-section",
            NavbarClass::SectionCenter => "navbar-section navbar-section--center",
            NavbarClass::SectionEnd => "navbar-section navbar-section--end",
            NavbarClass::Title => "navbar-title",
            NavbarClass::TitleAccent => "navbar-title navbar-title--accent",
            NavbarClass::IconBar => "nav-icon-bar",
        }
    }
}

#[derive(Clone, Copy)]
pub enum NavLinkState {
    Active,
    Inactive,
}

impl NavLinkState {
    pub fn accent_vars(self, accent: &str) -> String {
        match self {
            NavLinkState::Active => {
                format!("--accent: var(--{0}); --accent-light: var(--{0});", accent)
            }
            NavLinkState::Inactive => format!(
                "--accent: var(--{0}-light); --accent-light: var(--{0}-light);",
                accent
            ),
        }
    }
}

pub fn nav_link_style(active: bool, accent: &str) -> String {
    if active {
        NavLinkState::Active.accent_vars(accent)
    } else {
        NavLinkState::Inactive.accent_vars(accent)
    }
}