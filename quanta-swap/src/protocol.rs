use libp2p::request_response::ProtocolName;

const PROTOCOL_NAME: &[u8] = b"/quanta/swap/0.0.1";

#[derive(Debug, Clone)]
pub struct QuantaSwapProtocol;

impl ProtocolName for QuantaSwapProtocol {
  fn protocol_name(&self) -> &[u8] { PROTOCOL_NAME }
}
