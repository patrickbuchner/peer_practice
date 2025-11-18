use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AccentColor {
    // Base colors
    Rosewater,
    Flamingo,
    Pink,
    Mauve,
    Red,
    Maroon,
    Peach,
    Yellow,
    Green,
    Teal,
    Sky,
    Sapphire,
    Blue,
    Lavender,

    // Light variants
    RosewaterLight,
    FlamingoLight,
    PinkLight,
    MauveLight,
    RedLight,
    MaroonLight,
    PeachLight,
    YellowLight,
    GreenLight,
    TealLight,
    SkyLight,
    SapphireLight,
    BlueLight,
    LavenderLight,
}

impl AccentColor {
    /// Returns the CSS var() reference associated with this color (e.g., "var(--teal)").
    pub const fn css_var(self) -> &'static str {
        match self {
            // Base
            AccentColor::Rosewater => "var(--rosewater)",
            AccentColor::Flamingo => "var(--flamingo)",
            AccentColor::Pink => "var(--pink)",
            AccentColor::Mauve => "var(--mauve)",
            AccentColor::Red => "var(--red)",
            AccentColor::Maroon => "var(--maroon)",
            AccentColor::Peach => "var(--peach)",
            AccentColor::Yellow => "var(--yellow)",
            AccentColor::Green => "var(--green)",
            AccentColor::Teal => "var(--teal)",
            AccentColor::Sky => "var(--sky)",
            AccentColor::Sapphire => "var(--sapphire)",
            AccentColor::Blue => "var(--blue)",
            AccentColor::Lavender => "var(--lavender)",
            // Light
            AccentColor::RosewaterLight => "var(--rosewater-light)",
            AccentColor::FlamingoLight => "var(--flamingo-light)",
            AccentColor::PinkLight => "var(--pink-light)",
            AccentColor::MauveLight => "var(--mauve-light)",
            AccentColor::RedLight => "var(--red-light)",
            AccentColor::MaroonLight => "var(--maroon-light)",
            AccentColor::PeachLight => "var(--peach-light)",
            AccentColor::YellowLight => "var(--yellow-light)",
            AccentColor::GreenLight => "var(--green-light)",
            AccentColor::TealLight => "var(--teal-light)",
            AccentColor::SkyLight => "var(--sky-light)",
            AccentColor::SapphireLight => "var(--sapphire-light)",
            AccentColor::BlueLight => "var(--blue-light)",
            AccentColor::LavenderLight => "var(--lavender-light)",
        }
    }

    /// Whether this is a light variant.
    pub const fn is_light(self) -> bool {
        matches!(
            self,
            AccentColor::RosewaterLight
                | AccentColor::FlamingoLight
                | AccentColor::PinkLight
                | AccentColor::MauveLight
                | AccentColor::RedLight
                | AccentColor::MaroonLight
                | AccentColor::PeachLight
                | AccentColor::YellowLight
                | AccentColor::GreenLight
                | AccentColor::TealLight
                | AccentColor::SkyLight
                | AccentColor::SapphireLight
                | AccentColor::BlueLight
                | AccentColor::LavenderLight
        )
    }

    /// Return all base colors.
    pub const fn base() -> &'static [AccentColor] {
        &BASE
    }

    /// Return all light colors.
    pub const fn light() -> &'static [AccentColor] {
        &LIGHT
    }

    /// Return all colors (base + light).
    pub const fn all() -> &'static [AccentColor] {
        &ALL
    }
}

const BASE: [AccentColor; 14] = [
    AccentColor::Rosewater,
    AccentColor::Flamingo,
    AccentColor::Pink,
    AccentColor::Mauve,
    AccentColor::Red,
    AccentColor::Maroon,
    AccentColor::Peach,
    AccentColor::Yellow,
    AccentColor::Green,
    AccentColor::Teal,
    AccentColor::Sky,
    AccentColor::Sapphire,
    AccentColor::Blue,
    AccentColor::Lavender,
];

const LIGHT: [AccentColor; 14] = [
    AccentColor::RosewaterLight,
    AccentColor::FlamingoLight,
    AccentColor::PinkLight,
    AccentColor::MauveLight,
    AccentColor::RedLight,
    AccentColor::MaroonLight,
    AccentColor::PeachLight,
    AccentColor::YellowLight,
    AccentColor::GreenLight,
    AccentColor::TealLight,
    AccentColor::SkyLight,
    AccentColor::SapphireLight,
    AccentColor::BlueLight,
    AccentColor::LavenderLight,
];

const ALL: [AccentColor; 28] = {
    let mut a = [AccentColor::Rosewater; 28];
    // Base (0..14)
    a[0] = AccentColor::Rosewater;
    a[1] = AccentColor::Flamingo;
    a[2] = AccentColor::Pink;
    a[3] = AccentColor::Mauve;
    a[4] = AccentColor::Red;
    a[5] = AccentColor::Maroon;
    a[6] = AccentColor::Peach;
    a[7] = AccentColor::Yellow;
    a[8] = AccentColor::Green;
    a[9] = AccentColor::Teal;
    a[10] = AccentColor::Sky;
    a[11] = AccentColor::Sapphire;
    a[12] = AccentColor::Blue;
    a[13] = AccentColor::Lavender;

    // Light (14..28)
    a[14] = AccentColor::RosewaterLight;
    a[15] = AccentColor::FlamingoLight;
    a[16] = AccentColor::PinkLight;
    a[17] = AccentColor::MauveLight;
    a[18] = AccentColor::RedLight;
    a[19] = AccentColor::MaroonLight;
    a[20] = AccentColor::PeachLight;
    a[21] = AccentColor::YellowLight;
    a[22] = AccentColor::GreenLight;
    a[23] = AccentColor::TealLight;
    a[24] = AccentColor::SkyLight;
    a[25] = AccentColor::SapphireLight;
    a[26] = AccentColor::BlueLight;
    a[27] = AccentColor::LavenderLight;

    a
};

impl Display for AccentColor {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            // Base
            AccentColor::Rosewater => "rosewater",
            AccentColor::Flamingo => "flamingo",
            AccentColor::Pink => "pink",
            AccentColor::Mauve => "mauve",
            AccentColor::Red => "red",
            AccentColor::Maroon => "maroon",
            AccentColor::Peach => "peach",
            AccentColor::Yellow => "yellow",
            AccentColor::Green => "green",
            AccentColor::Teal => "teal",
            AccentColor::Sky => "sky",
            AccentColor::Sapphire => "sapphire",
            AccentColor::Blue => "blue",
            AccentColor::Lavender => "lavender",
            // Light
            AccentColor::RosewaterLight => "rosewater-light",
            AccentColor::FlamingoLight => "flamingo-light",
            AccentColor::PinkLight => "pink-light",
            AccentColor::MauveLight => "mauve-light",
            AccentColor::RedLight => "red-light",
            AccentColor::MaroonLight => "maroon-light",
            AccentColor::PeachLight => "peach-light",
            AccentColor::YellowLight => "yellow-light",
            AccentColor::GreenLight => "green-light",
            AccentColor::TealLight => "teal-light",
            AccentColor::SkyLight => "sky-light",
            AccentColor::SapphireLight => "sapphire-light",
            AccentColor::BlueLight => "blue-light",
            AccentColor::LavenderLight => "lavender-light",
        };
        f.write_str(s)
    }
}

impl FromStr for AccentColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = match s {
            // Base
            "rosewater" => AccentColor::Rosewater,
            "flamingo" => AccentColor::Flamingo,
            "pink" => AccentColor::Pink,
            "mauve" => AccentColor::Mauve,
            "red" => AccentColor::Red,
            "maroon" => AccentColor::Maroon,
            "peach" => AccentColor::Peach,
            "yellow" => AccentColor::Yellow,
            "green" => AccentColor::Green,
            "teal" => AccentColor::Teal,
            "sky" => AccentColor::Sky,
            "sapphire" => AccentColor::Sapphire,
            "blue" => AccentColor::Blue,
            "lavender" => AccentColor::Lavender,
            // Light
            "rosewater-light" => AccentColor::RosewaterLight,
            "flamingo-light" => AccentColor::FlamingoLight,
            "pink-light" => AccentColor::PinkLight,
            "mauve-light" => AccentColor::MauveLight,
            "red-light" => AccentColor::RedLight,
            "maroon-light" => AccentColor::MaroonLight,
            "peach-light" => AccentColor::PeachLight,
            "yellow-light" => AccentColor::YellowLight,
            "green-light" => AccentColor::GreenLight,
            "teal-light" => AccentColor::TealLight,
            "sky-light" => AccentColor::SkyLight,
            "sapphire-light" => AccentColor::SapphireLight,
            "blue-light" => AccentColor::BlueLight,
            "lavender-light" => AccentColor::LavenderLight,
            _ => return Err(()),
        };
        Ok(v)
    }
}
