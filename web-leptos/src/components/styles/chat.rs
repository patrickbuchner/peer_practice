#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum ChatClass {
    Row,
    Messages,
    Message,
    MessageMine,
    MessageSystem,
    Meta,
    BubbleSurface,
    BubbleSystem,
    InputBar,
    InputField,
}

impl ChatClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ChatClass::Row => "chat-row",
            ChatClass::Messages => "chat-messages",
            ChatClass::Message => "chat-message",
            ChatClass::MessageMine => "chat-message chat-message--mine",
            ChatClass::MessageSystem => "chat-message chat-message--system",
            ChatClass::Meta => "chat-meta",
            ChatClass::BubbleSurface => "surface chat-bubble",
            ChatClass::BubbleSystem => "chat-bubble chat-bubble--system",
            ChatClass::InputBar => "chat-input-bar",
            ChatClass::InputField => "chat-input-field",
        }
    }
}
