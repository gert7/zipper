pub mod ext;

use byteorder::{BigEndian as BE, ReadBytesExt, WriteBytesExt};
use num_traits::PrimInt;
use std::{io::{self, Cursor, Read, Write}, str::{self, Utf8Error}};

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

impl From<io::Error> for VarIntError {
    fn from(e: io::Error) -> Self {
        VarIntError::Io(e)
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
}

pub struct McString(pub String);

#[derive(Debug)]
pub enum McStringError {
    VarIntError(VarIntError),
    Io(io::Error),
    Utf8(Utf8Error),
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

    pub fn write_to(string: &str, writer: &mut impl Write) -> std::io::Result<usize> {
        let length = string.len();
        let mut count = 0;
        count += VarInt::write_to(writer, length as i32)?;
        count += writer.write(string.as_bytes())?;
        Ok(count)
    }
}