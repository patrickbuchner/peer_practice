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
