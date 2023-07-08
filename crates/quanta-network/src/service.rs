use std::{collections::HashMap, sync::Arc};

use libp2p::{
    futures::StreamExt,
    identify,
    identity::Keypair,
    kad,
    mdns,
    noise,
    ping,
    swarm,
    tcp,
    yamux,
    PeerId,
    Swarm,
    Transport,
};
use log::{debug, error, info};
use quanta_artifact::{Artifact, ArtifactId};
use quanta_swap::Storage;
use tokio::sync;
use crate::{
    behaviour::{QuantaBehaviour, QuantaBehaviourEvent},
    info::ConnectionInfo,
    proxy::{FromNetworkEvent, IntoNetworkEvent, QuantaNetworkServiceProxy},
};

const CHANNELS_BUF_SIZE: usize = 2048 * 2;
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Got error when trying to decode bytes into ArtifactId: {0}")]
    /// Error whill occur when trying to decode bytes into [ArtifactId]
    ArtifactId(quanta_artifact::ArtifactIdError),
    #[error("Got error when trying to send event into proxy: {0}")]
    FromNetworkSend(#[from] sync::mpsc::error::SendError<FromNetworkEvent>),
    #[error("Gto error when trying to deal addr: {0}")]
    /// Error whill occur when we are trying [Swarm::dial]
    Dial(swarm::DialError),
}
/// QuantaNetwork is the backbone of the networking service on the quanta network. It defines
/// the swarm that [QuantaBehaviour] uses. Storing information about connected peers. to our node
/// And define several channels for interacting with the proxy, which in turn is used in the HTTP-API
pub struct QuantaNetwork<S>
where
    S: Storage + 'static,
{
    /// Swarm is used to communicate between peers on a network using protocols that have been
    /// defined in [QuantaBehaviour]
    swarm: Swarm<QuantaBehaviour<S>>,
    /// We store information about the connection with peers. Not all connections can be stored
    /// here, because we only store those that gave at least some response from protocols:
    /// [ping::Behaviour], [identify::Behaviour]
    connections: HashMap<PeerId, ConnectionInfo>,
    /// Proxy sender. Send [FromNetworkEvent] into proxy
    proxy_tx: sync::mpsc::Sender<FromNetworkEvent>,
    /// Proxy receiver. Receive [IntoNetworkEvent] from proxy
    network_rx: sync::mpsc::Receiver<IntoNetworkEvent>,
}

impl<S> QuantaNetwork<S>
where
    S: Storage + 'static,
{
    /// Create new [QuantaNetwork]
    pub fn new(
        keypair: &Keypair,
        local_peer_id: PeerId,
        storage: Arc<S>,
    ) -> (QuantaNetwork<S>, QuantaNetworkServiceProxy) {
        // create new swarm with async_io tcp transport, yamux multiplexer, noise authenticate
        // and quanta behaviour
        let mut swarm = swarm::SwarmBuilder::with_async_std_executor(
            tcp::async_io::Transport::new(tcp::Config::default().port_reuse(true))
                .upgrade(libp2p::core::upgrade::Version::V1)
                .authenticate(
                    noise::Config::new(keypair)
                        .expect("Got unexpected error when trying to create libp2p::noise::Config"),
                )
                .multiplex(yamux::Config::default())
                .boxed(),
            QuantaBehaviour::new(local_peer_id, keypair.public(), storage),
            local_peer_id,
        )
        .build();
        swarm
            .listen_on("/ip4/0.0.0.0/tcp/0".parse().unwrap())
            .unwrap();
        let (proxy_tx, proxy_rx) = sync::mpsc::channel(CHANNELS_BUF_SIZE);
        let (network_tx, network_rx) = sync::mpsc::channel(CHANNELS_BUF_SIZE);
        let connections = HashMap::default();
        (
            QuantaNetwork {
                swarm,
                connections,
                proxy_tx,
                network_rx,
            },
            QuantaNetworkServiceProxy::new(proxy_rx, network_tx),
        )
    }
    /// Handle events from [ping::Behaviour]. We are intersted only in Ok events.
    /// Result of event we are use for compile information about connections with peer
    async fn handle_ping(&mut self, event: ping::Event) -> Result<(), Error> {
        if let Ok(ping::Success::Ping { rtt }) = event.result {
            // update or create new info about connection with peer which id given in ping::Event
            self.connections
                .entry(event.peer)
                .or_insert(ConnectionInfo::default())
                .rtt = Some(rtt)
        };
        Ok(())
    }
    /// Handle events from [identify::Behaviour]. We are interested only in [identify::Event::Received]
    /// events. Result of event we are use for compile info about connection with peer
    async fn handle_identify(&mut self, event: identify::Event) -> Result<(), Error> {
        if let identify::Event::Received { peer_id, info } = event {
            // update or create new info about connection with peer which id given in event
            self.connections
                .entry(peer_id)
                .or_insert(ConnectionInfo::default())
                .identify_info = Some(info)
        };
        Ok(())
    }
    /// Handle events from [mdns::Behaviour]. Here we interested only in [mdns::Event::Discovered]
    /// event.
    async fn handle_mdns(&mut self, event: mdns::Event) -> Result<(), Error> {
        if let mdns::Event::Discovered(discovered_peers) = event {
            // update info about connection with peer and send add connection into kademlia
            for (peer, address) in discovered_peers {
                info!(
                    "MDNS Discovered Local Device with PeerId={} and Address={}",
                    peer, address
                );
                self.connections
                    .entry(peer)
                    .or_insert(ConnectionInfo::default())
                    .is_mdns = true;
                self.swarm
                    .behaviour_mut()
                    .kademlia
                    .add_address(&peer, address.clone());
                self.swarm.dial(address).map_err(Error::Dial)?;
            }
        };
        Ok(())
    }
    /// Just log [kad::KademliaEvent] and do nothing. We are not intersted in kademlia events
    async fn handle_kademlia(&mut self, event: kad::KademliaEvent) -> Result<(), Error> {
        debug!("Received new Kademlia event from swarm: {:?}", event);
        Ok(())
    }
    /// Handle event from [quanta_swap::Event]. QuantaSwap got only one event.
    async fn handle_quanta_swap(&mut self, event: quanta_swap::Event) -> Result<(), Error> {
        let quanta_swap::Event::QueryCompleted {
            search_id,
            searching,
            item,
            ..
        } = event;
        // just send FromNetworkEvent into proxy
        Ok(self
            .proxy_tx
            .send(FromNetworkEvent::QuantaSwapSearched {
                search_id,
                searching: ArtifactId::from_bytes(searching.as_slice())
                    .map_err(Error::ArtifactId)?,
                artifact: Artifact::new(item),
            })
            .await?)
    }
    /// Handle events that we are accept from [Swarm]. Events based on [QuantaBehaviour]
    async fn handle_swarm(&mut self, event: QuantaBehaviourEvent<S>) -> Result<(), Error> {
        match event {
            QuantaBehaviourEvent::QuantaSwap(event) => self.handle_quanta_swap(event).await,
            QuantaBehaviourEvent::Kademlia(event) => self.handle_kademlia(event).await,
            QuantaBehaviourEvent::Identify(event) => self.handle_identify(event).await,
            QuantaBehaviourEvent::Ping(event) => self.handle_ping(event).await,
            QuantaBehaviourEvent::Mdns(event) => self.handle_mdns(event).await,
        }
    }
    /// Handle events that we are accept from [QuantaNetworkServiceProxy]
    async fn handle_proxy(&mut self, event: IntoNetworkEvent) -> Result<(), Error> {
        match event {
            IntoNetworkEvent::GetConnections { response_channel } => {
                if let Err(_) = response_channel.send(self.connections.clone()) {
                    error!("Got SendError when sending connections from network to proxy")
                }
                Ok(())
            },
            IntoNetworkEvent::CreateSearch {
                searching,
                response_channel,
            } => {
                if let Err(_) = response_channel.send(
                    self.swarm
                        .behaviour_mut()
                        .quanta_swap
                        .search_item_with(searching.to_bytes()),
                ) {
                    error!("Got SendError when sending SearchId from network to proxy");
                }
                Ok(())
            },
        }
    }
    /// Run [QuantaNetwork] that check [Swarm] for new events and handle
    pub async fn run_and_handle(mut self) -> Result<(), Error> {
        loop {
            tokio::select! {
                swarm_event = self.swarm.select_next_some() => {
                    if let Ok(swarm_event) = swarm_event.try_into_behaviour_event() {
                        println!("{:?}", swarm_event);
                        if let Err(error) = self.handle_swarm(swarm_event).await {
                            error!(
                                "Got unexpected error when handling events from swarm: {}",
                                error
                            )
                        }
                    }
                }
                proxy_event = self.network_rx.recv() => {
                    if let Some(proxy_event) = proxy_event {
                        if let Err(error) = self.handle_proxy(proxy_event).await {
                            error!(
                                "Got unexpected error when handling events from proxy: {}",
                                error
                            )
                        }
                    }
                }
            }
        }
    }
}
