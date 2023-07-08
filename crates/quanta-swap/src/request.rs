use std::fmt::{Display, Formatter};

use prost::Message;

use crate::{
    protobuffable::{Protobuffable, ProtobuffableError},
    searchid::SearchID,
    swap_pb,
};

const QUERY_MESSAGE_TYPE: i32 = 0;
const QUERY_WANT_MESSAGE_TYPE: i32 = 1;

/// Requests which sends over network
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuantaSwapRequest {
    /// Check if new/existing peer have item
    Query {
        /// Unique id
        search_id: SearchID,
        /// Key in [`crate::storage::Storage`]
        searching: Vec<u8>,
    },
    /// Get item from peer
    QueryWant {
        /// Unique ID
        search_id: SearchID,
        /// Key in [`crate::storage::Storage`]
        searching: Vec<u8>,
    },
}

impl Protobuffable for QuantaSwapRequest {
    type ProtoValue = Vec<u8>;
    /// Convert [`Vec<u8>`] into [`QuantaSwapRequest`]
    fn from_proto(input: Self::ProtoValue) -> Result<Self, ProtobuffableError> {
        // Get ProtoResponse
        let proto = swap_pb::ProtoRequest::decode(input.as_slice())?;
        // Based on pb_type which sent in ProtoResponse get QuantaSwapRequest
        match proto.pb_type {
            QUERY_MESSAGE_TYPE => {
                let query = swap_pb::proto_request::ProtoQuery::decode(proto.message.as_slice())?;
                Ok(Self::Query {
                    search_id: SearchID::from_proto(query.search_id)?,
                    searching: query.searching,
                })
            },
            QUERY_WANT_MESSAGE_TYPE => {
                let query_want =
                    swap_pb::proto_request::ProtoQueryWant::decode(proto.message.as_slice())?;
                Ok(Self::QueryWant {
                    search_id: SearchID::from_proto(query_want.search_id)?,
                    searching: query_want.searching,
                })
            },
            _ => Err(ProtobuffableError::InvalidProtoMessageType),
        }
    }
    /// Convert [`QuantaSwapRequest`] into [`Vec<u8>`]
    fn to_proto(&self) -> Self::ProtoValue {
        swap_pb::ProtoRequest {
            message: match self {
                QuantaSwapRequest::Query {
                    search_id,
                    searching,
                } => swap_pb::proto_request::ProtoQuery {
                    search_id: search_id.to_proto(),
                    searching: searching.to_vec(),
                }
                .encode_to_vec(),
                QuantaSwapRequest::QueryWant {
                    search_id,
                    searching,
                } => swap_pb::proto_request::ProtoQueryWant {
                    search_id: search_id.to_proto(),
                    searching: searching.to_vec(),
                }
                .encode_to_vec(),
            },
            pb_type: match self {
                QuantaSwapRequest::Query { .. } => QUERY_MESSAGE_TYPE,
                QuantaSwapRequest::QueryWant { .. } => QUERY_WANT_MESSAGE_TYPE,
            },
        }
        .encode_to_vec()
    }
}

impl Display for QuantaSwapRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantaSwapRequest::Query { search_id, .. } => {
                write!(f, "[QuantaSwapRequest::Query], SEARCH_ID={}", search_id)
            },
            QuantaSwapRequest::QueryWant { search_id, .. } => {
                write!(f, "[QuantaSwapRequest::QueryWant], SEARCH_ID={}", search_id)
            },
        }
    }
}
