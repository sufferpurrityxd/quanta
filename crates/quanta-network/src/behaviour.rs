use {
    libp2p::{identify, identity::PublicKey, kad, ping, swarm::NetworkBehaviour, PeerId},
    std::sync::Arc,
};

/// Behaviour of quanta network. Main protocol that contains [kad::Kademlia],
/// [quanta_swap::Behaviour] protocols
#[derive(NetworkBehaviour)]
pub struct QuantaNetworkBehaviour {
    /// [quanta_swap::Behaviour] used for finding Artifacts in network
    pub(crate) quanta_swap: quanta_swap::Behaviour<quanta_store::QuantaStore>,
    /// [kad::Kademlia] used for discovery peers in network
    pub(crate) kademlia: kad::Kademlia<kad::store::MemoryStore>,
    /// [ping::Behaviour] used for update inforumation about rtt with connecitons
    pub(crate) ping: ping::Behaviour,
    /// [identify::Behaviour] used for update information about connections
    pub(crate) identify: identify::Behaviour,
}

impl QuantaNetworkBehaviour {
    /// Create new [QuantaNetworkBehaviour]
    pub fn new(
        storage: Arc<quanta_store::QuantaStore>,
        local_peer_id: PeerId,
        local_public_key: PublicKey,
    ) -> QuantaNetworkBehaviour {
        let quanta_swap = quanta_swap::Behaviour::new(storage);
        let kademlia =
            kad::Kademlia::new(local_peer_id, kad::store::MemoryStore::new(local_peer_id));
        let ping = ping::Behaviour::new(ping::Config::new());
        let identify = identify::Behaviour::new(identify::Config::new(
            String::from("/identify/quanta/0.0.1"),
            local_public_key,
        ));
        QuantaNetworkBehaviour {
            quanta_swap,
            kademlia,
            ping,
            identify,
        }
    }
}
