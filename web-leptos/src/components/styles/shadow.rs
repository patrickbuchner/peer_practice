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