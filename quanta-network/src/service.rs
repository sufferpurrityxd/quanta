use {
    crate::{
        behaviour::{QuantaNetworkBehaviour, QuantaNetworkBehaviourEvent},
        info::ConnectionInformation,
        proxy::{ToProxyEvent, ToServiceEvent},
    },
    libp2p::{
        core::{muxing::StreamMuxerBox, transport::OrTransport, upgrade::Version},
        futures::{executor::block_on, future::Either, StreamExt},
        identify,
        identity::Keypair,
        noise,
        ping,
        quic,
        swarm::{self},
        tcp,
        yamux,
        PeerId,
        Transport,
    },
    log::error,
    quanta_artifact::Artifact,
    quanta_store::QuantaStore,
    std::{collections::HashMap, sync::Arc},
    tokio::sync,
};
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Got error when trying to send event from service into proxy: {0}")]
    /// Error whill occur when trying to send some information [ToProxyEvent] over channel that
    /// specified in [QuantaService]
    ProxySend(#[from] sync::mpsc::error::SendError<ToProxyEvent>),
}
/// Delay of walk runner: [kad::Kademlia::get_closest_peers]
const KADEMLIA_DELAY_WALK: u64 = 60;
/// Main module of quanta netowrk is [QuantaService].
///
/// [QuantaService] manages local node and receive events from [swarm::Swarm]
/// Also [QuantaService] send info around [sync::mpsc::channel] into proxy
///
/// [QuantaService] creates two transports. QUIC ( https://en.wikipedia.org/wiki/QUIC ) or TCP
/// Yamux being used as a multiplexer
/// Noise being used as a auth protocol
pub struct QuantaService {
    /// [swarm::Swarm] with [QuantaNetworkBehaviour].
    swarm: swarm::Swarm<QuantaNetworkBehaviour>,
    /// Send channel that send event to proxy
    proxy_sender: sync::mpsc::Sender<ToProxyEvent>,
    /// Receive channel that receive events from proxy
    service_receiver: sync::mpsc::Receiver<ToServiceEvent>,
    /// Map connection with peer and statistics
    connections: HashMap<PeerId, ConnectionInformation>,
}

impl QuantaService {
    /// Creates new [QuantaService] with QUIC or TCP Transport and default parameters,
    /// and [sync::mpsc::channel] that used for communication with proxy
    pub fn new(
        local_peer_id: PeerId,
        keypair: Keypair,
        storage: Arc<QuantaStore>,
    ) -> (
        Self,
        sync::mpsc::Sender<ToServiceEvent>,
        sync::mpsc::Receiver<ToProxyEvent>,
    ) {
        let swarm = {
            let local_public_key = keypair.public();
            let behaviour = QuantaNetworkBehaviour::new(storage, local_peer_id, local_public_key);
            let transport =
                OrTransport::new(
                    tcp::async_io::Transport::new(tcp::Config::default())
                        .upgrade(Version::V1)
                        .authenticate(noise::Config::new(&keypair).expect(
                            "Got error when trying to create noise::Config with given Keypair",
                        ))
                        .multiplex(yamux::Config::default())
                        .boxed(),
                    quic::async_std::Transport::new(quic::Config::new(&keypair)),
                )
                .map(|eo, _| match eo {
                    Either::Right((p, m)) => (p, StreamMuxerBox::new(m)),
                    Either::Left((p, m)) => (p, StreamMuxerBox::new(m)),
                })
                .boxed();
            swarm::SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id)
                .build()
        };
        let (proxy_sender, proxy_receiver) = sync::mpsc::channel(2048);
        let (service_sender, service_receiver) = sync::mpsc::channel(2048);
        let connections = HashMap::new();
        (
            Self {
                swarm,
                proxy_sender,
                service_receiver,
                connections,
            },
            service_sender,
            proxy_receiver,
        )
    }
    /// Handle events from [ping::Behaviour]
    async fn handle_ping(&mut self, event: ping::Event) -> Result<(), Error> {
        // we are intersted only in success and ping events
        if let Ok(ping::Success::Ping { rtt }) = event.result {
            // so we need just update rtt
            self.connections
                .entry(event.peer)
                .or_insert(ConnectionInformation::default())
                .rtt_update(rtt)
        }

        Ok(())
    }
    /// Handle events from [identify::Behaviour]
    async fn handle_identify(&mut self, event: identify::Event) -> Result<(), Error> {
        // we are intersted only in identify::Event::Received
        if let identify::Event::Received { peer_id, info } = event {
            // update some basic info about peer that connected.
            self.connections
                .entry(peer_id)
                .or_insert(ConnectionInformation::default())
                .update_with_identify(info)
        }
        Ok(())
    }
    /// Handle events from [quanta_swap::Behaviour]
    async fn handle_quanta_swap(&mut self, event: quanta_swap::Event) -> Result<(), Error> {
        // quanta swap got only one event
        let quanta_swap::Event::QueryCompleted {
            search_id, item, ..
        } = event;

        // just send event to proxy
        self.proxy_sender
            .send(ToProxyEvent::QuantaSearchReady {
                search_id,
                artifact: Artifact::new(item),
            })
            .await?;
        Ok(())
    }
    /// Handle events that we are receive from [swarm::Swarm::select_next_some]
    async fn handle_swarm_event(
        &mut self,
        event: QuantaNetworkBehaviourEvent,
    ) -> Result<(), Error> {
        match event {
            QuantaNetworkBehaviourEvent::QuantaSwap(event) => self.handle_quanta_swap(event).await,
            QuantaNetworkBehaviourEvent::Ping(event) => self.handle_ping(event).await,
            QuantaNetworkBehaviourEvent::Identify(event) => self.handle_identify(event).await,
            _ => Ok(()),
        }
    }
    /// Handle events that we are receive from [sync::mpsc::Sender<ToServiceEvent>]
    async fn handle_proxy_event(&mut self, proxy_event: ToServiceEvent) -> Result<(), Error> {
        match proxy_event {
            ToServiceEvent::GetConnections(ch) => {
                // Send all connections that we are got into proxy receiver channel
                if ch
                    .send(self.connections.clone())
                    .is_err()
                {
                    error!("err when trying to send connections to proxy")
                };
                Ok(())
            },
            ToServiceEvent::CreateQuantaSearch { searching, ch } => {
                // create new search with quanta_swap and send unique id of this search into proxy
                if ch
                    .send(
                        self.swarm
                            .behaviour_mut()
                            .quanta_swap
                            .search_item_with(searching.to_bytes()),
                    )
                    .is_err()
                {
                    error!("err when trying to send search id to proxy")
                }
                Ok(())
            },
        }
    }
    /// Run [QuantaService]. Here we waiting service_receiver or swarm events and handle it
    pub fn start_handle_service(mut self) -> Result<(), Error> {
        block_on(async move {
            loop {
                tokio::select! {
                    swarm_event = self.swarm.select_next_some() => {
                        if let Ok(swarm_event) = swarm_event.try_into_behaviour_event() {
                            self.handle_swarm_event(swarm_event).await?;
                        }
                    }
                    proxy_event = self.service_receiver.recv() => {
                        if let Some(proxy_event) = proxy_event {
                            self.handle_proxy_event(proxy_event).await?;
                        };
                    }
                }
            }
        })
    }
}
