#![allow(dead_code)]
use std::path::Path;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sled Error")]
    /// Sled datase error that whill be occur when we are workin with database
    Sled(#[from] sled::Error),
}

/// Custom result
type Result<O> = std::result::Result<O, Error>;

/// [quanta_artifact::Artifact] storage
pub struct QuantaStore {
    /// Database that used is [sled::Db]
    inner: sled::Db,
}

impl QuantaStore {
    /// Create new [QuantaArtifactStore]
    pub fn new<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let inner = sled::open(path)?;
        Ok(Self { inner })
    }
}

/// Implement [quanta_swap::Storage] for using [QuantaArtifactStore] in [quanta_swap::Behaviour]
impl quanta_swap::Storage for QuantaStore {
    fn exists(&self, key: Vec<u8>) -> bool {
        let Ok(exists) = self.inner.contains_key(key) else {
            return false;
        };
        exists
    }

    fn get(&self, key: Vec<u8>) -> Option<Vec<u8>> {
        let Ok(item) = self.inner.get(key) else {
            return None;
        };
        item.map(|ivec| ivec.to_vec())
    }
}
