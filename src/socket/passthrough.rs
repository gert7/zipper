use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

use super::Middleware;

/** Reads and writes without modification
 */
pub struct McPassthrough;

#[async_trait]
impl<S> Middleware<S> for McPassthrough
where
    S: AsyncWrite + AsyncRead + Send,
{
    async fn write_packet(
        &self,
        writer: &mut Pin<&mut S>,
        data: &[u8],
    ) -> tokio::io::Result<usize> {
        let len = writer.write(&data).await?;
        Ok(len)
    }

    async fn read_packet(&self, reader: &mut Pin<&mut S>, buf: &mut [u8]) -> tokio::io::Result<()> {
        reader.read(buf).await?;
        Ok(())
    }
}
