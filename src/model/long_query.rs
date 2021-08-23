#[allow(dead_code)]
/// LongQuery is a model of data returned by a STAT request
///
/// This data includes game_edition to server_unique_id in most implementations.
///
/// Depending on the server software gamemode_mode and port information might not be included
/// which is a Option is wrapped around its type.
///
#[derive(Debug)]
pub struct LongQuery {
    pub server_software: String,
    pub plugins: String,
    pub version: String,
    pub whitelist: String,
    pub players: Vec<String>,
    pub player_count: usize,
    pub max_players: usize,
    pub game_name: String,
    pub game_mode: String,
    pub map_name: String,
    pub host_name: String,
    pub host_ip: String,
    pub host_port: u16
}