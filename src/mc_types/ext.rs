use super::*;
use async_trait::async_trait;
use byteorder::ReadBytesExt;
use std::{io, pin::Pin};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub trait McReadExt: io::Read {
    fn read_mc_byte(&mut self) -> io::Result<i8> {
        self.read_i8()
    }

    fn read_mc_ubyte(&mut self) -> io::Result<u8> {
        self.read_u8()
    }

    fn read_mc_short(&mut self) -> io::Result<i16> {
        Ok(self.read_i16::<BE>()?)
    }

    fn read_mc_ushort(&mut self) -> io::Result<u16> {
        Ok(self.read_u16::<BE>()?)
    }

    fn read_mc_int(&mut self) -> io::Result<i32> {
        Ok(self.read_i32::<BE>()?)
    }

    fn read_mc_long(&mut self) -> io::Result<i64> {
        Ok(self.read_i64::<BE>()?)
    }

    fn read_mc_float(&mut self) -> io::Result<f32> {
        Ok(self.read_f32::<BE>()?)
    }

    fn read_mc_double(&mut self) -> io::Result<f64> {
        Ok(self.read_f64::<BE>()?)
    }

    fn read_mc_varint(&mut self) -> Result<i32, VarIntError>
    where
        Self: Sized,
    {
        Ok(VarInt::read_from(self)?)
    }

    fn read_mc_string(&mut self) -> Result<String, McStringError>
    where
        Self: Sized,
    {
        Ok(McString::read_from(self)?)
    }
}

impl<R: io::Read + ?Sized> McReadExt for R {}

pub trait McWriteExt: io::Write {
    fn write_mc_byte(&mut self, value: i8) -> io::Result<()> {
        self.write_i8(value)
    }

    fn write_mc_ubyte(&mut self, value: u8) -> io::Result<()> {
        self.write_u8(value)
    }

    fn write_mc_short(&mut self, value: i16) -> io::Result<()> {
        self.write_i16::<BE>(value)
    }

    fn write_mc_ushort(&mut self, value: u16) -> io::Result<()> {
        self.write_u16::<BE>(value)
    }

    fn write_mc_int(&mut self, value: i32) -> io::Result<()> {
        self.write_i32::<BE>(value)
    }

    fn write_mc_long(&mut self, value: i64) -> io::Result<()> {
        self.write_i64::<BE>(value)
    }

    fn write_mc_float(&mut self, value: f32) -> io::Result<()> {
        self.write_f32::<BE>(value)
    }

    fn write_mc_double(&mut self, value: f64) -> io::Result<()> {
        self.write_f64::<BE>(value)
    }

    fn write_mc_varint(&mut self, value: i32) -> io::Result<usize>
    where
        Self: Sized,
    {
        VarInt::write_to(self, value)
    }

    fn write_mc_string(&mut self, value: &str) -> io::Result<usize>
    where
        Self: Sized,
    {
        McString::write_to(value, self)
    }
}

impl<W: io::Write + ?Sized> McWriteExt for W {}

#[async_trait]
pub trait McAsyncReadExt: tokio::io::AsyncRead {
    async fn read_mc_byte(self: &mut Pin<&mut Self>) -> tokio::io::Result<i8> {
        self.read_i8().await
    }

    async fn read_mc_ubyte(self: &mut Pin<&mut Self>) -> tokio::io::Result<u8> {
        self.read_u8().await
    }

    async fn read_mc_short(self: &mut Pin<&mut Self>) -> tokio::io::Result<i16> {
        // NOTE: reads are BIG ENDIAN
        self.read_i16().await
    }

    async fn read_mc_ushort(self: &mut Pin<&mut Self>) -> tokio::io::Result<u16> {
        self.read_u16().await
    }

    async fn read_mc_int(self: &mut Pin<&mut Self>) -> tokio::io::Result<i32> {
        self.read_i32().await
    }

    async fn read_mc_long(self: &mut Pin<&mut Self>) -> tokio::io::Result<i64> {
        self.read_i64().await
    }

    async fn read_mc_float(self: &mut Pin<&mut Self>) -> tokio::io::Result<f32> {
        // No floating point in AsyncReadExt
        let fake = self.read_i32().await?;
        let f = unsafe { std::mem::transmute(fake) };
        Ok(f)
    }

    async fn read_mc_double(self: &mut Pin<&mut Self>) -> tokio::io::Result<f64> {
        let fake = self.read_i64().await?;
        let f = unsafe { std::mem::transmute(fake) };
        Ok(f)
    }

    async fn read_mc_varint(self: &mut Pin<&mut Self>) -> tokio::io::Result<i32> {
        Ok(VarInt::read_from_async(self.as_mut()).await.unwrap())
    }
}

impl<R: tokio::io::AsyncRead + ?Sized> McAsyncReadExt for R {}

#[async_trait]
pub trait McAsyncWriteExt: tokio::io::AsyncWrite {
    async fn write_mc_byte(self: &mut Pin<&mut Self>, value: i8) -> tokio::io::Result<()> {
        self.write_i8(value).await
    }

    async fn write_mc_ubyte(self: &mut Pin<&mut Self>, value: u8) -> tokio::io::Result<()> {
        self.write_u8(value).await
    }

    async fn write_mc_short(self: &mut Pin<&mut Self>, value: i16) -> tokio::io::Result<()> {
        // NOTE: reads are BIG ENDIAN
        self.write_i16(value).await
    }

    async fn write_mc_ushort(self: &mut Pin<&mut Self>, value: u16) -> tokio::io::Result<()> {
        self.write_u16(value).await
    }

    async fn write_mc_int(self: &mut Pin<&mut Self>, value: i32) -> tokio::io::Result<()> {
        self.write_i32(value).await
    }

    async fn write_mc_long(self: &mut Pin<&mut Self>, value: i64) -> tokio::io::Result<()> {
        self.write_i64(value).await
    }

    async fn write_mc_float(self: &mut Pin<&mut Self>, value: f32) -> tokio::io::Result<()> {
        let fake = unsafe { std::mem::transmute(value) };
        self.write_i32(fake).await
    }

    async fn write_mc_double(self: &mut Pin<&mut Self>, value: f64) -> tokio::io::Result<()> {
        let fake = unsafe { std::mem::transmute(value) };
        self.write_i64(fake).await
    }

    async fn write_mc_varint(self: &mut Pin<&mut Self>, value: i32) -> tokio::io::Result<usize> {
        VarInt::write_to_async(self.as_mut(), value).await
    }
}

impl<W: tokio::io::AsyncWrite + ?Sized> McAsyncWriteExt for W {}
