#[derive(Clone, Copy)]
pub enum NavMenuClass {
    Icon,
    IconBar,
    Label,
    Panel,
}

impl NavMenuClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            NavMenuClass::Icon => "nav-menu-icon",
            NavMenuClass::IconBar => "nav-menu-icon-bar",
            NavMenuClass::Label => "nav-menu-label",
            NavMenuClass::Panel => "nav-menu-panel",
        }
    }
}