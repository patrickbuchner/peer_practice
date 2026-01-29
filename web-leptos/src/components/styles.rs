use crate::components::theme::Theme;
use peer_practice_shared::accent_colors::AccentColor;

#[derive(Clone, Copy)]
pub enum ButtonClass {
    Base,
    Small,
    Fab,
    Icon,
    NavMenu,
    NavMenuLink,
}

impl ButtonClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ButtonClass::Base => "btn",
            ButtonClass::Small => "btn btn--sm",
            ButtonClass::Fab => "btn btn--fab",
            ButtonClass::Icon => "btn-icon",
            ButtonClass::NavMenu => "btn nav-menu-button",
            ButtonClass::NavMenuLink => "btn nav-menu-link",
        }
    }
}

#[derive(Clone, Copy)]
pub enum NavbarClass {
    Navbar,
    Section,
    SectionCenter,
    SectionEnd,
    Title,
    TitleAccent,
    IconBar,
}

impl NavbarClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            NavbarClass::Navbar => "navbar",
            NavbarClass::Section => "navbar-section",
            NavbarClass::SectionCenter => "navbar-section navbar-section--center",
            NavbarClass::SectionEnd => "navbar-section navbar-section--end",
            NavbarClass::Title => "navbar-title",
            NavbarClass::TitleAccent => "navbar-title navbar-title--accent",
            NavbarClass::IconBar => "nav-icon-bar",
        }
    }
}

#[derive(Clone, Copy)]
pub enum LayoutClass {
    PagePadStack,
    PagePad,
    PageCenter,
    ContainerNarrowPadSm,
    StackSm,
    RowEnd,
    RowBetween,
}

impl LayoutClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            LayoutClass::PagePadStack => "page page-pad page-stack",
            LayoutClass::PagePad => "page page-pad",
            LayoutClass::PageCenter => "page-center",
            LayoutClass::ContainerNarrowPadSm => "container container-narrow pad-sm",
            LayoutClass::StackSm => "stack stack-sm",
            LayoutClass::RowEnd => "row row-end",
            LayoutClass::RowBetween => "row row-between",
        }
    }
}

#[derive(Clone, Copy)]
pub enum ClusterClass {
    BetweenGapSmAlignCenter,
    BetweenGapSm,
    Between,
    GapSmAlignCenter,
}

impl ClusterClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ClusterClass::BetweenGapSmAlignCenter => {
                "cluster cluster--between cluster--gap-sm cluster--align-center"
            }
            ClusterClass::BetweenGapSm => "cluster cluster--between cluster--gap-sm",
            ClusterClass::Between => "cluster cluster--between",
            ClusterClass::GapSmAlignCenter => "cluster cluster--gap-sm cluster--align-center",
        }
    }
}

#[derive(Clone, Copy)]
pub enum TextClass {
    CardTitle,
    CardTitleTight,
    CardNoteMutedSm,
    Dim,
    DimSm,
    Muted,
    SmMuted,
    SmMutedMono,
    Lg,
}

impl TextClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            TextClass::CardTitle => "card-title",
            TextClass::CardTitleTight => "card-title card-title--tight",
            TextClass::CardNoteMutedSm => "text-muted text-sm card-note",
            TextClass::Dim => "text-dim",
            TextClass::DimSm => "text-dim text-sm",
            TextClass::Muted => "text-muted",
            TextClass::SmMuted => "text-sm text-muted",
            TextClass::SmMutedMono => "text-sm text-mono text-muted",
            TextClass::Lg => "text-lg",
        }
    }
}

#[derive(Clone, Copy)]
pub enum FormClass {
    Form,
    Grid,
    ActionsFull,
    LabelEnd,
    LabelSpaced,
    InputWide,
    InputCenter,
    PaletteGrid,
    SectionDivider,
}

impl FormClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            FormClass::Form => "form",
            FormClass::Grid => "form-grid",
            FormClass::ActionsFull => "form-actions form-actions--full",
            FormClass::LabelEnd => "label label--end",
            FormClass::LabelSpaced => "label label--spaced",
            FormClass::InputWide => "input--wide",
            FormClass::InputCenter => "input--center",
            FormClass::PaletteGrid => "palette-grid",
            FormClass::SectionDivider => "section-divider",
        }
    }
}

#[derive(Clone, Copy)]
pub enum NavMenuClass {
    Icon,
    IconBar,
    Label,
    Panel,
}

impl NavMenuClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            NavMenuClass::Icon => "nav-menu-icon",
            NavMenuClass::IconBar => "nav-menu-icon-bar",
            NavMenuClass::Label => "nav-menu-label",
            NavMenuClass::Panel => "nav-menu-panel",
        }
    }
}

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

#[derive(Clone, Copy)]
pub enum CardClass {
    Base,
    AuthElevated,
}

impl CardClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            CardClass::Base => "card",
            CardClass::AuthElevated => "card card--elevated auth-card",
        }
    }
}

#[derive(Clone, Copy)]
pub enum CardShellClass {
    Base,
}

impl CardShellClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            CardShellClass::Base => "card-shell",
        }
    }
}

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

#[derive(Clone, Copy)]
pub enum AccentName {
    Rosewater,
}

impl AccentName {
    pub const fn as_str(self) -> &'static str {
        match self {
            AccentName::Rosewater => "rosewater",
        }
    }
}

#[derive(Clone, Copy)]
pub enum NavLinkState {
    Active,
    Inactive,
}

impl NavLinkState {
    pub fn accent_vars(self, accent: &str) -> String {
        match self {
            NavLinkState::Active => {
                format!("--accent: var(--{0}); --accent-light: var(--{0});", accent)
            }
            NavLinkState::Inactive => format!(
                "--accent: var(--{0}-light); --accent-light: var(--{0}-light);",
                accent
            ),
        }
    }
}

pub fn nav_link_style(active: bool, accent: &str) -> String {
    if active {
        NavLinkState::Active.accent_vars(accent)
    } else {
        NavLinkState::Inactive.accent_vars(accent)
    }
}

#[derive(Clone, Copy)]
pub enum PaletteStyle {
    AccentSwatch,
}

impl PaletteStyle {
    pub fn with_accent(self, accent_var: &str) -> String {
        match self {
            PaletteStyle::AccentSwatch => {
                format!("--accent: {}; width: 100%;", accent_var)
            }
        }
    }
}

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

#[derive(Clone, Copy)]
pub enum StackStyle {
    Card,
}

impl StackStyle {
    pub fn with_z_index(self, z_index: usize) -> String {
        match self {
            StackStyle::Card => format!("position: relative; z-index: {z_index};"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum CssVar {
    BgStrong,
    BgStrongest,
    Teal,
}

impl CssVar {
    pub const fn as_str(self) -> &'static str {
        match self {
            CssVar::BgStrongest => "var(--bg-strongest-color)",
            CssVar::Teal => "var(--teal)",
            CssVar::BgStrong => "var(--bg-strong-color)",
        }
    }
}

#[derive(Clone, Copy)]
pub enum StatusColor {
    Connected,
    Disconnected,
}

impl StatusColor {
    pub const fn as_str(self) -> &'static str {
        match self {
            StatusColor::Connected => "var(--success-color)",
            StatusColor::Disconnected => "var(--danger-color)",
        }
    }

    pub const fn from_connected(connected: bool) -> StatusColor {
        if connected {
            StatusColor::Connected
        } else {
            StatusColor::Disconnected
        }
    }
}

#[derive(Clone, Copy)]
pub enum SvgStrokeColor {
    StatusOutline,
}

impl SvgStrokeColor {
    pub const fn as_str(self) -> &'static str {
        match self {
            SvgStrokeColor::StatusOutline => "#111827",
        }
    }
}

pub fn pin_theme(is_complete: bool) -> &'static str {
    if is_complete {
        Theme::Primary.as_str()
    } else {
        Theme::Secondary.as_str()
    }
}

pub fn chat_accent_style(color: AccentColor) -> String {
    format!("--accent: {};", color.css_var())
}

pub fn chat_name_style(color: AccentColor) -> String {
    format!("color: {};", color.css_var())
}

pub fn chat_border_style(color: AccentColor) -> String {
    format!("border-color: {};", color.css_var())
}
