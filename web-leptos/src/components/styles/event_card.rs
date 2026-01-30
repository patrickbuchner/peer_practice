#[derive(Clone, Copy)]
pub enum EventCardClass {
    Footer,
    FooterCluster,
    Label,
    Count,
    Author,
    Row,
    RowNoWrap,
    Header,
    TitleSelect,
    DateSelect,
    LevelSelect,
    IdeasGrid,
    Textarea,
    Preview,
    Actions,
    ActionsEnd,
    Badge,
    Ideas,
}

impl EventCardClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            EventCardClass::Footer => "cluster cluster--between event-card-footer",
            EventCardClass::FooterCluster => "cluster cluster--start cluster--gap-md",
            EventCardClass::Label => "event-card-label",
            EventCardClass::Count => "event-card-count",
            EventCardClass::Author => "event-card-author",
            EventCardClass::Row => "cluster cluster--start cluster--gap-md event-card-row",
            EventCardClass::RowNoWrap => {
                "cluster cluster--start cluster--gap-md cluster--nowrap event-card-row"
            }
            EventCardClass::Header => {
                "cluster cluster--between cluster--gap-sm cluster--nowrap event-card-header"
            }
            EventCardClass::TitleSelect => "card-title-input event-card-title-select",
            EventCardClass::DateSelect => "event-card-date-select",
            EventCardClass::LevelSelect => "event-card-level-select",
            EventCardClass::IdeasGrid => "event-card-ideas-grid",
            EventCardClass::Textarea => "event-card-textarea",
            EventCardClass::Preview => "event-card-preview",
            EventCardClass::Actions => "event-card-actions",
            EventCardClass::ActionsEnd => "event-card-actions-end",
            EventCardClass::Badge => "event-card-badge",
            EventCardClass::Ideas => "event-card-ideas",
        }
    }
}

#[derive(Clone, Copy)]
pub enum EventListClass {
    None,
    DateGap,
}

impl EventListClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            EventListClass::None => "",
            EventListClass::DateGap => "event-card-date-gap",
        }
    }
}