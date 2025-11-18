pub mod current {
    pub use super::v2025_10_14::*;
}

pub mod v2025_10_14;

#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum Version {
    #[default]
    V2025_10_14,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Envelope<T> {
    pub version: Version,
    pub data: T,
}
