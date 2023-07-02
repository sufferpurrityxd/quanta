use {
    crate::{protobuffable::Protobuffable, request::QuantaSwapRequest, searchid::SearchID},
    libp2p::PeerId,
};

#[test]
fn test_query_id() {
    let query_id = SearchID::random();
    let bytes_query_id = query_id.to_bytes();
    let from_bytes_query_id = SearchID::from_bytes(bytes_query_id).unwrap();
    assert_eq!(query_id, from_bytes_query_id);
    assert_eq!(query_id.to_string(), from_bytes_query_id.to_string());
}

#[test]
fn test_proto_query_id() {
    let query_id = SearchID::random();
    let proto_query_id = query_id.to_proto();
    let from_proto_query_id = SearchID::from_proto(proto_query_id).unwrap();
    assert_eq!(query_id, from_proto_query_id);
}

#[test]
fn test_proto_peer_id() {
    let peer_id = PeerId::random();
    let proto_peer_id = peer_id.to_proto();
    let from_proto_peer_id = PeerId::from_proto(proto_peer_id).unwrap();
    assert_eq!(peer_id, from_proto_peer_id)
}

#[test]
fn test_proto_vec_peer_id() {
    let vec_peer_id = Vec::from([
        PeerId::random(),
        PeerId::random(),
        PeerId::random(),
        PeerId::random(),
    ]);
    let proto_vec_peer_id = vec_peer_id.to_proto();
    let from_proto_vec_peer_id =
        <Vec<PeerId> as Protobuffable>::from_proto(proto_vec_peer_id).unwrap();
    assert_eq!(vec_peer_id, from_proto_vec_peer_id);
}

#[test]
fn test_request_query() {
    let request = QuantaSwapRequest::Query {
        search_id: SearchID::random(),
        searching: b"beep boop".to_vec(),
    };
    let proto_bytes_request = request.to_proto();
    let from_proto_request = QuantaSwapRequest::from_proto(proto_bytes_request).unwrap();
    assert_eq!(request, from_proto_request);
}

#[test]
fn test_request_query_want() {
    let request = QuantaSwapRequest::QueryWant {
        search_id: SearchID::random(),
        searching: b"beep boop".to_vec(),
    };
    let proto_bytes_request = request.to_proto();
    let from_proto_request = QuantaSwapRequest::from_proto(proto_bytes_request).unwrap();
    assert_eq!(request, from_proto_request);
}
