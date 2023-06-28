use crate::id::ArtifactId;

/// Main object in Quanta Protocol
///
/// Artifact contains just
/// two fields - data-bytes
/// and unique sha2-256 hash of
/// this bytes that called are [`ArtifactId`]
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct Artifact {
    /// Unqiue id of [`Artifact::data`]
    pub id: ArtifactId,
    /// Bytes data
    pub data: Vec<u8>,
}

impl Artifact {
    /// Get new [`Artifact`] from input bytes
    pub fn new(data: Vec<u8>) -> Self {
        let id = ArtifactId::new(data.as_slice());
        Self { id, data }
    }
}
