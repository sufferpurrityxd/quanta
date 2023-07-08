use std::time::Duration;

use libp2p::{identify, Multiaddr};

/// [identify::Info] but with serde derives
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IdentifyInfoSerde {
    /// Application-specific version of the protocol family used by the peer,
    /// e.g. `ipfs/1.0.0` or `polkadot/1.0.0`.
    pub protocol_version: String,
    /// Name and version of the peer, similar to the `User-Agent` header in
    /// the HTTP protocol.
    pub agent_version: String,
    /// The addresses that the peer is listening on.
    pub listen_addrs: Vec<Multiaddr>,
    /// The list of protocols supported by the peer, e.g. `/ipfs/ping/1.0.0`.
    pub protocols: Vec<String>,
    /// Address observed by or for the remote.
    pub observed_addr: Multiaddr,
}

/// Implement from from identify info into serde derives info
impl From<identify::Info> for IdentifyInfoSerde {
    fn from(value: identify::Info) -> Self {
        IdentifyInfoSerde {
            protocol_version: value.protocol_version,
            agent_version: value.agent_version,
            listen_addrs: value.listen_addrs,
            protocols: value.protocols,
            observed_addr: value.observed_addr,
        }
    }
}

/// Base information about connection
#[derive(Default, Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConnectionInfo {
    /// Info that we can get from [identify::Behaviour] protocol
    pub(crate) identify_info: Option<IdentifyInfoSerde>,
    /// rtt to peer that we are can get from [libp2p::ping::Behaviour] protocol
    pub(crate) rtt: Option<Duration>,
    /// if peer_id discovered from mdns
    pub(crate) is_mdns: bool,
}
