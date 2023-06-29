use {
    crate::{
        behaviour::{create_behaviour, Behaviour},
        swarm::create_swarm,
        transport::create_transport,
    },
    libp2p::{identity::Keypair, swarm::Swarm, PeerId},
    quanta_database::Repository,
    std::sync::Arc,
};

/// [QuantaCore] - is a main struct of quanta networking
///
/// That got a database, utils for proxying, swarm and e.t.c
pub struct QuantaCore {
    /// Relation with peers
    swarm: Swarm<Behaviour>,
    /// Repository
    repository: Arc<Repository>,
}

impl QuantaCore {
    /// Create new [QuantaCore]
    pub fn new(repository: Arc<Repository>, keypair: Keypair, local_peer_id: PeerId) -> Self {
        Self {
            swarm: create_swarm(
                create_transport(&keypair).expect("failed to create transport"),
                create_behaviour(Arc::clone(&repository), local_peer_id, &keypair),
                local_peer_id,
            ),
            repository,
        }
    }
}
