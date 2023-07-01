/// Events that [crate::service::QuantaService] receive from proxy
#[derive(Debug, Clone)]
pub enum ToServiceEvent {}

/// Events that proxy receive from [crate::service::QuantaService]
#[derive(Debug, Clone)]
pub enum ToProxyEvent {}
