use {
    log::{error, warn},
    quanta_artifact::ArtifactId,
    sled::Db,
    std::path::Path,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("SledDB Error")]
    /// Error whill occur when we
    /// are working with sled database
    SledDB(#[from] sled::Error),
}

/// Repository is main store of
/// artifacts in quanta protocol
pub struct Repository {
    /// We use sled for artifact storage
    storage: Db,
}

impl Repository {
    /// Create new [Repository]
    pub fn new<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let storage = sled::open(path)?;
        Ok(Self { storage })
    }
}

/// Implement quanta-swap storage for repository
impl quanta_swap::Storage for Repository {
    fn exists(&mut self, key: Vec<u8>) -> bool {
        match self.storage.contains_key(key) {
            Ok(contains) => contains,
            Err(error) => {
                warn!(
                    "Error occured when trying to check if exists in repository: {}",
                    error
                );
                false
            },
        }
    }

    fn get(&mut self, key: Vec<u8>) -> Option<Vec<u8>> {
        match self.storage.get(key) {
            Ok(value) => value.map(|ivec| ivec.to_vec()),
            Err(error) => {
                warn!(
                    "Error occured when trying to get value from repository: {}",
                    error
                );
                None
            },
        }
    }
}
