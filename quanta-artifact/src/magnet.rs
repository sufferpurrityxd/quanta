use {
    crate::id::ArtifactId,
    serde::{Deserialize, Serialize},
    std::{
        collections::HashMap,
        fmt::{Display, Formatter},
    },
};

#[derive(thiserror::Error, Debug)]
pub enum MagnetError {
    #[error("To Json Error")]
    /// This error whill occur
    /// when trying to convert [`MagnetLink`]
    /// into json bytes
    ToJson,
    #[error("From json error")]
    /// Error whill occur when trying
    /// to convert json-bytes into [`MagnetLink`]
    FromJson,
    #[error("Base58 Decode Error: {0}")]
    /// Error whill occur when trying to
    /// convert string magnet link into bytes
    Base58Decode(#[from] bs58::decode::Error),
}

/// A magnet link is a link to a file on the quanta network.
/// It stores information about the id of the artifacts that
/// need to be obtained in order to collect the file. File size,
/// file name, extension. All this is encoded in json, and then
/// in base58 string
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MagnetLink {
    /// Artifact id mapping stores the order number of
    /// the id of the artifact. That is, the index determines
    /// in what order to collect the final file
    artifact_id_mapping: HashMap<usize, ArtifactId>,
    /// Name of file
    file_name: String,
    /// File extension
    extension: String,
    /// File size
    size: usize,
}

impl MagnetLink {
    /// Creates new [`MagnetLink`]
    pub fn new(file_name: String, extension: String, size: usize) -> Self {
        let artifact_id_mapping = HashMap::default();

        Self {
            artifact_id_mapping,
            file_name,
            extension,
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
    /// Conver [`MagnetLink`] to str based json
    pub fn to_json_str(&self) -> Result<String, MagnetError> {
        serde_json::to_string(self).map_err(|_| MagnetError::ToJson)
    }
    /// returns json-based bytes
    pub fn to_json(&self) -> Result<Vec<u8>, MagnetError> {
        serde_json::to_vec(self).map_err(|_| MagnetError::ToJson)
    }
    /// returns [`Self`] from json-based bytes
    pub fn from_json(json: Vec<u8>) -> Result<Self, MagnetError> {
        serde_json::from_slice(json.as_slice()).map_err(|_| MagnetError::FromJson)
    }
}

impl Display for MagnetLink {
    /// Get string-based type of magnet
    /// link for sharing over network
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let json = self
            .to_json()
            .map_err(|_| std::fmt::Error)?;
        write!(f, "{}", bs58::encode(json).into_string())
    }
}

impl TryFrom<String> for MagnetLink {
    type Error = MagnetError;
    /// Get [`MagnetLink`] from string-based
    /// link that we are receive in [`Display`]
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::from_json(bs58::decode(value).into_vec()?)
    }
}