use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};

use super::{CompressionMiddleware, Middleware};
use crate::mc_types::ext::{McAsyncReadExt, McAsyncWriteExt};

/// Prepends the length of the data in front of the data before writing.
pub struct McNoCompression;

#[async_trait]
impl<S> Middleware<S> for McNoCompression
where
    S: AsyncWrite + AsyncRead + Send,
{
    async fn write_packet(
        &self,
        writer: &mut Pin<&mut S>,
        data: &[u8],
    ) -> tokio::io::Result<usize> {
        let mut count = 0;
        let length = data.len();
        count += writer.write_mc_varint(length as i32).await?;
        writer.write_all(data).await?;
        count += length;
        Ok(count)
    }

    async fn read_packet(&self, reader: &mut Pin<&mut S>, buf: &mut [u8]) -> tokio::io::Result<()> {
        let length = reader.read_mc_varint().await.unwrap();

        let mut handle = reader.take(length as u64);
        handle.read(buf).await?;
        Ok(())
    }
}

pub struct McZlibCompression;

#[async_trait]
impl<S> Middleware<S> for McZlibCompression
where
    S: AsyncWrite + AsyncRead + Send,
{
    async fn write_packet(
        &self,
        writer: &mut Pin<&mut S>,
        data: &[u8],
    ) -> tokio::io::Result<usize> {
        let mut count = 0;
        // TODO
        Ok(count)
    }

    async fn read_packet(&self, reader: &mut Pin<&mut S>, buf: &mut [u8]) -> tokio::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_std::task;
    use std::io::Cursor;

    #[test]
    fn test_write() {
        let input = vec![0x01 as u8, 0x11, 0x22, 0x33];

        let output = Vec::new();
        let mut output = Cursor::new(output);

        let mw = McNoCompression;

        task::block_on(async move {
            mw.write_packet(&mut Pin::new(&mut output), &input)
                .await
                .ok();
            let vecnow = output.get_ref();
            assert!(vecnow[0] == 0x04);
            assert!(vecnow[1] == 0x01);
            assert!(vecnow[2] == 0x11);
            assert!(vecnow[3] == 0x22);
            assert!(vecnow[4] == 0x33);
        });
    }
}
