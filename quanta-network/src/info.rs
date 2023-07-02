use {
    libp2p::{identify::Info, Multiaddr},
    std::time::Duration,
};

/// Information about connection with [libp2p::PeerId]
#[derive(Clone, Debug, Default)]
pub struct ConnectionInformation {
    /// The addresses that the peer is listening on.
    listen_addrs: Vec<Multiaddr>,
    /// List of protocols that supported by [libp2p::PeerId]
    protocols: Vec<String>,
    /// Timeout to peer
    rtt: Duration,
}

impl ConnectionInformation {
    /// Update information about connection that we are received from [libp2p::identify::Event]
    pub fn update_with_identify(&mut self, info: Info) {
        self.listen_addrs = info.listen_addrs;
        self.protocols = info.protocols;
    }
    /// Update rtt that we are receive from [libp2p::ping::Event]
    pub fn rtt_update(&mut self, rtt: Duration) { self.rtt = rtt }
}
