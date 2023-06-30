use std::time::Duration;

/// Statistics with peer
#[derive(Debug, Clone, Default)]
pub struct PeerStatistic {
    /// Artifacts that we
    /// are received from peer
    artifacts_received: usize,
    /// Rtt to peer
    rtt: Duration,
}

impl PeerStatistic {
    /// update artifacts received
    pub fn increment_artifacts_received(&mut self) { self.artifacts_received += 1 }
    /// update rtt with peer
    pub fn update_rtt(&mut self, rtt: Duration) { self.rtt = rtt }
}
