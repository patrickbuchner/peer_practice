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
