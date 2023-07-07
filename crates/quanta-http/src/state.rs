use {quanta_database::Database, std::sync::Arc};

/// Base [actix_web::web::Data] state of http proxy of quanta peer to peer data transfer protocol
pub(crate) struct HttpServerState {
    /// Quanta database used for writing artifacts/get info about magnet links and e.t.c
    database: Arc<Database>,
}

impl HttpServerState {
    /// Returns new [HttpServerState]
    pub fn new(database: Arc<Database>) -> Self { HttpServerState { database } }
}
