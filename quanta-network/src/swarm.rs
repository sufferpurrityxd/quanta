use {
    crate::behaviour::Behaviour,
    libp2p::{
        core::{muxing::StreamMuxerBox, transport::Boxed},
        swarm::SwarmBuilder,
        PeerId,
        Swarm,
    },
    std::num::{NonZeroU8, NonZeroUsize},
};

/// Create a new swarm and send it to quantacore.
pub fn create_swarm(
    transport: Boxed<(PeerId, StreamMuxerBox)>,
    behaviour: Behaviour,
    local_peer_id: PeerId,
) -> Swarm<Behaviour> {
    SwarmBuilder::with_async_std_executor(transport, behaviour, local_peer_id)
        .notify_handler_buffer_size(NonZeroUsize::new(2 << 7).unwrap())
        .dial_concurrency_factor(NonZeroU8::new(8).unwrap())
        .per_connection_event_buffer_size(2 << 7)
        .build()
}
