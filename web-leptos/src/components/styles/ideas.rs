#[derive(Clone, Copy)]
pub enum IdeasColumns {
    Single,
    Split,
}

impl IdeasColumns {
    pub fn to_style(self) -> String {
        match self {
            IdeasColumns::Single => "--ideas-columns: 1fr;".to_string(),
            IdeasColumns::Split => "--ideas-columns: 1fr 1fr;".to_string(),
        }
    }
}
