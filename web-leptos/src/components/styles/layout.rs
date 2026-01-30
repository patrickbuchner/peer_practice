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