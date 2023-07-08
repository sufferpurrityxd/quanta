use std::sync::Arc;

use libp2p::{identify, identity::PublicKey, kad, mdns, ping, swarm::NetworkBehaviour, PeerId};

const QUANTA_IDENTIFY_PROTOCOL_VERSION: &str = "/quanta/identify/0.0.1";

/// [QuantaBehaviour] defines the protocols that will be used in the quanta-network
#[derive(NetworkBehaviour)]
pub struct QuantaBehaviour<S>
where
    S: quanta_swap::Storage + 'static,
{
    /// [kad::Kademlia] is a DHT that used for peers discovery for more info see
    /// https://en.wikipedia.org/wiki/Kademlia
    pub(crate) kademlia: kad::Kademlia<kad::store::MemoryStore>,
    /// [quanta_swap::Behaviour] is a custom protocol that used for searching artifacts in network.
    pub(crate) quanta_swap: quanta_swap::Behaviour<S>,
    /// [identify::Behaviour] is a protocol that used for peers-identification. If we got this
    /// behaviour we know some [identify::Info] about peer that we can use in HTTP-API
    pub(crate) identify: identify::Behaviour,
    /// [ping::Behaviour] is a protocol that used for check rtt delay to peers.
    pub(crate) ping: ping::Behaviour,
    /// [mdns::async_io::Behaviour] is a protool that used for local-peers identification.
    /// using this protocol we can discover peers on the local network
    pub(crate) mdns: mdns::async_io::Behaviour,
}

impl<S> QuantaBehaviour<S>
where
    S: quanta_swap::Storage + 'static,
{
    /// Creates new [QuantaBehaviour]
    pub fn new(
        local_peer_id: PeerId,
        public_key: PublicKey,
        storage: Arc<S>,
    ) -> QuantaBehaviour<S> {
        let kademlia =
            kad::Kademlia::new(local_peer_id, kad::store::MemoryStore::new(local_peer_id));
        let quanta_swap = quanta_swap::Behaviour::new(storage);
        let identify = identify::Behaviour::new(identify::Config::new(
            QUANTA_IDENTIFY_PROTOCOL_VERSION.to_string(),
            public_key,
        ));
        let ping = ping::Behaviour::new(ping::Config::default());
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id)
            .expect("Got error when trying to create mdns::Behaviour");

        QuantaBehaviour {
            kademlia,
            quanta_swap,
            identify,
            ping,
            mdns,
        }
    }
}
