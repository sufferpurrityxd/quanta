use {sled::Db, std::path::Path};

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
