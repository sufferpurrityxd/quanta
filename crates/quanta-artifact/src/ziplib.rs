use {
    libflate::gzip::{Decoder, Encoder},
    std::io::{Read, Write},
};

#[derive(thiserror::Error, Debug)]
pub enum ZipLibError {
    #[error("Into Result")]
    /// Eror whill occur when convert finish state into result when trying to encode into
    /// gzip-bytes
    IntoResult,
    #[error("Gzip decoder creation error")]
    /// Error whill occur when trying to create new gzip decoder
    CreateGzipDecoder,
    #[error("Gzip encoder creation error")]
    /// Error whill occur when trying to create new gzip encoder
    CreateGzipEncoder,
    #[error("IO error")]
    /// Eror whill occur when decoding/encoding bytes
    IO,
}
/// Decode all input gzip-bytes
///
/// Accept: gzip-bytes
/// Returns: source bytes
pub fn decode_gzip_all(input: &[u8]) -> Result<Vec<u8>, ZipLibError> {
    // Creat new decoder
    let mut decoder = Decoder::new(input).map_err(|_| ZipLibError::CreateGzipDecoder)?;
    // Buffer
    let mut buf = Vec::new();
    // Read bytes into buffer
    decoder
        .read_to_end(&mut buf)
        .map_err(|_| ZipLibError::IO)?;

    Ok(buf)
}

/// Encode all input bytes into gzip-bytes
///
/// Accept: some data
/// Returns: gzip-bytes
pub fn encode_gzip_all(input: Vec<u8>) -> Result<Vec<u8>, ZipLibError> {
    // create new encoder
    let mut encoder = Encoder::new(Vec::new()).map_err(|_| ZipLibError::CreateGzipEncoder)?;
    // write bytes
    encoder
        .write_all(input.as_slice())
        .map_err(|_| ZipLibError::IO)?;
    // finalize
    encoder
        .finish()
        .into_result()
        .map_err(|_| ZipLibError::IntoResult)
}
