use std::{
    collections::HashMap,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use futures::Stream;
use libp2p::PeerId;
use log::error;
use quanta_artifact::{Artifact, ArtifactId};
use quanta_swap::SearchID;
use tokio::sync;

use crate::info::ConnectionInfo;

#[derive(thiserror::Error, Debug)]
pub enum ProxyError {
    #[error("Got Recv timeout error when trying to recv response from network")]
    /// Error whill occur when trying to recv response from network
    RecvTimeout,
    #[error("Got error when trying to recv response from network: {0}")]
    /// Error whill occur in [timeout_oneshot_recv]
    Recv(sync::oneshot::error::RecvError),
    #[error("Got error when sending event into network: {0}")]
    /// Error whill occur when trying to send event into network
    Send(#[from] sync::mpsc::error::SendError<IntoNetworkEvent>),
}
/// Events that we are send from [`crate::service::QuantaNetwork`] into [`QuantaNetworkServiceProxy`]
#[derive(Debug, Clone)]
pub enum FromNetworkEvent {
    /// When receive [quanta_swap::Event::QueryCompleted] we are send this event to proxy
    QuantaSwapSearched {
        /// Unique id of search that we are receive from [quanta_swap::Behaviour::search_item_with]
        search_id: SearchID,
        /// Id of Artifact that user search
        searching: ArtifactId,
        /// Artifact that user search
        artifact: Artifact,
    },
}
/// Events that we are send from [`QuantaNetworkServiceProxy`] into [`crate::service::QuantaNetwork`]
#[derive(Debug)]
pub enum IntoNetworkEvent {
    /// Get all connections from network
    GetConnections {
        /// Over this channel network sends connections
        response_channel: sync::oneshot::Sender<HashMap<PeerId, ConnectionInfo>>,
    },
}
/// [`QuantaNetworkServiceProxy`] is a way to communicate with a service that is
/// running on a different thread [crate::service::QuantaNetwork].
/// Used in conjunication with various services (http-api e.t.c)
pub struct QuantaNetworkServiceProxy {
    /// Receive events from [crate::service::QuantaNetwork]
    proxy_rx: sync::mpsc::Receiver<FromNetworkEvent>,
    /// Send events into [crate::service::QuantaNetwork]
    network_tx: sync::mpsc::Sender<IntoNetworkEvent>,
}

impl QuantaNetworkServiceProxy {
    /// Create new [QuantaNetworkServiceProxy]
    pub fn new(
        proxy_rx: sync::mpsc::Receiver<FromNetworkEvent>,
        network_tx: sync::mpsc::Sender<IntoNetworkEvent>,
    ) -> Self {
        QuantaNetworkServiceProxy {
            proxy_rx,
            network_tx,
        }
    }
    /// Send event into [crate::service::QuantaNetwork] that we are want all identified connections
    /// and for response from [crate::service::QuantaNetwork]
    pub fn get_connections(&self) -> Result<HashMap<PeerId, ConnectionInfo>, ProxyError> {
        futures::executor::block_on(async move {
            let (response_channel, response_channel_rx) = sync::oneshot::channel();
            self.network_tx
                .send(IntoNetworkEvent::GetConnections { response_channel })
                .await?;
            Ok(timeout_oneshot_recv(response_channel_rx).await?)
        })
    }
}
/// Read response from oneshot channel that we are get when sending specific events into network
async fn timeout_oneshot_recv<R>(
    response_channel_rx: sync::oneshot::Receiver<R>,
) -> Result<R, ProxyError> {
    let duration = Duration::from_secs(10);
    Ok(tokio::time::timeout(duration, response_channel_rx)
        .await
        .map_err(|_| ProxyError::RecvTimeout)?
        .map_err(|why| ProxyError::Recv(why))?)
}

impl Stream for QuantaNetworkServiceProxy {
    type Item = FromNetworkEvent;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.proxy_rx.poll_recv(cx)
    }
}
