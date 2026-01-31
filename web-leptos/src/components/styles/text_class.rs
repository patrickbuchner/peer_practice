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
