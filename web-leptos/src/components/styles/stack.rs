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