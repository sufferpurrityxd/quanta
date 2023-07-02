use {
    crate::{
        protobuffable::{Protobuffable, ProtobuffableError},
        searchid::SearchID,
        swap_pb,
    },
    prost::Message,
    std::fmt::{Display, Formatter},
};

const QUERY_RESPONSE_MESSAGE_TYPE: i32 = 0;
const QUERY_WANT_RESPONSE_MESSAGE_TYPE: i32 = 1;

/// Responses which sends over network
#[derive(Debug, Clone)]
pub enum QuantaSwapRespone {
    /// Response of [`crate::request::QuantaSwapRequest`]
    Query {
        /// Unique id
        search_id: SearchID,
        /// Check [`crate::Storage::exists`]
        /// and send it
        exists: bool,
    },
    /// Response
    QueryWant {
        /// Unique ID
        search_id: SearchID,
        /// Item that peer searching
        item: Vec<u8>,
    },
}

impl Protobuffable for QuantaSwapRespone {
    type ProtoValue = Vec<u8>;
    /// Convert [`Vec<u8>`] into [`QuantaSwapRespone`]
    fn from_proto(input: Self::ProtoValue) -> Result<Self, ProtobuffableError> {
        // Get ProtoRepsonse
        let proto = swap_pb::ProtoResponse::decode(input.as_slice())?;
        // Based on pb_type which sents in protoresponse get QuantasSwapResponse
        match proto.pb_type {
            QUERY_RESPONSE_MESSAGE_TYPE => {
                let query_response =
                    swap_pb::proto_response::ProtoQueryResponse::decode(proto.message.as_slice())?;
                Ok(Self::Query {
                    search_id: SearchID::from_proto(query_response.search_id)?,
                    exists: query_response.exists,
                })
            },
            QUERY_WANT_RESPONSE_MESSAGE_TYPE => {
                let query_want_response = swap_pb::proto_response::ProtoQueryWantResponse::decode(
                    proto.message.as_slice(),
                )?;
                Ok(Self::QueryWant {
                    search_id: SearchID::from_proto(query_want_response.search_id)?,
                    item: query_want_response.item,
                })
            },
            _ => Err(ProtobuffableError::InvalidProtoMessageType),
        }
    }

    fn to_proto(&self) -> Self::ProtoValue {
        swap_pb::ProtoResponse {
            message: match self {
                QuantaSwapRespone::Query { search_id, exists } => {
                    swap_pb::proto_response::ProtoQueryResponse {
                        search_id: search_id.to_proto(),
                        exists: *exists,
                    }
                    .encode_to_vec()
                },
                QuantaSwapRespone::QueryWant { search_id, item } => {
                    swap_pb::proto_response::ProtoQueryWantResponse {
                        search_id: search_id.to_proto(),
                        item: item.to_vec(),
                    }
                    .encode_to_vec()
                },
            },
            pb_type: match self {
                QuantaSwapRespone::Query { .. } => QUERY_RESPONSE_MESSAGE_TYPE,
                QuantaSwapRespone::QueryWant { .. } => QUERY_WANT_RESPONSE_MESSAGE_TYPE,
            },
        }
        .encode_to_vec()
    }
}

impl Display for QuantaSwapRespone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            QuantaSwapRespone::Query { search_id, exists } => {
                write!(
                    f,
                    "[QuantaSwapResponse::Query], SEARCH_ID={}, EXISTS={}",
                    search_id, exists
                )
            },
            QuantaSwapRespone::QueryWant { search_id, .. } => {
                write!(
                    f,
                    "[QuantaSwapResponse::QueryWant], SEARCH_ID={}",
                    search_id
                )
            },
        }
    }
}
