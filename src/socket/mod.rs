mod compression;
mod encryption;

use std::pin::Pin;

use async_trait::async_trait;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

#[async_trait]
pub trait Middleware<S: AsyncRead + AsyncWrite> {
    async fn write_packet(&self, writer: &mut Pin<&mut S>, data: &[u8]) -> tokio::io::Result<usize>;

    async fn read_packet(&self, reader: &mut Pin<&mut S>, buf: &mut [u8]) -> tokio::io::Result<()>;
}

type CompressionMiddleware<S> = dyn Middleware<S>;

type EncryptionMiddleware<S> = dyn Middleware<S>;

pub struct McSocket<'a, S: AsyncWrite + AsyncRead, C: Middleware<S>, E: Middleware<S>> {
    socket: &'a S,
    compressor: C,
    encryptor: E,
}

impl<'a, S: AsyncWrite + AsyncRead, C: Middleware<S>, E: Middleware<S>> McSocket<'a, S, C, E> {
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