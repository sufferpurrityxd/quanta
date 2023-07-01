use {
    crate::{
        behaviour::QuantaNetworkBehaviour,
        proxy::{ToProxyEvent, ToServiceEvent},
    },
    libp2p::{
        core::{muxing::StreamMuxerBox, transport::OrTransport, upgrade::Version},
        futures::future::Either,
        identity::Keypair,
        noise,
        quic,
        swarm::{self, SwarmBuilder},
        tcp,
        yamux,
        PeerId,
        Transport,
    },
    quanta_store::QuantaStore,
    std::sync::Arc,
    tokio::sync,
};

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
            let behaviour = QuantaNetworkBehaviour::new(storage, local_peer_id);
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
            SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id).build()
        };
        let (proxy_sender, proxy_receiver) = sync::mpsc::channel(2048);
        let (service_sender, service_receiver) = sync::mpsc::channel(2048);
        (
            Self {
                swarm,
                proxy_sender,
                service_receiver,
            },
            service_sender,
            proxy_receiver,
        )
    }
}
