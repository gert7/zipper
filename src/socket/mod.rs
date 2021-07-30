mod compression;
mod encryption;

use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[async_trait]
pub trait Middleware<W: AsyncWrite, R: AsyncRead> {
    async fn write_packet(writer: &mut Pin<&mut W>, data: &[u8]) -> tokio::io::Result<usize>;

    async fn read_packet(reader: &mut Pin<&mut R>, buf: &mut [u8]) -> tokio::io::Result<()>;
}

type CompressionMiddleware<W, R> = dyn Middleware<W, R>;

type EncryptionMiddleware<W, R> = dyn Middleware<W, R>;

pub struct McSocket<'a, S: AsyncWrite + AsyncRead, C: Middleware<S, S>, E: Middleware<S, S>> {
    socket: &'a S,
    compressor: C,
    encryptor: E,
}

impl<'a, S: AsyncWrite + AsyncRead, C: Middleware<S, S>, E: Middleware<S, S>> McSocket<'a, S, C, E> {
    pub fn new(socket: &'a mut S, compressor: C, encryptor: E) -> McSocket<'a, S, C, E> {
        McSocket {
            socket,
            compressor,
            encryptor,
        }
    }

    pub fn write_mc_packet(pid: i32, packet: &[u8]) {

    }

    pub fn read_mc_packet() {

    }
}