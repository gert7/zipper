pub mod ext;

use byteorder::{BigEndian as BE, ReadBytesExt, WriteBytesExt};
use num_traits::PrimInt;
use std::{
    borrow::BorrowMut,
    io::{self, Cursor, ErrorKind, Read, Write},
    ops::Add,
    pin::Pin,
    str::{self, Utf8Error},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

fn i32_to_u32_reinterpret(n: i32) -> u32 {
    unsafe { std::mem::transmute(n) }
}

fn u32_to_i32_reinterpret(n: u32) -> i32 {
    unsafe { std::mem::transmute(n) }
}

pub trait MinecraftType<T> {
    fn write_to(&self, writer: &mut impl Write);

    fn read_from(reader: &mut impl Read) -> T;
}

type Boolean = i8;

type Byte = i8;

type UnsignedByte = u8;

pub struct Short(pub i16);

impl MinecraftType<i16> for Short {
    fn write_to(&self, wtr: &mut impl Write) {
        wtr.write_i16::<BE>(self.0);
    }

    fn read_from(reader: &mut impl Read) -> i16 {
        let number = reader.read_i16::<BE>().unwrap();
        number
    }
}

pub struct UnsignedShort(pub u16);

impl MinecraftType<u16> for UnsignedShort {
    fn write_to(&self, wtr: &mut impl Write) {
        wtr.write_u16::<BE>(self.0);
    }

    fn read_from(reader: &mut impl Read) -> u16 {
        let number = reader.read_u16::<BE>().unwrap();
        number
    }
}

pub struct Int(pub i32);

impl MinecraftType<i32> for Int {
    fn write_to(&self, wtr: &mut impl Write) {
        wtr.write_i32::<BE>(self.0);
    }

    fn read_from(reader: &mut impl Read) -> i32 {
        let number = reader.read_i32::<BE>().unwrap();
        number
    }
}

pub struct Long(i64);

impl MinecraftType<i64> for Long {
    fn write_to(&self, wtr: &mut impl Write) {
        wtr.write_i64::<BE>(self.0);
    }

    fn read_from(reader: &mut impl Read) -> i64 {
        let number = reader.read_i64::<BE>().unwrap();
        number
    }
}

pub struct Float(pub f32);

impl MinecraftType<f32> for Float {
    fn write_to(&self, wtr: &mut impl Write) {
        wtr.write_f32::<BE>(self.0);
    }

    fn read_from(reader: &mut impl Read) -> f32 {
        let number = reader.read_f32::<BE>().unwrap();
        number
    }
}

pub struct Double(pub f64);

impl MinecraftType<f64> for Double {
    fn write_to(&self, wtr: &mut impl Write) {
        wtr.write_f64::<BE>(self.0);
    }

    fn read_from(reader: &mut impl Read) -> f64 {
        let number = reader.read_f64::<BE>().unwrap();
        number
    }
}

pub struct VarInt(pub i32);

#[derive(Debug)]
pub enum VarIntError {
    VarIntTooLong,
    Io(io::Error),
}

impl VarIntError {
    fn to_string(&self) -> &str {
        match self {
            VarIntError::VarIntTooLong => "VarInt is too long",
            VarIntError::Io(_) => "IO error found!",
        }
    }
}

impl From<io::Error> for VarIntError {
    fn from(e: io::Error) -> Self {
        VarIntError::Io(e)
    }
}

impl From<VarIntError> for io::Error {
    fn from(e: VarIntError) -> Self {
        match e {
            VarIntError::VarIntTooLong => io::Error::new(ErrorKind::Other, "VarInt too long"),
            VarIntError::Io(v) => v,
        }
    }
}

impl VarInt {
    pub fn read_from(reader: &mut impl Read) -> Result<i32, VarIntError> {
        let mut decoded_int: i32 = 0;
        let mut offset = 0;

        loop {
            let current_byte = reader.read_u8()?;
            decoded_int |= ((current_byte as i32) & 0b01111111) << offset;

            if offset >= 35 {
                return Err(VarIntError::VarIntTooLong);
            }

            offset += 7;
            if (current_byte & 0b10000000) == 0 {
                break;
            }
        }
        Ok(decoded_int)
    }

    pub fn write_to(writer: &mut impl Write, value: i32) -> std::io::Result<usize> {
        let mut value = i32_to_u32_reinterpret(value);
        let mut count = 0;
        loop {
            let mut current_byte = (value & 0b01111111) as u8;

            value = value.unsigned_shr(7);
            if value != 0 {
                current_byte |= 0b10000000;
            }

            writer.write_u8(current_byte)?;
            count += 1;
            if value == 0 {
                break;
            }
        }
        Ok(count)
    }

    pub async fn read_from_async<R: AsyncRead + ?Sized>(
        reader: &mut Pin<&mut R>,
    ) -> Result<i32, VarIntError> {
        let mut decoded_int: i32 = 0;
        let mut offset = 0;

        loop {
            let current_byte = reader.read_u8().await?;
            decoded_int |= ((current_byte as i32) & 0b01111111) << offset;

            if offset >= 35 {
                return Err(VarIntError::VarIntTooLong);
            }

            offset += 7;
            if (current_byte & 0b10000000) == 0 {
                break;
            }
        }
        Ok(decoded_int)
    }

    pub async fn write_to_async<W: AsyncWrite + ?Sized>(
        writer: &mut Pin<&mut W>,
        value: i32,
    ) -> tokio::io::Result<usize> {
        let mut value = i32_to_u32_reinterpret(value);
        let mut count = 0;
        loop {
            let mut current_byte = (value & 0b01111111) as u8;

            value = value.unsigned_shr(7);
            if value != 0 {
                current_byte |= 0b10000000;
            }

            writer.write_u8(current_byte).await?;
            count += 1;
            if value == 0 {
                break;
            }
        }
        Ok(count)
    }
}

pub struct McString(pub String);

#[derive(Debug)]
pub enum McStringError {
    VarIntError(VarIntError),
    Io(io::Error),
    Utf8(Utf8Error),
    LengthMismatch,
}

impl From<VarIntError> for McStringError {
    fn from(vie: VarIntError) -> Self {
        McStringError::VarIntError(vie)
    }
}

impl From<io::Error> for McStringError {
    fn from(e: io::Error) -> Self {
        McStringError::Io(e)
    }
}

impl From<McStringError> for io::Error {
    fn from(e: McStringError) -> Self {
        match e {
            McStringError::VarIntError(_) => io::Error::new(ErrorKind::Other, "VarInt Error"),
            McStringError::Io(v) => v,
            McStringError::Utf8(_) => io::Error::new(ErrorKind::InvalidData, "UTF-8 coding error"),
            McStringError::LengthMismatch => {
                io::Error::new(ErrorKind::InvalidData, "String length mismatch")
            }
        }
    }
}

impl McString {
    pub fn read_from(reader: &mut impl Read) -> Result<String, McStringError> {
        let length = VarInt::read_from(reader)? as usize;
        let mut buffer: Vec<u8> = vec![0; length];
        reader.read_exact(&mut buffer)?;

        let s = match str::from_utf8(&buffer) {
            Ok(v) => v,
            Err(u8err) => return Err(McStringError::Utf8(u8err)),
        };
        Ok(s.to_owned())
    }

    pub fn write_to(writer: &mut impl Write, string: &str) -> io::Result<usize> {
        let length = string.len();
        let mut count = 0;
        count += VarInt::write_to(writer, length as i32)?;
        count += writer.write(string.as_bytes())?;
        Ok(count)
    }

    pub async fn read_from_async<R: AsyncRead + ?Sized>(
        reader: &mut Pin<&mut R>,
    ) -> Result<String, McStringError> {
        let length = VarInt::read_from_async(reader).await? as usize;
        let mut buffer: Vec<u8> = vec![0; length];
        let actual_len = reader.read_exact(buffer.as_mut()).await?;

        if actual_len < length {
            return Err(McStringError::LengthMismatch);
        }

        let s = match str::from_utf8(&buffer) {
            Ok(v) => v,
            Err(u8err) => return Err(McStringError::Utf8(u8err)),
        };
        Ok(s.to_owned())
    }

    pub async fn write_to_async<W: AsyncWrite + ?Sized>(
        writer: &mut Pin<&mut W>,
        string: &str,
    ) -> io::Result<usize> {
        let length = string.len();
        let mut count = 0;
        count += VarInt::write_to_async(writer, length as i32).await?;
        count += writer.write(string.as_bytes()).await?;
        Ok(count)
    }
}

pub struct McIdentifier {
    pub namespace: Option<String>,
    pub name: String,
}

impl McIdentifier {
    fn from_string(string: &str) -> Result<McIdentifier, McStringError> {
        let mut split = string.split(':');
        let first = split.next().ok_or(McStringError::LengthMismatch)?;
        let second = split.next();
        Ok(McIdentifier {
            namespace: if second.is_some() {
                Some(first.to_owned())
            } else {
                None
            },
            name: second.unwrap_or(first).to_owned(),
        })
    }

    fn to_string(&self) -> String {
        let mut s = String::new();
        if self.namespace.is_some() {
            s.push_str(self.namespace.as_ref().unwrap().as_str());
        } else {
            s.push_str("minecraft");
        }
        s.push(':');
        s.push_str(&self.name);
        s
    }
}

pub struct McUUID {
    pub most: u64,
    pub least: u64,
}

impl McUUID {
    pub fn read_from(reader: &mut impl Read) -> io::Result<McUUID> {
        let most = reader.read_u64::<BE>()?;
        let least = reader.read_u64::<BE>()?;
        let m = McUUID { most, least };
        Ok(m)
    }

    pub fn write_to(writer: &mut impl Write, uuid: &McUUID) -> io::Result<usize> {
        writer.write_u64::<BE>(uuid.most)?;
        writer.write_u64::<BE>(uuid.least)?;
        Ok(16)
    }

    pub async fn read_from_async<R: AsyncRead + ?Sized>(
        reader: &mut Pin<&mut R>,
    ) -> io::Result<McUUID> {
        let most = reader.read_u64().await?;
        let least = reader.read_u64().await?;
        let m = McUUID { most, least };
        Ok(m)
    }

    pub async fn write_to_async<W: AsyncWrite + ?Sized>(
        writer: &mut Pin<&mut W>,
        uuid: &McUUID,
    ) -> io::Result<usize> {
        writer.write_u64(uuid.most).await.ok();
        writer.write_u64(uuid.least).await.ok();
        Ok(16)
    }
}
