use libp2p::{
    core::{
        muxing::StreamMuxerBox,
        transport::{Boxed, OrTransport},
        upgrade::Version,
    },
    futures::future::Either,
    identity::Keypair,
    noise,
    quic,
    tcp,
    PeerId,
    Transport,
};

#[derive(thiserror::Error, Debug)]
pub enum CreateTransportError {
    #[error("Noise error: {0}")]
    /// Eror whill occur when creating new [noise::Config]
    Noise(#[from] noise::Error),
}

/// Create new transport and
/// use it in quanta protocol
///
/// Quanta supports two protocols - tcp and quic
pub fn create_transport(
    keypair: &Keypair,
) -> Result<Boxed<(PeerId, StreamMuxerBox)>, CreateTransportError> {
    Ok(OrTransport::new(
        tcp::async_io::Transport::new(tcp::Config::default())
            .upgrade(Version::V1)
            .authenticate(noise::Config::new(keypair)?)
            .multiplex(libp2p::yamux::Config::default())
            .boxed(),
        quic::async_std::Transport::new(quic::Config::new(keypair)).boxed(),
    )
    .map(|eo, _| match eo {
        Either::Left((p, m)) => (p, StreamMuxerBox::new(m)),
        Either::Right((p, m)) => (p, StreamMuxerBox::new(m)),
    })
    .boxed())
}
