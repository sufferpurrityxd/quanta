use {
    crate::{
        behaviour::{create_behaviour, Behaviour},
        proxy::{FromProxy, ToProxy},
        swarm::create_swarm,
        transport::create_transport,
    },
    futures::StreamExt,
    libp2p::{identity::Keypair, swarm::Swarm, PeerId},
    quanta_database::Repository,
    std::{sync::Arc, time::Duration},
    tokio::{
        select,
        sync::mpsc::{channel, Receiver, Sender},
        time::sleep,
    },
};
use crate::behaviour::BehaviourEvent;

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
    /// Receive events from proxy
    proxy_receiver_rx: Receiver<FromProxy>,
    /// Send events into proxy
    proxy_sender_tx: Sender<ToProxy>,
    /// Delay of kademlia walk in secs
    kad_walk_delay_secs: u64,
}

impl QuantaCore {
    /// Create new [QuantaCore] and proxy channels
    pub fn new(
        repository: Arc<Repository>,
        keypair: Keypair,
        local_peer_id: PeerId,
    ) -> (Self, Receiver<ToProxy>, Sender<FromProxy>) {
        let kad_walk_delay_secs = 60; // minute
        let transport = create_transport(&keypair).expect("failed to create transport");
        let behaviour = create_behaviour(Arc::clone(&repository), local_peer_id, &keypair);
        let swarm = create_swarm(transport, behaviour, local_peer_id);
        let (proxy_receiver_tx, proxy_receiver_rx) = channel::<FromProxy>(2048);
        let (proxy_sender_tx, proxy_sender_rx) = channel::<ToProxy>(2048);
        (
            Self {
                swarm,
                repository,
                proxy_receiver_rx,
                proxy_sender_tx,
                kad_walk_delay_secs,
            },
            proxy_sender_rx,
            proxy_receiver_tx,
        )
    }
    /// Handle events that we are accept from swarm
    async fn handle_swarm_event(&mut self, swarm_event: BehaviourEvent) -> Result<(), Error> {
        match swarm_event {
            BehaviourEvent::QuantaSwap(_) => {},
            BehaviourEvent::Kademlia(_) => {}
            BehaviourEvent::Ping(_) => {}
            BehaviourEvent::Identify(_) => {}
        };
        Ok(())
    }

    /// Handle proxy events that we
    /// are accept from [Receiver<ToProxy>]
    async fn handle_proxy_event(&mut self, proxy_event: FromProxy) -> Result<(), Error> {
        match proxy_event {
            FromProxy::QuantaSwapSearch { .. } => {}
        }
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
