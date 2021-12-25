#[macro_use]
extern crate num_derive;
extern crate lazy_static;

mod mc_types;
mod packet;
mod socket;
mod world;

use lazy_static::lazy_static;
use mc_types::McIdentifier;
use nbt::Blob;
use rand::prelude::*;
use std::{
    fs::{read_to_string, File},
    io::{self, Cursor, Read, Write},
    pin::Pin,
};

// use std::net::{TcpListener, TcpStream};
use tokio::{
    io::{AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

use crate::{
    mc_types::ext::{McAsyncReadExt, McAsyncWriteExt, McReadExt, McWriteExt},
    packet::{LoginPacketOut, SocketMode},
    socket::{compression::McNoCompression, passthrough::McPassthrough, McSocket},
};
use packet::{GameMode, HandshakingPacket, LoginPacket};

const ENCRYPTION_ENABLED: bool = false;

lazy_static! {
    static ref PUBLIC_KEY: Vec<u8> = {
        let mut f = File::open("public.der").unwrap();
        let mut data = Vec::new();
        f.read_to_end(&mut data).ok();
        data
    };
    static ref PRIVATE_KEY: String = read_to_string("private.pem").unwrap();
}

fn buffer_cursor() -> Cursor<Vec<u8>> {
    let buf: Vec<u8> = Vec::new();
    Cursor::new(buf)
}

fn prepare_encryption_request(buf: &mut impl Write) -> io::Result<()> {
    // Server ID
    buf.write_mc_string("\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0")?;
    // Public Key Length
    buf.write_mc_varint(PUBLIC_KEY.len() as i32)?;
    // Public Key
    buf.write(PUBLIC_KEY.as_slice())?;
    // Verify Token Length
    buf.write_mc_varint(4)?;
    // Verify Token
    let mut vtoken = Vec::new();
    for _ in 0..4 {
        vtoken.push(random::<u8>());
    }
    buf.write(&vtoken)?;
    println!(
        "Sent encryption request... Public key Length {}",
        PUBLIC_KEY.as_slice().len()
    );
    Ok(())
}

fn prepare_login_success(buf: &mut impl Write) -> io::Result<usize> {
    let mut count = 0;
    let uuid = mc_types::McUUID {
        most: 64,
        least: 64,
    };
    count += buf.write_mc_uuid(&uuid)?;
    count += buf.write_mc_string("Shifty")?;
    Ok(count)
}

fn prepare_join_game(buf: &mut impl Write) -> io::Result<usize> {
    let mut f = File::create("nbt.txt").unwrap();

    let mut count = 0;
    // player Entity ID
    count += buf.write_mc_int(24)?;
    count += buf.write_mc_bool(false)?;
    // current gamemode
    count += buf.write_mc_ubyte(num::ToPrimitive::to_u8(&GameMode::Survival).unwrap())?;
    count += buf.write_mc_byte(num::ToPrimitive::to_i8(&GameMode::Survival).unwrap())?;
    count += buf.write_mc_varint(1)?; // world count
    let world_id = McIdentifier {
        namespace: Some("zipperland".to_owned()),
        name: "steveland".to_owned(),
    };
    count += buf.write_mc_identifier(&world_id)?; // world names
    count += buf.write_mc_nbt(&world::dimensions::DEFAULT_DIMENSION_CODEC)?;
    f.write_mc_nbt(&world::dimensions::DEFAULT_DIMENSION_CODEC)?;
    count += buf.write_mc_nbt(&world::dimensions::DEFAULT_DIMENSION_TYPE)?;
    f.write_mc_nbt(&world::dimensions::DEFAULT_DIMENSION_TYPE)?;
    count += buf.write_mc_identifier(&world_id)?; // world name
    count += buf.write_mc_long(56717237888581)?; // hashed seed
    count += buf.write_mc_varint(32)?; // max players
    count += buf.write_mc_varint(16)?; // view distance
    count += buf.write_mc_varint(16)?; // simulation distance
    count += buf.write_mc_bool(false)?; // reduced debug info
    count += buf.write_mc_bool(true)?; // respawn screen
    count += buf.write_mc_bool(true)?; // debug world
    count += buf.write_mc_bool(false)?; // superflat
    Ok(count)
}

async fn send_packet_uncompressed<S>(
    pid: u8,
    stream: &mut Pin<&mut S>,
    buf: &[u8],
) -> io::Result<()>
where
    S: AsyncWrite + Send,
{
    let mut f = File::create("hello.txt").unwrap();

    stream.write_mc_varint((buf.len() + 1) as i32).await?;
    f.write_mc_varint((buf.len() + 1) as i32)?;

    println!("Buffer length {}", buf.len());

    stream.write_mc_varint(pid as i32).await?;
    f.write_mc_varint(pid as i32)?;

    stream.write(buf).await?;
    f.write(buf)?;
    Ok(())
}

async fn handle_client(mut stream: TcpStream) -> io::Result<()> {
    let stream_m = &mut stream;
    let mut stream_m = Pin::new(stream_m);
    // let socket = McSocket::new(stream_m, McNoCompression, McPassthrough);
    let mut mode = SocketMode::Handshaking;

    println!(
        "Received a stream {}",
        stream_m.peer_addr().unwrap().ip().to_string()
    );

    loop {
        let length = stream_m.read_mc_varint().await?;
        let packet_id_u8 = stream_m.read_mc_varint().await? as u8;
        match mode {
            SocketMode::Handshaking => {
                let packet_id = num::FromPrimitive::from_u8(packet_id_u8);
                println!("Length {}, ID {}", length, packet_id_u8);
                match packet_id {
                    Some(HandshakingPacket::Handshaking) => {
                        let protocol_version = stream_m.read_mc_varint().await?;
                        let addr = stream_m.read_mc_string().await?;
                        let port = stream_m.read_mc_ushort().await?;
                        let next_state = stream_m.read_mc_varint().await?;

                        println!(
                            "Version {}, addr {}, port {}, next state {}",
                            protocol_version, addr, port, next_state
                        );
                        mode = SocketMode::Login;
                    }
                    None => println!("Unknown handshake packet id {}", packet_id_u8),
                };
            }
            SocketMode::Status => todo!(),
            SocketMode::Login => {
                let packet_id = num::FromPrimitive::from_u8(packet_id_u8);
                println!("Length {}, ID {}", length, packet_id_u8);
                match packet_id {
                    Some(LoginPacket::LoginStart) => {
                        let username = stream_m.read_mc_string().await;
                        match username {
                            Ok(v) => println!("Username {}", v),
                            Err(e) => {
                                println!("Username is invalid UTF-8: {:?}", e);
                                return Err(io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    "Username is invalid",
                                ));
                            }
                        }

                        // LOGIN

                        let mut cur = buffer_cursor();
                        if ENCRYPTION_ENABLED {
                            prepare_encryption_request(&mut cur).ok();
                        } else {
                            prepare_login_success(&mut cur).ok();
                        }
                        let cur = cur.get_ref();
                        let pid = LoginPacketOut::EncryptionRequest;
                        //send_packet_uncompressed(num::ToPrimitive::to_u8(&pid).unwrap(),
                        send_packet_uncompressed(0x02, &mut stream_m, cur).await?;

                        // JOIN GAME

                        let mut cur = buffer_cursor();
                        prepare_join_game(&mut cur).ok();
                        send_packet_uncompressed(0x26, &mut stream_m, cur.get_ref()).await?;
                    }
                    Some(LoginPacket::EncryptionResponse) => {}
                    Some(LoginPacket::LoginPluginResponse) => {}
                    None => println!("Unknown Login packet id {}", packet_id_u8),
                }
            }
            SocketMode::Play => {
                println!("Play packet received");
            }
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565").await?;

    println!("Continued");

    loop {
        let (socket, _) = listener.accept().await?;
        handle_client(socket).await;
    }

    // for stream in listener.incoming() {
    //     let stream = stream.unwrap();
    //     tokio::spawn(async move {
    //         handle_client(stream);
    //     });
    // }

    Ok(())
}
