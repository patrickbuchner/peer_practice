use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Level {
    Beginner1,
    Beginner2,
    Beginner3,
    Club,
}

impl Level {
    pub const fn all() -> &'static [Level] {
        &[
            Level::Beginner1,
            Level::Beginner2,
            Level::Beginner3,
            Level::Club,
        ]
    }
    pub const ALL: &'static [Level] = Self::all();

    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Beginner1 => "Level 1",
            Level::Beginner2 => "Level 2",
            Level::Beginner3 => "Level 3",
            Level::Club => "Club",
        }
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for Level {
    fn from(s: &str) -> Self {
        match s {
            "Level 1" => Level::Beginner1,
            "Level 2" => Level::Beginner2,
            "Level 3" => Level::Beginner3,
            "Club" => Level::Club,
            _ => Level::Beginner1,
        }
    }
}
