use quanta_artifact::{Artifact, ArtifactId};

/// Enum that we are accept
/// from proxy and do something with
pub enum FromProxy {
    /// When we receive this event that
    /// means we should start new quanta search
    QuantaSwapSearch {
        /// Id that we are looking for
        artifact_id: ArtifactId,
    },
}

/// This enum we send into proxy
pub enum ToProxy {
    /// When search is complete than we send this
    QuantaSwapSearchCompleted {
        /// Artifact
        artifact: Artifact,
    },
}

/// Generate action (into_proxy, from_proxy)
pub enum ActionGenerate {
    /// Send actions to
    /// core from proxy
    ToCore(FromProxy),
    /// Get actions from core
    FromCore(ToProxy),
}