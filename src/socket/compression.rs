use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use async_trait::async_trait;

use super::{CompressionMiddleware, Middleware};
use crate::mc_types::ext::{McAsyncReadExt, McAsyncWriteExt};

pub struct McNoCompression;

#[async_trait]
impl<S> Middleware<S> for McNoCompression
where
    S: AsyncWrite + AsyncRead + Send,
{
    async fn write_packet(&self, writer: &mut Pin<&mut S>, data: &[u8]) -> tokio::io::Result<usize> {
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use tokio::{io::BufWriter, runtime::Handle, task::block_in_place};
    use async_std::task;

    use super::*;

    #[test]
    fn test_write() {
        let input = vec![0x01 as u8, 0x11, 0x22, 0x33];
        
        let output = Vec::new();
        let output = Cursor::new(output);
        let mut output = BufWriter::new(output);

        let mw = McNoCompression;

        task::block_on(async move {
            mw.write_packet(&mut Pin::new(&mut output), &input).await.ok();
            let vecnow = output.buffer();
            assert!(vecnow[0] == 0x04);
            assert!(vecnow[1] == 0x01);
            assert!(vecnow[2] == 0x11);
            assert!(vecnow[3] == 0x22);
            assert!(vecnow[4] == 0x33);
        });

    }
}