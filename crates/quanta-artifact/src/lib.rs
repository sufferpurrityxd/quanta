#![allow(dead_code)]
mod artifact;
mod id;
mod magnet;
mod reader;
#[cfg(test)]
mod test;
mod ziplib;

/// All files that have been added to access the
/// network are called artifacts.
///
/// An artifact is a file that is
/// distributed directly from the user's computer.
pub(crate) const MAX_ARTIFACT_SIZE: usize = 2048;

pub use crate::{
    artifact::Artifact,
    id::{ArtifactId, ArtifactIdError},
    magnet::{MagnetError, MagnetLink},
    reader::ArtifactStreamReader,
};
