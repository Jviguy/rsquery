pub const MAGIC: u16 = 0xFEFD;
pub const STAT: u8 = 0x00;
pub const HANDSHAKE: u8 = 0x09;
//Its hacky but it works.
pub const PLAYER_KEY: [u8; 11] = [0x00, 0x01, 'p' as u8, 'l' as u8, 'a' as u8, 'y' as u8, 'e' as u8, 'r' as u8, '_' as u8, 0x00, 0x00];