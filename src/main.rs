#[macro_use]
extern crate num_derive;
extern crate lazy_static;

mod mc_types;
mod packet;
mod socket;

use lazy_static::lazy_static;
use rand::prelude::*;
use std::{
    fs::{read_to_string, File},
    io::{Cursor, Read, Result, Write},
};

use std::net::{TcpListener, TcpStream};

use crate::{
    mc_types::ext::{McReadExt, McWriteExt},
    packet::{LoginPacketOut, SocketMode},
};
use packet::{HandshakingPacket, LoginPacket};

lazy_static! {
    static ref PUBLIC_KEY: Vec<u8> = {
        let mut f = File::open("public.der").unwrap();
        let mut data = Vec::new();
        f.read_to_end(&mut data).ok();
        data
    };
    static ref PRIVATE_KEY: String = { read_to_string("private.pem").unwrap() };
}

fn prepare_encryption_request(buf: &mut impl Write) -> Result<()> {
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

fn send_packet_uncompressed(pid: u8, stream: &mut impl Write, buf: &[u8]) -> Result<()> {
    let mut f = File::create("hello.txt").unwrap();

    stream.write_mc_varint((buf.len() + 1) as i32)?;
    f.write_mc_varint((buf.len() + 1) as i32)?;

    println!("Buffer length {}", buf.len());

    stream.write_mc_varint(pid as i32)?;
    f.write_mc_varint(pid as i32)?;

    stream.write(buf)?;
    f.write(buf)?;
    Ok(())
}

fn handle_client(mut stream: TcpStream) {
    let stream_m = &mut stream;
    let mut mode = SocketMode::Handshaking;

    println!(
        "Received a stream {}",
        stream_m.peer_addr().unwrap().ip().to_string()
    );

    loop {
        let length = stream_m.read_mc_varint().unwrap();
        let packet_id_u8 = stream_m.read_mc_varint().unwrap() as u8;
        match mode {
            SocketMode::Handshaking => {
                let packet_id = num::FromPrimitive::from_u8(packet_id_u8);
                println!("Length {}, ID {}", length, packet_id_u8);
                match packet_id {
                    Some(HandshakingPacket::Handshaking) => {
                        let protocol_version = stream_m.read_mc_varint().unwrap();
                        let addr = stream_m.read_mc_string().unwrap();
                        let port = stream_m.read_mc_ushort().unwrap();
                        let next_state = stream_m.read_mc_varint().unwrap();

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
                        let username = stream_m.read_mc_string();
                        match username {
                            Ok(v) => println!("Username {}", v),
                            Err(e) => {
                                println!("Username is invalid UTF-8: {:?}", e);
                                return ();
                            }
                        }

                        let buf = Vec::new();
                        let mut cur = Cursor::new(buf);
                        prepare_encryption_request(&mut cur).ok();
                        let cur = cur.get_ref();
                        let pid = LoginPacketOut::EncryptionRequest;
                        //send_packet_uncompressed(num::ToPrimitive::to_u8(&pid).unwrap(),
                        send_packet_uncompressed(0x01, stream_m, cur).unwrap();
                    }
                    Some(LoginPacket::EncryptionResponse) => {}
                    Some(LoginPacket::LoginPluginResponse) => {}
                    None => println!("Unknown Login packet id {}", packet_id_u8),
                }
            }
            SocketMode::Play => todo!(),
        }
    }
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:25565")?;

    println!("Continued");

    // loop {
    //     let (socket, _) = listener.accept().await?;
    //     handle_client(socket).await;
    // }

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        tokio::spawn(async move {
            handle_client(stream);
        });
    }

    Ok(())
}
