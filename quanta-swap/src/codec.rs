use {
    crate::{
        protobuffable::Protobuffable,
        protocol::QuantaSwapProtocol,
        request::QuantaSwapRequest,
        response::QuantaSwapRespone,
    },
    futures::{AsyncRead, AsyncWrite},
    libp2p::{core::upgrade, request_response::Codec},
};

#[derive(Debug, Clone)]
pub struct QuantaSwapCodec;

const MAX_BUFFER_SIZE: usize = 512 * 4;

#[async_trait::async_trait]
impl Codec for QuantaSwapCodec {
    type Protocol = QuantaSwapProtocol;
    type Request = QuantaSwapRequest;
    type Response = QuantaSwapRespone;

    async fn read_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        let bytes = upgrade::read_length_prefixed(io, MAX_BUFFER_SIZE).await?;
        QuantaSwapRequest::from_proto(bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    async fn read_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
    ) -> std::io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        let bytes = upgrade::read_length_prefixed(io, MAX_BUFFER_SIZE).await?;
        QuantaSwapRespone::from_proto(bytes)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))
    }

    async fn write_request<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let bytes = req.to_proto();
        upgrade::write_length_prefixed(io, bytes.as_slice()).await
    }

    async fn write_response<T>(
        &mut self,
        _protocol: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> std::io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        let bytes = res.to_proto();
        upgrade::write_length_prefixed(io, bytes.as_slice()).await
    }
}
