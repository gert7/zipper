pub enum SocketMode {
    Handshaking,
    Status,
    Login,
    Play,
}

#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum HandshakingPacket {
    Handshaking = 0x00,
}

#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum LoginPacket {
    LoginStart = 0x00,
    EncryptionResponse = 0x01,
    LoginPluginResponse = 0x02,
}

#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(u8)]
pub enum LoginPacketOut {
    Disconnect = 0x00,
    EncryptionRequest = 0x01,
    LoginSuccess = 0x02,
}

#[derive(Debug, PartialEq, Eq, FromPrimitive, ToPrimitive)]
#[repr(i8)]
pub enum GameMode {
    NoPreviousGameMode = -1,
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
}
