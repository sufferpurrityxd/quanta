use {
    crate::info::ConnectionInformation,
    futures::executor::block_on,
    libp2p::{futures, PeerId},
    quanta_artifact::{Artifact, ArtifactId},
    quanta_swap::SearchID,
    std::{collections::HashMap, time::Duration},
    tokio::sync,
};
/// Timeout in seconds before response from [QuantaServiceProxy]
const TIMEOUT_PROXY_RECV: u64 = 15;
#[derive(thiserror::Error, Debug)]
pub enum ProxyError {
    #[error("Response timeout from service error")]
    /// Error whill occur if [crate::service::QuantaService] doesnot send
    /// response on [ToServiceEvent]
    ResponseTimeout,
    #[error("From service response recv error")]
    /// Error whill occur when we are trying to recv response from [crate::service::QuantaService]
    FromServiceResponseRecv(#[from] sync::oneshot::error::RecvError),
    #[error("To Service send error")]
    /// Error whill occur when trying to send evenets to [crate::service::QuantaService]
    ToServiceSend(#[from] sync::mpsc::error::SendError<ToServiceEvent>),
}
/// Events that [crate::service::QuantaService] receive from proxy
#[derive(Debug)]
pub enum ToServiceEvent {
    /// Proxy sends this event when he wants all connections with node
    ///
    /// Here we got [sync::oneshot::Sender] that we want send to
    GetConnections(sync::oneshot::Sender<HashMap<PeerId, ConnectionInformation>>),
    /// Proxy sends this event when he wants create new search that implemented by
    /// [quanta_swap::Behaviour]
    CreateQuantaSearch {
        /// [ArtifactId] of [Artifact] in [quanta_store::QuantaStore]
        searching: ArtifactId,
        /// [:sync::oneshot::Sender] that send [SearchID] which received from
        /// [quanta_swap::Behaviour::search_item_with]
        ch: sync::oneshot::Sender<SearchID>,
    },
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
    },
}
/// A proxy is a way to communicate with a service that is running on a different thread.
/// Used in conjunction with various interfaces (CLI, API, GPRC)
///
/// Send events into [crate::service::QuantaService]
pub struct QuantaServiceRequestResponseProxy(sync::mpsc::Sender<ToServiceEvent>);
impl QuantaServiceRequestResponseProxy {
    /// Create new [QuantaServiceProxy]. That channels gets when we are create new
    /// [crate::service::QuantaService]
    pub fn new(service_sender: sync::mpsc::Sender<ToServiceEvent>) -> Self {
        QuantaServiceRequestResponseProxy(service_sender)
    }
    /// Send event [ToServiceEvent::GetConnections] that we are want all identified connections
    /// and wait for response from [crate::service::QuantaService]
    pub fn get_connections(&self) -> Result<HashMap<PeerId, ConnectionInformation>, ProxyError> {
        block_on(async move {
            let (tx, rx) = sync::oneshot::channel();
            self.0
                .send(ToServiceEvent::GetConnections(tx))
                .await?;
            timeout_oneshot_channel_recv(rx, TIMEOUT_PROXY_RECV).await
        })
    }
    /// Send event [ToServiceEvent::CreateQuantaSearch] that we are want to create new search with
    /// [ArtifactId] that specified in func args
    pub fn create_quanta_search(&self, searching: ArtifactId) -> Result<SearchID, ProxyError> {
        block_on(async move {
            let (tx, rx) = sync::oneshot::channel();
            self.0
                .send(ToServiceEvent::CreateQuantaSearch { searching, ch: tx })
                .await?;
            timeout_oneshot_channel_recv(rx, TIMEOUT_PROXY_RECV).await
        })
    }
}
/// Read for response with timeout that specified in [TIMEOUT_PROXY_RECV]
async fn timeout_oneshot_channel_recv<V>(
    channel_recv: sync::oneshot::Receiver<V>,
    timeout_secs: u64,
) -> Result<V, ProxyError> {
    let duration = Duration::from_secs(timeout_secs);
    Ok(tokio::time::timeout(duration, channel_recv)
        .await
        .map_err(|_| ProxyError::ResponseTimeout)??)
}
