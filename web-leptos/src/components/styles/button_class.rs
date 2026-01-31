#[derive(Clone, Copy)]
pub enum ButtonClass {
    Base,
    Small,
    Fab,
    Icon,
    NavMenu,
    NavMenuLink,
}

impl ButtonClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ButtonClass::Base => "btn",
            ButtonClass::Small => "btn btn--sm",
            ButtonClass::Fab => "btn btn--fab",
            ButtonClass::Icon => "btn-icon",
            ButtonClass::NavMenu => "btn nav-menu-button",
            ButtonClass::NavMenuLink => "btn nav-menu-link",
        }
    }
}
