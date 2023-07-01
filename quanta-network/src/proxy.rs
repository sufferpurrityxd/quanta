use libp2p::PeerId;
use quanta_artifact::{Artifact, ArtifactId};
use quanta_swap::SearchID;
use crate::info::ConnectionInformation;

/// Events that [crate::service::QuantaService] receive from proxy
#[derive(Debug)]
pub enum ToServiceEvent {
    /// Proxy sends this event when he wants all connections with node
    ///
    /// Here we got [tokio::sync::oneshot::Sender] that we want send to
    GetConnections(tokio::sync::oneshot::Sender<(PeerId, ConnectionInformation)>),
    /// Proxy sends this event when he wants create new search that implemented by
    /// [quanta_swap::Behaviour]
    CreateQuantaSearch {
        /// [ArtifactId] of [Artifact] in [quanta_store::QuantaStore]
        searching: ArtifactId,
        /// [tokio::sync::oneshot::Sender] that send [SearchID] which received from
        /// [quanta_swap::Behaviour::search_item_with]
        search_id_ch: tokio::sync::oneshot::Sender<SearchID>,
    }
}

/// Events that proxy receive from [crate::service::QuantaService]
#[derive(Debug)]
pub enum ToProxyEvent {
    /// Service sends that event when [quanta_swap::Behaviour] search item with specific
    /// searching key. This event always run afetr [ToServiceEvent::CreateQuantaSearch]
    QuantaSearchReady {
        /// [SearchID] which received from [quanta_swap::Behaviour::search_item_with]
        search_id: SearchID,
        /// Value that proxy searchs
        artifact: Artifact,
    }
}
