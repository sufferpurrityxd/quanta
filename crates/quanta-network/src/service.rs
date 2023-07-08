use std::{collections::HashMap, sync::Arc};

use libp2p::{identity::Keypair, noise, swarm, tcp, yamux, PeerId, Swarm, Transport};
use quanta_swap::Storage;

use crate::{behaviour::QuantaBehaviour, info::ConnectionInfo};

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
    /// [libp2p::ping::Behaviour], [libp2p::identify::Behaviour]
    connections: HashMap<PeerId, ConnectionInfo>,
}

impl<S> QuantaNetwork<S>
where
    S: Storage + 'static,
{
    /// Create new [QuantaNetwork]
    pub fn new(keypair: &Keypair, local_peer_id: PeerId, storage: Arc<S>) -> QuantaNetwork<S> {
        // create new swarm with async_io tcp transport, yamux multiplexer, noise authenticate
        // and quanta behaviour
        let swarm = swarm::SwarmBuilder::with_async_std_executor(
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
        let connections = HashMap::default();
        QuantaNetwork { swarm, connections }
    }
}
