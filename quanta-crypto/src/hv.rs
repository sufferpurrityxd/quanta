use {
    crate::Hash,
    hex::FromHex,
    std::fmt::{Display, Formatter, LowerHex},
};

#[derive(Debug, thiserror::Error)]
pub enum HashValueError {
    #[error("From Hex-String Error: {0}")]
    /// Error whill occur when trying to
    /// get [`HashValue`]  from hex based string
    StringHex(#[from] hex::FromHexError),
    #[error("From SLice Error: {0}")]
    /// Error whill occur when trying to
    /// get [`HashValue`] from bytes of hash
    Bytes(#[from] std::array::TryFromSliceError),
}

/// [`HashValue`] its just a hash
/// but with some implementations
#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct HashValue(Hash);

impl HashValue {
    /// Get [`HashValue`] from raw [`Hash`]
    pub fn new(hash: Hash) -> Self { Self(hash) }
    /// Convert [`HashValue`] to bytes
    pub fn to_bytes(self) -> Vec<u8> { self.0.to_vec() }
}
/// Implement for to_string
impl LowerHex for HashValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            write!(f, "0x")?;
        };

        for byte in &self.0 {
            write!(f, "{:02x}", byte)?
        }
        Ok(())
    }
}
/// [`HashValue::to_string`]
impl Display for HashValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result { write!(f, "{:x}", self) }
}
/// Get [`HashValue`] from hex-string
impl TryFrom<&str> for HashValue {
    type Error = HashValueError;
    fn try_from(value: &str) -> Result<Self, Self::Error> { Ok(Self(Hash::from_hex(value)?)) }
}
/// Get [`HashValue`]
/// from raw hash bytes
impl TryFrom<&[u8]> for HashValue {
    type Error = HashValueError;
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> { Ok(Self(Hash::try_from(value)?)) }
}
