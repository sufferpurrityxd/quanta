use {
    crate::{
        behaviour::{create_behaviour, Behaviour, BehaviourEvent},
        statistics::PeerStatistic,
        swarm::create_swarm,
        transport::create_transport,
    },
    futures::StreamExt,
    libp2p::{identity::Keypair, kad, ping, swarm::Swarm, Multiaddr, PeerId},
    log::{error, trace},
    quanta_artifact::{Artifact, ArtifactId},
    quanta_database::Repository,
    quanta_swap::SearchID,
    std::{collections::HashMap, sync::Arc, time::Duration},
    tokio::{
        select,
        sync::{mpsc, oneshot},
        time::sleep,
    },
};

/// Enum that we are accept
/// from proxy and do something with
pub enum FromProxy {
    /// When we receive this event that
    /// means we should start new quanta search
    QuantaSwapSearch {
        /// Id that we are looking for
        artifact_id: ArtifactId,
    },
    /// Send connected peers into proxy
    GetConnectedPeers {
        /// Over this channel peers
        /// sends from core to proxy
        ch: oneshot::Sender<HashMap<PeerId, PeerStatistic>>,
    },
    /// Send info about listeners into proxy
    GetListeners {
        /// Over this channel multiaddrs
        /// sends from core to proxy
        ch: oneshot::Sender<Vec<Multiaddr>>,
    },
}

/// This enum we send into proxy
#[derive(Debug, Clone)]
pub enum ToProxy {
    /// When search is complete than we send this
    QuantaSwapSearchCompleted {
        search_id: SearchID,
        /// Artifact
        artifact: Artifact,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Artifact id decode error")]
    /// Error whill occur when
    /// trying to get artifact id from bytes
    ArtifactId(#[from] quanta_artifact::ArtifactIdError),
    #[error("To proxy send error")]
    /// Error whill occur when trying
    /// to send [ToProxy] enum over channels
    Send(#[from] mpsc::error::SendError<ToProxy>),
}

/// [QuantaCore] - is a main struct of quanta networking
///
/// That got a database, utils for proxying, swarm and e.t.c
pub struct QuantaCore {
    /// Relation with peers
    swarm: Swarm<Behaviour>,
    /// Repository
    repository: Arc<Repository>,
    /// Receive events from proxy
    proxy_receiver_rx: mpsc::Receiver<FromProxy>,
    /// Send events into proxy
    proxy_sender_tx: mpsc::Sender<ToProxy>,
    /// Delay of kademlia walk in secs
    kad_walk_delay_secs: u64,
    /// Statistics
    peers_stat: HashMap<PeerId, PeerStatistic>,
}

impl QuantaCore {
    /// Create new [QuantaCore] and proxy channels
    pub fn new(
        repository: Arc<Repository>,
        keypair: Keypair,
        local_peer_id: PeerId,
    ) -> (Self, mpsc::Receiver<ToProxy>, mpsc::Sender<FromProxy>) {
        let kad_walk_delay_secs = 60; // minute
        let transport = create_transport(&keypair).expect("failed to create transport");
        let behaviour = create_behaviour(Arc::clone(&repository), local_peer_id);
        let swarm = create_swarm(transport, behaviour, local_peer_id);
        let (proxy_receiver_tx, proxy_receiver_rx) = mpsc::channel::<FromProxy>(2048);
        let (proxy_sender_tx, proxy_sender_rx) = mpsc::channel::<ToProxy>(2048);
        let peers_stat = HashMap::new();
        (
            Self {
                swarm,
                repository,
                proxy_receiver_rx,
                proxy_sender_tx,
                kad_walk_delay_secs,
                peers_stat,
            },
            proxy_sender_rx,
            proxy_receiver_tx,
        )
    }
    /// handle events from [quanta_swap::Behaviour]
    async fn handle_quanta_swap(&mut self, event: quanta_swap::Event) -> Result<(), Error> {
        let quanta_swap::Event::QueryCompleted {
            peer,
            search_id,
            item,
            ..
        } = event;
        self.proxy_sender_tx
            .send(ToProxy::QuantaSwapSearchCompleted {
                search_id,
                artifact: Artifact::new(item),
            })
            .await?;
        // update statistics
        self.peers_stat
            .entry(peer)
            .or_insert(PeerStatistic::default())
            .increment_artifacts_received();
        Ok(())
    }
    /// handle events from [kad::KademliaEvent]
    async fn handle_kad(&mut self, event: kad::KademliaEvent) -> Result<(), Error> {
        match event {
            kad::KademliaEvent::InboundRequest { .. } => {},
            kad::KademliaEvent::OutboundQueryProgressed { .. } => {},
            kad::KademliaEvent::RoutingUpdated { .. } => {},
            kad::KademliaEvent::UnroutablePeer { .. } => {},
            kad::KademliaEvent::RoutablePeer { .. } => {},
            kad::KademliaEvent::PendingRoutablePeer { .. } => {},
        };
        Ok(())
    }
    /// handle events from [ping::Event]
    async fn handle_ping(&mut self, event: ping::Event) -> Result<(), Error> {
        match event.result {
            Ok(ping::Success::Ping { rtt }) => self
                .peers_stat
                .entry(event.peer)
                .or_insert(PeerStatistic::default())
                .update_rtt(rtt),
            Ok(ping::Success::Pong) => trace!("pong from peer: {}", event.peer),
            Err(ping::Failure::Timeout) => trace!("timeout failure from peer: {}", event.peer),
            Err(ping::Failure::Other { error }) => {
                trace!("unknown error: {} from peer: {}", error, event.peer)
            },
            Err(ping::Failure::Unsupported) => {
                trace!("unsupported ping protocol from peer: {}", event.peer)
            },
        };
        Ok(())
    }
    /// Handle events that we are accept from swarm
    async fn handle_swarm_event(&mut self, swarm_event: BehaviourEvent) -> Result<(), Error> {
        match swarm_event {
            BehaviourEvent::QuantaSwap(event) => self.handle_quanta_swap(event).await,
            BehaviourEvent::Kademlia(event) => self.handle_kad(event).await,
            BehaviourEvent::Ping(event) => self.handle_ping(event).await,
        }
    }

    /// Handle proxy events that we
    /// are accept from [Receiver<ToProxy>]
    async fn handle_proxy_event(&mut self, proxy_event: FromProxy) -> Result<(), Error> {
        match proxy_event {
            FromProxy::QuantaSwapSearch { artifact_id } => {
                self.swarm
                    .behaviour_mut()
                    .quanta_swap
                    .search_item_with(artifact_id.to_bytes());
            },
            FromProxy::GetConnectedPeers { ch } => {
                if let Err(_) = ch.send(self.peers_stat.clone()) {
                    error!("got unexpected error when sending connected peers to proxy");
                };
            },
            FromProxy::GetListeners { ch } => {
                if let Err(_) = ch.send(
                    self.swarm
                        .listeners()
                        .map(|multiaddr| multiaddr.clone())
                        .collect::<Vec<Multiaddr>>(),
                ) {
                    error!("got unexpected error when sending connected peers to proxy");
                };
            },
        };
        Ok(())
    }

    /// run core and handle events e.t.c
    pub async fn listen_and_handle(&mut self) -> Result<(), Error> {
        let kad_walk_sleep = sleep(Duration::from_secs(self.kad_walk_delay_secs));
        tokio::pin!(kad_walk_sleep);

        loop {
            select! {
                swarm_event = self.swarm.select_next_some() => {
                    if let Ok(swarm_event) = swarm_event.try_into_behaviour_event() {
                        self.handle_swarm_event(swarm_event).await?;
                    }
                }
                proxy_event = self.proxy_receiver_rx.recv() => {
                    if let Some(proxy_event) = proxy_event {
                        self.handle_proxy_event(proxy_event).await?;
                    }
                }
                _ = &mut kad_walk_sleep => {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .get_closest_peers(PeerId::random());
                    kad_walk_sleep
                        .as_mut()
                        .reset(tokio::time::Instant::now() + Duration::from_secs(self.kad_walk_delay_secs));
                }
            }
        }
    }
}
