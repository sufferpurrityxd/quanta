use std::sync::Arc;

use quanta_database::Database;
use quanta_network::QuantaNetworkServiceProxy;

/// Base [actix_web::web::Data] state of http proxy of quanta peer to peer data transfer protocol
pub struct HttpServerState {
    /// Quanta database used for writing artifacts/get info about magnet links and e.t.c
    database: Arc<Database>,
    /// Proxy services for get info from diff thread
    network_proxy: Arc<QuantaNetworkServiceProxy>,
}

impl HttpServerState {
    /// Returns new [HttpServerState]
    pub fn new(database: Arc<Database>, network_proxy: Arc<QuantaNetworkServiceProxy>) -> Self {
        HttpServerState {
            database,
            network_proxy,
        }
    }
    /// Returns ref of [`Database`]
    pub fn database(&self) -> &Database { &self.database }
    /// Returns ref of [`QuantaNetworkServiceProxy`]
    pub fn network_proxy(&self) -> &QuantaNetworkServiceProxy { &self.network_proxy }
}
