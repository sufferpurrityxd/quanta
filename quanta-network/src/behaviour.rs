use {
    libp2p::{kad, swarm::NetworkBehaviour, PeerId},
    std::sync::Arc,
};

/// Behaviour of quanta network. Main protocol that contains [kad::Kademlia],
/// [quanta_swap::Behaviour] protocols
#[derive(NetworkBehaviour)]
pub struct QuantaNetworkBehaviour {
    /// [quanta_swap::Behaviour] used for finding Artifacts in network
    quanta_swap: quanta_swap::Behaviour<quanta_store::QuantaStore>,
    /// [kad::Kademlia] Used for discovery peers in network
    kademlia: kad::Kademlia<kad::store::MemoryStore>,
}

impl QuantaNetworkBehaviour {
    /// Create new [QuantaNetworkBehaviour]
    pub fn new(storage: Arc<quanta_store::QuantaStore>, local_peer_id: PeerId) -> Self {
        let quanta_swap = quanta_swap::Behaviour::new(storage);
        let kademlia = {
            let store = kad::store::MemoryStore::new(local_peer_id);
            kad::Kademlia::new(local_peer_id, store)
        };
        QuantaNetworkBehaviour {
            quanta_swap,
            kademlia,
        }
    }
}
