#[derive(Clone, Copy)]
pub enum ToastClass {
    Base,
}

impl ToastClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ToastClass::Base => "toast",
        }
    }
}
