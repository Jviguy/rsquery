#[allow(dead_code)]
#[derive(Debug)]
pub struct LongQuery {
    server_software: String,
    plugins: String,
    version: String,
    whitelist: String,
    players: Vec<String>,
    player_count: String,
    max_players: String,
    game_name: String,
    game_mode: String,
    map_name: String,
    host_name: String,
    host_ip: String,
    host_port: String
}