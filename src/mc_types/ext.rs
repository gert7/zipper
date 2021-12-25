use super::*;
use async_trait::async_trait;
use byteorder::ReadBytesExt;
use std::{io, pin::Pin};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub trait McReadExt: io::Read {
    fn read_mc_bool(&mut self) -> io::Result<bool> {
        let is_false = self.read_i8()? == 0x00;
        Ok(!is_false)
    }

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

    fn read_mc_identifier(&mut self) -> Result<McIdentifier, McStringError>
    where
        Self: Sized,
    {
        let string = self.read_mc_string()?;
        McIdentifier::from_string(&string)
    }

    fn read_mc_uuid(&mut self) -> Result<McUUID, io::Error>
    where
        Self: Sized,
    {
        Ok(McUUID::read_from(self)?)
    }

    fn read_mc_nbt(&mut self) -> Result<nbt::Value, io::Error>
    where
        Self: Sized,
    {
        let nbt = nbt::Value::from_reader(0x0a, self);
        match nbt {
            Ok(v) => Ok(v),
            Err(e) => panic!("{}", e),
        }
    }
}

impl<R: io::Read + ?Sized> McReadExt for R {}

pub trait McWriteExt: io::Write {
    fn write_mc_bool(&mut self, value: bool) -> io::Result<usize> {
        self.write_i8(if value { 0x01 } else { 0x00 })?;
        Ok(1)
    }

    fn write_mc_byte(&mut self, value: i8) -> io::Result<usize> {
        self.write_i8(value)?;
        Ok(1)
    }

    fn write_mc_ubyte(&mut self, value: u8) -> io::Result<usize> {
        self.write_u8(value)?;
        Ok(1)
    }

    fn write_mc_short(&mut self, value: i16) -> io::Result<usize> {
        self.write_i16::<BE>(value)?;
        Ok(2)
    }

    fn write_mc_ushort(&mut self, value: u16) -> io::Result<usize> {
        self.write_u16::<BE>(value)?;
        Ok(2)
    }

    fn write_mc_int(&mut self, value: i32) -> io::Result<usize> {
        self.write_i32::<BE>(value)?;
        Ok(4)
    }

    fn write_mc_long(&mut self, value: i64) -> io::Result<usize> {
        self.write_i64::<BE>(value)?;
        Ok(8)
    }

    fn write_mc_float(&mut self, value: f32) -> io::Result<usize> {
        self.write_f32::<BE>(value)?;
        Ok(4)
    }

    fn write_mc_double(&mut self, value: f64) -> io::Result<usize> {
        self.write_f64::<BE>(value)?;
        Ok(8)
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
        McString::write_to(self, value)
    }

    fn write_mc_identifier(&mut self, value: &McIdentifier) -> io::Result<usize>
    where
        Self: Sized,
    {
        let string = value.to_string();
        McString::write_to(self, &string)
    }

    fn write_mc_uuid(&mut self, value: &McUUID) -> io::Result<usize>
    where
        Self: Sized,
    {
        McUUID::write_to(self, value)
    }

    fn write_mc_nbt(&mut self, value: &nbt::Value) -> io::Result<usize>
    where
        Self: Sized,
    {
        value.to_writer(self)?;
        Ok(value.len_bytes())
    }
}

impl<W: io::Write + ?Sized> McWriteExt for W {}

#[async_trait]
pub trait McAsyncReadExt: tokio::io::AsyncRead {
    async fn read_mc_bool(self: &mut Pin<&mut Self>) -> io::Result<bool> {
        let is_false = self.read_i8().await? == 0x00;
        Ok(!is_false)
    }

    async fn read_mc_byte(self: &mut Pin<&mut Self>) -> io::Result<i8> {
        self.read_i8().await
    }

    async fn read_mc_ubyte(self: &mut Pin<&mut Self>) -> io::Result<u8> {
        self.read_u8().await
    }

    async fn read_mc_short(self: &mut Pin<&mut Self>) -> io::Result<i16> {
        // NOTE: reads are BIG ENDIAN
        self.read_i16().await
    }

    async fn read_mc_ushort(self: &mut Pin<&mut Self>) -> io::Result<u16> {
        self.read_u16().await
    }

    async fn read_mc_int(self: &mut Pin<&mut Self>) -> io::Result<i32> {
        self.read_i32().await
    }

    async fn read_mc_long(self: &mut Pin<&mut Self>) -> io::Result<i64> {
        self.read_i64().await
    }

    async fn read_mc_float(self: &mut Pin<&mut Self>) -> io::Result<f32> {
        // No floating point in AsyncReadExt
        let fake = self.read_i32().await?;
        let f = unsafe { std::mem::transmute(fake) };
        Ok(f)
    }

    async fn read_mc_double(self: &mut Pin<&mut Self>) -> io::Result<f64> {
        let fake = self.read_i64().await?;
        let f = unsafe { std::mem::transmute(fake) };
        Ok(f)
    }

    async fn read_mc_varint(self: &mut Pin<&mut Self>) -> io::Result<i32> {
        Ok(VarInt::read_from_async(self).await?)
    }

    async fn read_mc_string(self: &mut Pin<&mut Self>) -> io::Result<String> {
        Ok(McString::read_from_async(self).await?)
    }

    async fn read_mc_identifier(self: &mut Pin<&mut Self>) -> io::Result<McIdentifier> {
        let string = McString::read_from_async(self).await?;
        Ok(McIdentifier::from_string(&string)?)
    }

    async fn read_mc_uuid(self: &mut Pin<&mut Self>) -> io::Result<McUUID> {
        Ok(McUUID::read_from_async(self).await?)
    }
}

impl<R: tokio::io::AsyncRead + ?Sized> McAsyncReadExt for R {}

#[async_trait]
pub trait McAsyncWriteExt: tokio::io::AsyncWrite {
    async fn write_mc_bool(self: &mut Pin<&mut Self>, value: bool) -> io::Result<()> {
        self.write_i8(if value { 0x01 } else { 0x00 }).await
    }

    async fn write_mc_byte(self: &mut Pin<&mut Self>, value: i8) -> io::Result<()> {
        self.write_i8(value).await
    }

    async fn write_mc_ubyte(self: &mut Pin<&mut Self>, value: u8) -> io::Result<()> {
        self.write_u8(value).await
    }

    async fn write_mc_short(self: &mut Pin<&mut Self>, value: i16) -> io::Result<()> {
        // NOTE: reads are BIG ENDIAN
        self.write_i16(value).await
    }

    async fn write_mc_ushort(self: &mut Pin<&mut Self>, value: u16) -> io::Result<()> {
        self.write_u16(value).await
    }

    async fn write_mc_int(self: &mut Pin<&mut Self>, value: i32) -> io::Result<()> {
        self.write_i32(value).await
    }

    async fn write_mc_long(self: &mut Pin<&mut Self>, value: i64) -> io::Result<()> {
        self.write_i64(value).await
    }

    async fn write_mc_float(self: &mut Pin<&mut Self>, value: f32) -> io::Result<()> {
        let fake = unsafe { std::mem::transmute(value) };
        self.write_i32(fake).await
    }

    async fn write_mc_double(self: &mut Pin<&mut Self>, value: f64) -> io::Result<()> {
        let fake = unsafe { std::mem::transmute(value) };
        self.write_i64(fake).await
    }

    async fn write_mc_varint(self: &mut Pin<&mut Self>, value: i32) -> io::Result<usize> {
        VarInt::write_to_async(self, value).await
    }

    async fn write_mc_string(self: &mut Pin<&mut Self>, value: &str) -> io::Result<usize> {
        McString::write_to_async(self, value).await
    }

    async fn write_mc_identifier(
        self: &mut Pin<&mut Self>,
        value: &McIdentifier,
    ) -> io::Result<usize> {
        let string = value.to_string();
        McString::write_to_async(self, &string).await
    }

    async fn write_mc_uuid(self: &mut Pin<&mut Self>, value: &McUUID) -> io::Result<usize> {
        McUUID::write_to_async(self, value).await
    }
}

impl<W: tokio::io::AsyncWrite + ?Sized> McAsyncWriteExt for W {}
