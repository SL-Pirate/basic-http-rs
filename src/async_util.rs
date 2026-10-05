use crate::server::response_handler::STREAM_BODY_BUFFER_SIZE;
use std::io;
use tokio::io::{AsyncRead, AsyncWriteExt, DuplexStream, duplex};

pub async fn to_duplex<R: AsyncRead + Unpin + Send + 'static>(
    mut input: R,
) -> io::Result<DuplexStream> {
    let (mut tx, rx) = duplex(STREAM_BODY_BUFFER_SIZE); // max bytes buffered between the two ends

    tokio::spawn(async move {
        if let Err(e) = tokio::io::copy(&mut input, &mut tx).await {
            eprintln!("copy failed: {e}");
        }
        let _ = tx.shutdown().await; // signals EOF to the reading end
    });

    Ok(rx)
}
