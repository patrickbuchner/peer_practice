use serde::{Deserialize, Serialize};
#[derive(Hash, Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub struct Email {
    #[cfg(target_arch = "wasm32")]
    value: String,
    #[cfg(not(target_arch = "wasm32"))]
    value: lettre::message::Mailbox,
}

#[cfg(not(target_arch = "wasm32"))]
impl Email {
    pub fn new(value: &str) -> Option<Email> {
        use lettre::message::Mailbox;
        match value.parse::<Mailbox>() {
            Ok(mailbox) => Some(mailbox.into()),
            Err(_) => None,
        }
    }

    pub fn value(&self) -> String {
        self.value.email.to_string()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod all {
    use crate::v2025_10_14::email::Email;
    use lettre::message::Mailbox;

    impl From<Email> for Mailbox {
        fn from(value: Email) -> Self {
            value.value().parse::<Mailbox>().unwrap()
        }
    }

    impl From<Mailbox> for Email {
        fn from(value: Mailbox) -> Self {
            Email { value }
        }
    }
}
#[cfg(target_arch = "wasm32")]
impl Email {
    pub fn new(value: &str) -> Option<Email> {
        use regex::Regex;
        let r = Regex::new(
            r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$",
        ).unwrap();
        if r.is_match(value) {
            Some(Email {
                value: value.into(),
            })
        } else {
            None
        }
    }

    pub fn value(&self) -> String {
        self.value.to_string()
    }
}
