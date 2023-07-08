use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
    task::{Context, Poll},
};

use fnv::FnvHashSet;
use libp2p::{
    core::Endpoint,
    request_response::{self, ProtocolSupport, ResponseChannel},
    swarm::{
        behaviour::ConnectionEstablished,
        ConnectionClosed as RequestResponseConnectionClosed,
        ConnectionDenied,
        ConnectionId,
        FromSwarm,
        NetworkBehaviour,
        PollParameters,
        THandler,
        THandlerInEvent,
        THandlerOutEvent,
        ToSwarm,
    },
    Multiaddr,
    PeerId,
};
use log::debug;

use crate::{
    codec::QuantaSwapCodec,
    protocol::QuantaSwapProtocol,
    request::QuantaSwapRequest,
    response::QuantaSwapRespone,
    searchid::SearchID,
};

/// Base storage of QuantaSwap protocol. Any database can be used as storage (even in memory),
/// but I recommend using something like Rocksdb
pub trait Storage {
    /// Check if value with key exists
    fn exists(&self, key: Vec<u8>) -> bool;
    /// Get value by key
    fn get(&self, key: Vec<u8>) -> Option<Vec<u8>>;
}

/// Events that we are send out of this behaviour Here we got only 1 event - QueryCompleted
#[derive(Debug)]
pub enum Event {
    QueryCompleted {
        /// Who send this result
        peer: PeerId,
        /// Unqiue ID
        search_id: SearchID,
        /// Key in bytes of value
        /// that we are searched
        searching: Vec<u8>,
        /// Result of Query
        item: Vec<u8>,
    },
}

/// [`request_response::Behaviour`] with [`QuantaSwapCodec`]
///
/// NOTE: Create this type for better code readability
type RequestResponse = request_response::Behaviour<QuantaSwapCodec>;
/// Create this type for better code readability
type RequestResponseMessage =
    request_response::Message<QuantaSwapRequest, QuantaSwapRespone, QuantaSwapRespone>;
/// Create this type for better code readability
type ConnectionClosed<'l> =
    RequestResponseConnectionClosed<'l, <RequestResponse as NetworkBehaviour>::ConnectionHandler>;
/// Create this type for better code readability
type OutEventsQueue<S> =
    VecDeque<ToSwarm<<Behaviour<S> as NetworkBehaviour>::OutEvent, THandlerInEvent<Behaviour<S>>>>;
/// [`NetworkBehaviour`] for Quanta-swap
pub struct Behaviour<S>
where
    S: Storage + 'static,
{
    /// Communication between peers is implemented using the request_repsonse protocol
    request_response: RequestResponse,
    /// Storage is needed to check or receive data that will be sent later to other network members
    storage: Arc<S>,
    /// All active connections
    connections: FnvHashSet<PeerId>,
    /// All active queries.
    ///
    /// [`SearchID`] - Unique ID of query.
    /// Vec<u8> - Key in [`Storage`]
    /// that peer looking for
    queries: HashMap<SearchID, Vec<u8>>,
    /// Out events queue that we are send out of [`Behaviour`]
    out_evenets_queue: OutEventsQueue<S>,
}

impl<S> Behaviour<S>
where
    S: Storage + 'static,
{
    /// Create new [`Behaviour`]
    pub fn new(storage: Arc<S>) -> Self {
        let request_response = RequestResponse::new(
            QuantaSwapCodec,
            std::iter::once((QuantaSwapProtocol, ProtocolSupport::Full)),
            Default::default(),
        );
        let connections = FnvHashSet::default();
        let queries = HashMap::default();
        let out_evenets_queue = OutEventsQueue::<S>::default();
        Self {
            request_response,
            storage,
            connections,
            queries,
            out_evenets_queue,
        }
    }
    /// Call this function if you need create new search query. Search query create new
    /// random [`SearchID`] and sends [`QuantaSwapRequest::Query`] to all connections
    pub fn search_item_with(&mut self, searching: Vec<u8>) -> SearchID {
        let search_id = SearchID::random();
        debug!(
            "[`QuantaBehaviour`]: Strarted new search with id: {}",
            search_id
        );
        self.queries
            .entry(search_id)
            .or_insert(searching.clone());
        for peer in &self.connections {
            self.request_response
                .send_request(peer, QuantaSwapRequest::Query {
                    search_id,
                    searching: searching.to_vec(),
                });
        }
        search_id
    }
    /// Handle [`FromSwarm::ConnectionEstablished`] event and send it into [`RequestResponse`]
    fn on_connection_established(&mut self, connection_established: ConnectionEstablished) {
        // Send swarm connection_established event into request_response behaviour
        for (search_id, searching) in &self.queries {
            self.request_response.send_request(
                &connection_established.peer_id,
                QuantaSwapRequest::Query {
                    search_id: *search_id,
                    searching: searching.to_vec(),
                },
            );
        }
        self.request_response
            .on_swarm_event(FromSwarm::ConnectionEstablished(connection_established));
        // Insert new peer into connections
        self.connections
            .insert(connection_established.peer_id);
        // Send all active queries to new peer
    }
    /// Handle [`FromSwarm::ConnectionClosed`] event and send it into [`RequestResponse`]
    fn on_connection_closed(&mut self, connection_closed: ConnectionClosed) {
        // Send swarm connection_closed event into request_response behaviour
        self.connections
            .remove(&connection_closed.peer_id);
        self.request_response
            .on_swarm_event(FromSwarm::ConnectionClosed(connection_closed));
    }
    /// handle ch err
    fn handle_err_and_sent_response(
        &mut self,
        channel: ResponseChannel<QuantaSwapRespone>,
        response: QuantaSwapRespone,
    ) {
        self.request_response
            .send_response(channel, response)
            .expect("got unexpected err when trying to send response")
    }
    /// Handle [`QuantaSwapRequest`]
    fn handle_request_message(
        &mut self,
        request: QuantaSwapRequest,
        channel: ResponseChannel<QuantaSwapRespone>,
    ) -> Option<Event> {
        debug!("[`QuantaBehaviour`]: New Request={}", request);
        match request {
            QuantaSwapRequest::Query {
                search_id,
                searching,
            } => {
                let exists = self.storage.exists(searching);
                let response = QuantaSwapRespone::Query { search_id, exists };
                self.handle_err_and_sent_response(channel, response);
                None
            },
            QuantaSwapRequest::QueryWant {
                search_id,
                searching,
            } => {
                if let Some(item) = self.storage.get(searching) {
                    let response = QuantaSwapRespone::QueryWant { search_id, item };
                    self.handle_err_and_sent_response(channel, response);
                }
                None
            },
        }
    }
    /// Handle [`QuantaSwapRespone`]
    fn handle_response_message(
        &mut self,
        peer: PeerId,
        response: QuantaSwapRespone,
    ) -> Option<Event> {
        debug!("[`QuantaBehaviour`]: New Response={}", response);
        match response {
            QuantaSwapRespone::Query { search_id, exists } => {
                if exists {
                    if let Some(searching) = self.queries.get(&search_id) {
                        self.request_response
                            .send_request(&peer, QuantaSwapRequest::QueryWant {
                                search_id,
                                searching: searching.to_vec(),
                            });
                    };
                };
                None
            },
            QuantaSwapRespone::QueryWant { search_id, item } => {
                if let Some(searching) = self.queries.remove(&search_id) {
                    return Some(Event::QueryCompleted {
                        peer,
                        search_id,
                        searching,
                        item,
                    });
                }
                None
            },
        }
    }
    /// Handle all [`RequestResponse`] messages([`QuantaSwapRequest`], [`QuantaSwapRespone`])
    fn handle_request_response_message(
        &mut self,
        peer: PeerId,
        message: RequestResponseMessage,
    ) -> Option<Event> {
        match message {
            RequestResponseMessage::Request {
                request, channel, ..
            } => self.handle_request_message(request, channel),
            RequestResponseMessage::Response { response, .. } => {
                self.handle_response_message(peer, response)
            },
        }
    }
}
/// [`NetworkBehaviour`] impl
impl<S> NetworkBehaviour for Behaviour<S>
where
    S: Storage + 'static,
{
    type ConnectionHandler = <RequestResponse as NetworkBehaviour>::ConnectionHandler;
    type OutEvent = Event;
    /// Send into [`RequestResponse`]
    fn handle_established_inbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        peer: PeerId,
        local_addr: &Multiaddr,
        remote_addr: &Multiaddr,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.request_response
            .handle_established_inbound_connection(_connection_id, peer, local_addr, remote_addr)
    }
    /// Send into [`RequestResponse`]
    fn handle_pending_outbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        maybe_peer: Option<PeerId>,
        _addresses: &[Multiaddr],
        _effective_role: Endpoint,
    ) -> Result<Vec<Multiaddr>, ConnectionDenied> {
        self.request_response
            .handle_pending_outbound_connection(
                _connection_id,
                maybe_peer,
                _addresses,
                _effective_role,
            )
    }
    /// Send into [`RequestResponse`]
    fn handle_established_outbound_connection(
        &mut self,
        _connection_id: ConnectionId,
        peer: PeerId,
        addr: &Multiaddr,
        role_override: Endpoint,
    ) -> Result<THandler<Self>, ConnectionDenied> {
        self.request_response
            .handle_established_outbound_connection(_connection_id, peer, addr, role_override)
    }
    /// Handle events that we need and send they into [`RequestResponse`]
    fn on_swarm_event(&mut self, event: FromSwarm<Self::ConnectionHandler>) {
        match event {
            FromSwarm::ConnectionEstablished(connection_established) => {
                debug!("[`QuantaBehaviour`]: New ConnectionEstablished swarm event");
                self.on_connection_established(connection_established)
            },
            FromSwarm::ConnectionClosed(connection_closed) => {
                debug!("[`QuantaBehaviour`]: New ConnectionClosed swarm event");
                self.on_connection_closed(connection_closed)
            },
            // All events that we are need we are handle.
            // Other we just send into [`RequestReponse`]
            _ => {
                debug!("[`QuantaBehaviour`]: New swarm event");
                self.request_response
                    .on_swarm_event(event)
            },
        }
    }
    /// Send all connections handler events into [`RequestResponse`]
    fn on_connection_handler_event(
        &mut self,
        peer: PeerId,
        connection: ConnectionId,
        _event: THandlerOutEvent<Self>,
    ) {
        self.request_response
            .on_connection_handler_event(peer, connection, _event)
    }
    /// Poll
    fn poll(
        &mut self,
        _cx: &mut Context<'_>,
        _params: &mut impl PollParameters,
    ) -> Poll<ToSwarm<Self::OutEvent, THandlerInEvent<Self>>> {
        loop {
            if let Some(event) = self.out_evenets_queue.pop_front() {
                return Poll::Ready(event);
            };

            let event = self.request_response.poll(_cx, _params);
            debug!(
                "[`QuantaBehaviour`]: New event from [`RequestResposne`]: {:?}",
                event
            );
            match event {
                Poll::Ready(event) => {
                    if let ToSwarm::NotifyHandler {
                        peer_id,
                        handler,
                        event,
                    } = event
                    {
                        return Poll::Ready(ToSwarm::NotifyHandler {
                            peer_id,
                            handler,
                            event,
                        });
                    }
                    if let ToSwarm::GenerateEvent(request_response::Event::Message {
                        peer,
                        message,
                    }) = event
                    {
                        debug!(
                            "[`QuantaBehaviour`]: New [`RequestReponse`] message from peer: {:?}",
                            peer
                        );
                        if let Some(event) = self.handle_request_response_message(peer, message) {
                            self.out_evenets_queue
                                .push_back(ToSwarm::GenerateEvent(event))
                        };
                        continue;
                    }
                },
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}
