#[allow(dead_code)]
/// RakNetPong is a model of data returned by raknet Unconnected Ping
///
/// This data includes game_edition to server_unique_id in most implementations.
///
/// Depending on the server software gamemode_mode and port information might not be included
/// which is a Option is wrapped around its type.
///
#[derive(Debug)]
pub struct RakNetPong {
    pub game_edition:      String,
    pub motd:              Vec<String>,
    pub protocol_version:  usize,
    pub game_version:      String,
    pub player_count:      usize,
    pub max_player_count:  usize,
    pub server_uid:        String,
    pub game_mode:         Option<String>,
    pub game_mode_integer: Option<usize>,
    pub port:              Option<u16>,
    pub port_v6:           Option<u16>
}