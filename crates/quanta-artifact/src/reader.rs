use std::{
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};

use futures::{AsyncRead, Stream};

use crate::{artifact::Artifact, MAX_ARTIFACT_SIZE};

/// Custom Artifact file reader
pub struct ArtifactStreamReader<'a, R: AsyncRead + Unpin> {
    /// Futures Reader
    reader: &'a mut R,
    /// Buffer
    buf: [u8; MAX_ARTIFACT_SIZE],
}

impl<'a, R: AsyncRead + Unpin> ArtifactStreamReader<'a, R> {
    /// Createw new [`ArtifactStreamReader`]
    pub fn new(reader: &'a mut R) -> Self {
        let buf = [0; MAX_ARTIFACT_SIZE];
        Self { reader, buf }
    }
}

/// Implementing a stream for the reader to stream artifacts from a file
impl<'a, R: AsyncRead + Unpin> Stream for ArtifactStreamReader<'a, R> {
    type Item = Result<Artifact>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let Self { reader, buf } = &mut *self;

        let size = match futures::ready!(Pin::new(reader).poll_read(cx, &mut buf[..])) {
            Ok(size) => size,
            Err(error) => return Poll::Ready(Some(Err(error))),
        };

        match size == 0 {
            // if the size of the loaded buf == 0, then we have completely read the file
            true => Poll::Ready(None),
            // If the size is not equal to zero, then we create a new artifact
            false => Poll::Ready(Some(Ok(Artifact::new(buf[..size].to_vec())))),
        }
    }
}
