use {
    libp2p::{identify, identity::Keypair, kad, ping, swarm::NetworkBehaviour, PeerId},
    quanta_database::Repository,
    std::sync::Arc,
};

const IDENTIFY_PROTOCOL: &str = "/quanta/swap/0.0.1";

#[derive(Debug)]
/// Behaviour events
pub enum Event {
    /// Events from quanta swap
    QuantaSwap(quanta_swap::Event),
    /// Events from kademlia
    Kademlia(kad::KademliaEvent),
    /// Events from ping
    Ping(ping::Event),
    /// Events from identify
    Identify(identify::Event),
}

impl From<kad::KademliaEvent> for Event {
    fn from(value: kad::KademliaEvent) -> Self { Event::Kademlia(value) }
}

impl From<quanta_swap::Event> for Event {
    fn from(value: quanta_swap::Event) -> Self { Event::QuantaSwap(value) }
}

impl From<ping::Event> for Event {
    fn from(value: ping::Event) -> Self { Event::Ping(value) }
}

impl From<identify::Event> for Event {
    fn from(value: identify::Event) -> Self { Event::Identify(value) }
}

/// Main behaviour of quanta protocol
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "Event")]
pub struct Behaviour {
    /// Search artifacts in network
    quanta_swap: quanta_swap::Behaviour<Repository>,
    /// Search peers in network
    kademlia: kad::Kademlia<kad::store::MemoryStore>,
    ping: ping::Behaviour,
    identify: identify::Behaviour,
}

impl Behaviour {
    /// Create new behaviour
    pub fn new(repository: Arc<Repository>, local_peer_id: PeerId, keypair: &Keypair) -> Self {
        Self {
            quanta_swap: quanta_swap::Behaviour::new(repository),
            kademlia: kad::Kademlia::new(
                local_peer_id,
                kad::store::MemoryStore::new(local_peer_id),
            ),
            ping: ping::Behaviour::new(ping::Config::default()),
            identify: identify::Behaviour::new(identify::Config::new(
                String::from(IDENTIFY_PROTOCOL),
                keypair.public(),
            )),
        }
    }
}

pub fn create_behaviour(
    repository: Arc<Repository>,
    local_peer_id: PeerId,
    keypair: &Keypair,
) -> Behaviour {
    Behaviour::new(repository, local_peer_id, keypair)
}