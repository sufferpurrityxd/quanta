use {
    crate::protobuffable::{Protobuffable, ProtobuffableError},
    quanta_crypto::{AdvancedHasher, HashValue},
    rand::{thread_rng, Rng},
    std::fmt::{Display, Formatter},
};

#[derive(Debug, thiserror::Error)]
pub enum QueryIDError {
    #[error("HashValue Error: {0}")]
    HashValue(#[from] quanta_crypto::HashValueError),
}

/// SearchID is always (hopefully) the unique
/// identifier for query queries on the network.
/// it allows you to understand what request a
/// particular peer is currently transmitting
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct SearchID(HashValue);

impl SearchID {
    /// Generate random [`QueryID`]
    pub fn random() -> Self {
        // Using the blake3 hash
        Self(
            AdvancedHasher::new_with_blake3(
                thread_rng()
                    .gen::<[u8; 32]>()
                    .as_slice(),
            )
            .finalize(),
        )
    }
    /// Try get [`QueryID`] from input str
    pub fn from_str(input: &str) -> Result<Self, QueryIDError> {
        Ok(Self(HashValue::try_from(input)?))
    }
    /// Convert hash to bytes
    pub fn to_bytes(self) -> Vec<u8> { self.0.to_bytes() }
    /// Get hash from input bytes
    pub fn from_bytes(input: Vec<u8>) -> Result<Self, QueryIDError> {
        Ok(Self(HashValue::try_from(input.as_slice())?))
    }
}

impl Display for SearchID {
    /// To convert hash-bytes to a string type, use the hex library
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0.to_string()) }
}

impl Protobuffable for SearchID {
    type ProtoValue = Vec<u8>;
    /// Convert [`Vec<u8>`] which we are receive from network into [`QueryID`]
    fn from_proto(input: Self::ProtoValue) -> Result<Self, ProtobuffableError> {
        Ok(Self::from_bytes(input)?)
    }
    /// Convert [`QueryID`] into [`Vec<u8>`] for sending it over network
    fn to_proto(&self) -> Self::ProtoValue { self.to_bytes() }
}
