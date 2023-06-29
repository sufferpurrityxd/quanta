use {
    crate::{
        behaviour::{create_behaviour, Behaviour},
        swarm::create_swarm,
        transport::create_transport,
    },
    futures::StreamExt,
    libp2p::{identity::Keypair, swarm::Swarm, PeerId},
    quanta_database::Repository,
    std::{sync::Arc, time::Duration},
    tokio::{select, time::sleep},
};

#[derive(thiserror::Error, Debug)]
pub enum Error {}

/// [QuantaCore] - is a main struct of quanta networking
///
/// That got a database, utils for proxying, swarm and e.t.c
pub struct QuantaCore {
    /// Relation with peers
    swarm: Swarm<Behaviour>,
    /// Repository
    repository: Arc<Repository>,
    /// Delay of kademlia walk in secs
    kad_walk_delay_secs: u64,
}

impl QuantaCore {
    /// Create new [QuantaCore]
    pub fn new(repository: Arc<Repository>, keypair: Keypair, local_peer_id: PeerId) -> Self {
        let kad_walk_delay_secs = 60; // minute
        let transport = create_transport(&keypair).expect("failed to create transport");
        let behaviour = create_behaviour(Arc::clone(&repository), local_peer_id, &keypair);
        let swarm = create_swarm(transport, behaviour, local_peer_id);
        Self {
            swarm,
            repository,
            kad_walk_delay_secs,
        }
    }
    /// run core and handle events e.t.c
    pub async fn listen_and_handle(&mut self) -> Result<(), Error> {
        let kad_walk_sleep = sleep(Duration::from_secs(self.kad_walk_delay_secs));
        tokio::pin!(kad_walk_sleep);

        loop {
            select! {
                _from_swarm = self.swarm.select_next_some() => {}
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
