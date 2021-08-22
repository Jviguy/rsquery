#[allow(dead_code)]
#[derive(Debug)]
pub struct ShortQuery {
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