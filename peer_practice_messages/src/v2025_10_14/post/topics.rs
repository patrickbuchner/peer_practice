use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Copy, PartialOrd, Ord)]
pub enum Topics {
    Basics,
    Swing,
    Spins,
    Connection,
    Timing,
    RockAndGo,
    Anchor,
    FootWork,
    Pattern,
    Blues,
}

impl Topics {
    pub const ALL: &'static [Topics] = Self::all();

    pub const fn all() -> &'static [Topics] {
        &[
            Topics::Basics,
            Topics::Swing,
            Topics::Spins,
            Topics::Connection,
            Topics::Timing,
            Topics::RockAndGo,
            Topics::Anchor,
            Topics::FootWork,
            Topics::Pattern,
            Topics::Blues,
        ]
    }
}

impl From<&str> for Topics {
    fn from(s: &str) -> Self {
        match s {
            "Basics" => Topics::Basics,
            "Swing" => Topics::Swing,
            "Spins" => Topics::Spins,
            "Connection" => Topics::Connection,
            "Timing" => Topics::Timing,
            "RockAndGo" | "Rock & Go" | "Rock-and-Go" => Topics::RockAndGo,
            "Anchor" => Topics::Anchor,
            "FootWork" | "Footwork" | "Foot Work" => Topics::FootWork,
            "Pattern" | "Patterns" => Topics::Pattern,
            "Blues" => Topics::Blues,
            _ => Topics::Basics,
        }
    }
}

impl Display for Topics {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Topics::Basics => "Basics",
            Topics::Swing => "Swing",
            Topics::Spins => "Spins",
            Topics::Connection => "Connection",
            Topics::Timing => "Timing",
            Topics::RockAndGo => "Rock & Go",
            Topics::Anchor => "Anchor",
            Topics::FootWork => "Footwork",
            Topics::Pattern => "Pattern",
            Topics::Blues => "Blues",
        };
        write!(f, "{}", s)
    }
}

impl From<Topics> for String {
    fn from(t: Topics) -> Self {
        t.to_string()
    }
}
