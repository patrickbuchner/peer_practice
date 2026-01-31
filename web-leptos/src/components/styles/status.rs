#[derive(Clone, Copy)]
pub enum StatusClass {
    Indicator,
    Toast,
}

impl StatusClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            StatusClass::Indicator => "status-indicator",
            StatusClass::Toast => "toast status-toast",
        }
    }
}
