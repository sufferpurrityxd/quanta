use std::time::Duration;

use libp2p::identify;
/// Base information about connection
#[derive(Default, Debug, Clone)]
pub struct ConnectionInfo {
    /// Info that we can get from [identify::Behaviour] protocol
    pub(crate) identify_info: Option<identify::Info>,
    /// rtt to peer that we are can get from [libp2p::ping::Behaviour] protocol
    pub(crate) rtt: Option<Duration>,
    /// if peer_id discovered from mdns
    pub(crate) is_mdns: bool,
}
