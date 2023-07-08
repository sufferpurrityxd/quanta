use std::{
    collections::HashMap,
    fmt::{Display, Formatter},
};

use async_std::path::Path;
use futures::{AsyncReadExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};

use crate::{
    id::ArtifactId,
    ziplib::{decode_gzip_all, encode_gzip_all},
};

#[derive(thiserror::Error, Debug)]
pub enum MagnetError {
    #[error("To Bincode Error")]
    /// This error whill occur when trying to convert [`MagnetLink`] into bincode bytes
    ToBincode,
    #[error("From bincode error")]
    /// Error whill occur when trying to convert bincode-bytes into [`MagnetLink`]
    FromBincode,
    #[error("Base58 Decode Error: {0}")]
    /// Error whill occur when trying to convert string magnet link into bytes
    Base58Decode(#[from] bs58::decode::Error),
    #[error("Zip lib error")]
    /// Error whill occur when working with compression
    ZipLibError(#[from] crate::ziplib::ZipLibError),
    #[error("IO Error")]
    /// IO error
    IOError(#[from] std::io::Error),
}

/// A magnet link is a link to a file on the quanta network. It stores information about the id of
/// the artifacts that need to be obtained in order to collect the file. File size, file name,
/// extension. All this is encoded in bincode, and then in base58 string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagnetLink {
    /// Artifact id mapping stores the order number of the id of the artifact. That is, the index
    /// determines in what order to collect the final file
    artifact_id_mapping: HashMap<usize, ArtifactId>,
    /// Name of file
    file_name: String,
    /// File size
    size: usize,
}

impl MagnetLink {
    /// Creates new [`MagnetLink`]
    pub fn new(file_name: String, size: usize) -> Self {
        let artifact_id_mapping = HashMap::default();

        Self {
            artifact_id_mapping,
            file_name,
            size,
        }
    }
    /// updates the current state of [`artifact_id_mapping`]
    pub fn new_update_with_artifact_id(&mut self, artifact_id: ArtifactId) {
        self.artifact_id_mapping
            .entry(self.next_idx())
            .or_insert(artifact_id);
    }
    /// returns next index that be used in [`MagnetLink::new_update_with_artifact_id`]
    fn next_idx(&self) -> usize {
        match self.artifact_id_mapping.keys().max() {
            Some(idx) => idx + 1,
            None => 1,
        }
    }
    /// returns bincode-based bytes
    pub fn to_bincode(&self) -> Result<Vec<u8>, MagnetError> {
        bincode::serialize(self).map_err(|_| MagnetError::ToBincode)
    }
    /// get bincode bytes and compress them with gzip
    pub fn to_bincode_compressed(&self) -> Result<Vec<u8>, MagnetError> {
        Ok(encode_gzip_all(self.to_bincode()?)?)
    }
    /// get self from bincode-compressed bytes
    pub fn from_bincode_compressed(input: Vec<u8>) -> Result<Self, MagnetError> {
        Self::from_bincode_compressed(decode_gzip_all(input.as_slice())?)
    }
    /// returns [`Self`] from bincode-based bytes
    pub fn from_bincode(bincod: Vec<u8>) -> Result<Self, MagnetError> {
        bincode::deserialize(bincod.as_slice()).map_err(|_| MagnetError::FromBincode)
    }
    /// save magnet link in file
    pub async fn save_into_file<P>(&self, path: P) -> Result<(), MagnetError>
    where
        P: AsRef<Path>,
    {
        let bytes = self.to_bincode_compressed()?;
        let mut file = async_std::fs::File::create(path).await?;
        file.write_all(bytes.as_slice()).await?;
        Ok(())
    }
    /// load magnet link from that we are get in [MagnetLink::save_into_file]
    pub async fn read_from_file<P>(&self, path: P) -> Result<Self, MagnetError>
    where
        P: AsRef<Path>,
    {
        let mut file = async_std::fs::File::open(path).await?;
        let mut buf = Vec::new();
        file.read_to_end(&mut buf).await?;
        Self::from_bincode_compressed(buf)
    }
}

impl Display for MagnetLink {
    /// Get string-based type of magnet link for sharing over network
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let bincoded = self
            .to_bincode()
            .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", bs58::encode(bincoded.as_slice()).into_string())
    }
}

impl TryFrom<String> for MagnetLink {
    type Error = MagnetError;
    /// Get [`MagnetLink`] from string-based link that we are receive in [`Display`]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_bincode(bs58::decode(value).into_vec()?)
    }
}
