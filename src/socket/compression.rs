use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use async_trait::async_trait;

use super::{CompressionMiddleware, Middleware};
use crate::mc_types::ext::{McAsyncReadExt, McAsyncWriteExt};

pub struct McNoCompression;

#[async_trait]
impl<W, R> Middleware<W, R> for McNoCompression
where
    W: AsyncWrite + Send,
    R: AsyncRead + Send,
{
    async fn write_packet(writer: &mut Pin<&mut W>, data: &[u8]) -> tokio::io::Result<usize> {
        let mut count = 0;
        let length = data.len();
        count += writer.write_mc_varint(length as i32).await?;
        writer.write_all(data).await?;
        count += length;
        Ok(count)
    }

    async fn read_packet(reader: &mut Pin<&mut R>, buf: &mut [u8]) -> tokio::io::Result<()> {
        let length = reader.read_mc_varint().await.unwrap();

        let mut handle = reader.take(length as u64);
        handle.read(buf).await?;
        Ok(())
    }
}
