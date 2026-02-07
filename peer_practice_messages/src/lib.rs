pub mod current {
    pub use super::v2026_02_07::*;
}

pub mod v2026_02_07;
pub mod v2026_01_11;
pub mod v2025_10_14;

#[derive(Default, Debug, Clone, Copy, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub enum Version {
    #[default]
    V2026_02_07,
    V2026_01_11,
    V2025_10_14,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EnvelopeHeader {
    pub version: Version,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Envelope<T> {
    pub version: Version,
    pub data: T,
}

pub mod test_helpers_impl;
