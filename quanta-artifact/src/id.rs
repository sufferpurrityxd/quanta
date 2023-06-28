use {
    quanta_crypto::{AdvancedHasher, HashValue},
    serde::{Deserialize, Serialize},
    sha2::Digest,
    std::fmt::{Display, Formatter},
};

#[derive(thiserror::Error, Debug)]
pub enum ArtifactIdError {
    #[error("Hash Value Error: {0}")]
    /// Error whill occur when trying to
    /// get [`ArtifactId`] from hex-based
    /// string or from [`Vec<u8>`]
    HashValue(#[from] quanta_crypto::HashValueError),
    #[error("Base58 Decode error")]
    /// Error whill occur when trying
    /// to get [`ArtifactId`] from
    /// bs58-based string
    BS58Decode(#[from] bs58::decode::Error),
}

/// Unique identifier in network.
/// Used to validate accepeted data
/// from network or for search.
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId {
    /// [`ArtifactId`] its just a
    /// sha2-256 hash of input bytes
    hv: HashValue,
}

impl ArtifactId {
    /// Get input bytes and create
    /// sha2-256 hash
    pub fn new(input: &[u8]) -> Self {
        Self {
            hv: AdvancedHasher::new(input, sha2::Sha256::new()).finalize(),
        }
    }
    /// By default artifact id does not
    /// use hex value to representation
    pub fn to_hex(self) -> String { self.hv.to_string() }
    /// Get [`ArtifactId`] from string hex-based string
    /// that we get in [`ArtifactId::to_hex`]
    pub fn from_hex_string(input: &str) -> Result<Self, ArtifactIdError> {
        Ok(Self {
            hv: HashValue::try_from(input)?,
        })
    }
    /// Get [`ArtifactId`] from bs58-based string
    /// that we get in [`ArtifactId::to_string`]
    pub fn from_bs58_string(input: &str) -> Result<Self, ArtifactIdError> {
        Ok(Self {
            hv: HashValue::try_from(
                bs58::decode(input)
                    .into_vec()?
                    .as_slice(),
            )?,
        })
    }
    /// Convert artifact id to bytes
    pub fn to_bytes(self) -> Vec<u8> { self.hv.to_bytes() }
    /// Get artifact id from bytes
    pub fn from_bytes(input: &[u8]) -> Result<Self, ArtifactIdError> {
        Ok(Self {
            hv: HashValue::try_from(input)?,
        })
    }
}

impl Display for ArtifactId {
    /// [`ArtifactId`] use bs58-based
    /// string for identify artifacts in network
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", bs58::encode(&self.hv.to_bytes()).into_string())
    }
}
