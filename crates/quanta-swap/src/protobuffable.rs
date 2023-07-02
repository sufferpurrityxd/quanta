#[derive(Debug, thiserror::Error)]
pub enum ProtobuffableError {
    #[error("Multihash from proto error")]
    /// This error can result from trying to convert bytes that were received from the network
    /// to multihash based bytes
    Multihash(#[from] libp2p::multihash::Error),
    #[error("QueryID from proto error")]
    /// An error can get when we try to convert the bytes received as a result
    /// of to_proto to QueryID
    QueryId(#[from] crate::searchid::QueryIDError),
    #[error("Protobuf decode error")]
    /// Error whill occur when trying to decode protobuf bytes
    ProtobufDecode(#[from] prost::DecodeError),
    #[error("Invalid protobuf message type")]
    InvalidProtoMessageType,
}
/// if protobuffable is implemented for an object, then it can most likely be
/// transmitted over the network
pub trait Protobuffable
where
    Self: Sized,
{
    /// The value that we get from the network can be not only Vec<u8>, for example Vec<Peerid>
    /// turns into Vec<Vec<u8>>
    type ProtoValue;
    /// Convert the bytes that were received from protobuf to the value that we have locally in rust
    fn from_proto(input: Self::ProtoValue) -> Result<Self, ProtobuffableError>;
    /// Convert local object rust to protobuf based bytes
    fn to_proto(&self) -> Self::ProtoValue;
}

impl Protobuffable for libp2p::PeerId {
    type ProtoValue = Vec<u8>;
    /// Convert [`Vec<u8>`] which we are receive from network into [`libp2p::PeerId`]
    fn from_proto(input: Self::ProtoValue) -> Result<Self, ProtobuffableError> {
        Ok(Self::from_bytes(&input)?)
    }
    /// Convert [`libp2p::PeerId`] into [`Vec<u8>`] for sending it over network
    fn to_proto(&self) -> Self::ProtoValue { self.to_bytes() }
}

impl Protobuffable for Vec<libp2p::PeerId> {
    type ProtoValue = Vec<Vec<u8>>;
    /// Convert [`Vec<Vec<u8>>`] which we are receive from network into [`Vec<libp2p::PeerId>`]
    fn from_proto(input: Self::ProtoValue) -> Result<Self, ProtobuffableError> {
        Ok(input
            .into_iter()
            .map(libp2p::PeerId::from_proto)
            .filter(|result| result.is_ok())
            .flatten()
            .collect())
    }
    /// Convert [`Vec<libp2p::PeerId>`] into [`Vec<Vec<u8>>`] for sending it over network
    fn to_proto(&self) -> Self::ProtoValue {
        self.iter()
            .map(|peer_id| peer_id.to_proto())
            .collect()
    }
}
