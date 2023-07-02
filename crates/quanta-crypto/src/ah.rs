use {
    crate::{hv::HashValue, Hash},
    digest::{generic_array::GenericArray, Digest, OutputSizeUser},
};

/// [`AdvancedHasher`] - this is about nested hashing
#[derive(Clone, Copy)]
pub struct AdvancedHasher(Hash);

impl AdvancedHasher {
    /// Create new [`AdvancedHasher`]
    ///
    /// Input values:
    ///     [`input`]: Data that going to be hashed
    ///     [`hasher`]: Hasher that hash [`input`]
    pub fn new<H>(input: &[u8], mut hasher: H) -> Self
    where
        H: Digest,
        [u8; 32]: From<GenericArray<u8, <H as OutputSizeUser>::OutputSize>>,
    {
        hasher.update(input);

        Self(hasher.finalize().into())
    }
    /// For some reason blake3 not support [`Digest`] so we just use this func if we want blake
    pub fn new_with_blake3(input: &[u8]) -> Self {
        let mut hasher = Self::get_blake_hasher();
        hasher.update(input);
        Self(hasher.finalize().into())
    }
    /// Update current digest with new hasher
    pub fn update_digest<H>(&mut self, mut hasher: H) -> &mut AdvancedHasher
    where
        H: Digest,
        [u8; 32]: From<GenericArray<u8, <H as OutputSizeUser>::OutputSize>>,
    {
        hasher.update(self.0);
        self.0 = hasher.finalize().into();
        self
    }
    /// For some reason blake3 not support [`Digest`] so we just use this func if we want blake
    pub fn update_with_blake3(&mut self) -> &mut AdvancedHasher {
        let mut hasher = Self::get_blake_hasher();
        hasher.update(self.0.as_slice());
        self.0 = hasher.finalize().into();
        self
    }
    /// get [`blake3::Hasher`]
    fn get_blake_hasher() -> blake3::Hasher { blake3::Hasher::new() }
    /// Finalize hash cycle and return [`HashValue`]
    pub fn finalize(self) -> HashValue { HashValue::new(self.0) }
}
