use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};
use log::error;
use quanta_artifact::{Artifact, MagnetLink};

const MAGNET_TREE_NAME: &str = "magnets";

#[derive(thiserror::Error, Debug)]
pub enum DatabaseError {
    #[error("Got err when trying to open ArtifactDatabase: {0}")]
    /// Error whill occur in [Database::new] call when we are trying to open artifact database from
    /// path that we are get
    ArtifactStorageOpen(sled::Error),
    #[error("Got err when trying to open MagnetTreeDatabase: {0}")]
    /// Error whill occur in [Database::new] call when we are trying to open magnet tree database
    /// from  path that we are get
    MagnetTreeStorageOpen(sled::Error),
    #[error("Got error when trying to insert artifact into storage: {0}")]
    /// Err whill occur when we are call [Database::insert_artifact]
    ArtifactInsert(sled::Error),
    #[error("Got error when trying to insert magnet link into tree: {0}")]
    /// Err whill occur when we are call [Database::insert_magnet_link]
    MagnetInsert(sled::Error),
    #[error("Got error when converting MagnetLink into json")]
    /// Error whill occur when trying to convert magnet link into json-bytes
    MagnetToJson(quanta_artifact::MagnetError),
    #[error("Got error when converting magnetlink from json")]
    /// Error whill occur when trying to convert json-bytes into magnet-link
    MagnetFromJson(quanta_artifact::MagnetError),
    #[error("Got unexpected sled error: {0}")]
    SledUnexpected(#[from] sled::Error),
}
/// Local database that manages magnets and artifacts
pub struct Database {
    /// Artifact DB - is a storage that store artifacts
    artifact_db: sled::Db,
    /// Magnet tree - is a storage that store magnetlinks.
    magnet_tree: sled::Tree,
}

impl Database {
    /// Creates new [Database]. Open [sled::Db] with given path
    pub fn new<P>(path: P) -> Result<Self, DatabaseError>
    where
        P: AsRef<Path>,
    {
        let artifact_db = sled::open(&path).map_err(DatabaseError::ArtifactStorageOpen)?;
        let magnet_tree = artifact_db
            .open_tree(MAGNET_TREE_NAME)
            .map_err(DatabaseError::MagnetTreeStorageOpen)?;

        Ok(Database {
            artifact_db,
            magnet_tree,
        })
    }
    /// Insert [Artifact] into Database... Key in t
    pub fn insert_artifact(&self, artifact: Artifact) -> Result<(), DatabaseError> {
        self.artifact_db
            .insert(artifact.id.to_bytes(), artifact.data)
            .map_err(DatabaseError::ArtifactInsert)?;
        Ok(())
    }
    /// Last index that be inserted into storage.
    fn magnet_tree_last_index(&self) -> Result<u64, DatabaseError> {
        match self.magnet_tree.last()? {
            Some((index, _)) => Ok(u64_from_bytes(index.to_vec())),
            None => Ok(0),
        }
    }
    /// Insert [MagnetLink] into Tree... Key in is just a indexed-integer.
    /// Value its a json-based bytes of magnet link
    pub fn insert_magnet_link(&self, magnet_link: MagnetLink) -> Result<(), DatabaseError> {
        let magnet_tree_last_index = self.magnet_tree_last_index()?;
        self.magnet_tree
            .insert(
                u64_to_bytes(magnet_tree_last_index),
                magnet_link
                    .to_json()
                    .map_err(DatabaseError::MagnetToJson)?,
            )
            .map_err(DatabaseError::MagnetInsert)?;
        Ok(())
    }
    /// Returns all magnet links that stored in [Database] tree
    pub fn get_magnet_links(&self) -> Result<Vec<(u64, MagnetLink)>, DatabaseError> {
        Ok(self
            .magnet_tree
            .iter()
            .map(|result| -> Result<(u64, MagnetLink), DatabaseError> {
                let (index_ivec, magnet_ivec) = result?;
                let index = u64_from_bytes(index_ivec.to_vec());
                match MagnetLink::from_json(magnet_ivec.to_vec()) {
                    Ok(magnet) => Ok((index, magnet)),
                    Err(kind) => {
                        error!("got invalid mangnet link bytes in storage: {}", kind);
                        Err(DatabaseError::MagnetFromJson(kind))
                    },
                }
            })
            .filter(|result| result.is_ok())
            .flatten()
            .collect())
    }
}
/// Implement [quanta_swap::Storage] for [quanta_swap::Behaviour] because all artifacts we are
/// store in [Database]
impl quanta_swap::Storage for Database {
    /// Check if item exists in storage. Here we are dont need to convert
    /// key-bytes into artifact id
    fn exists(&self, key: Vec<u8>) -> bool {
        match self.artifact_db.contains_key(key) {
            Ok(exists) => exists,
            Err(error) => {
                error!(
                    "got an error when checking for artifact in database, : {:?}",
                    error
                );
                // if we got err, then we always should return false
                false
            },
        }
    }
    /// Get item from storage. Here we are dont need to convert key-bytes into artifact id
    fn get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        match self.artifact_db.get(key) {
            Ok(artifact) => artifact.map(|ivec| ivec.to_vec()),
            Err(error) => {
                error!(
                    "got an error when trying to get artifact from database, : {:?}",
                    error
                );
                // if we got err, then we always should return None
                None
            },
        }
    }
}

/// Convert [u64] into bytes this fn used when we are store magnets
fn u64_to_bytes(val: u64) -> Vec<u8> {
    let mut buf = Vec::new();
    LittleEndian::write_u64(&mut buf, val);
    buf
}

/// Convert bytes into u64 this fn used when we are store magnets
fn u64_from_bytes(bytes: Vec<u8>) -> u64 { LittleEndian::read_u64(bytes.as_slice()) }
