#[allow(dead_code)]

/// ShortQuery is a model of data returned by GS3 BASIC STAT
///
/// This data includes game_type (SMP) to host_ip
///
/// Depending on the server software ip/port information might not be included
/// which a Option is wrapped around its type.
///
#[derive(Debug)]
pub struct ShortQuery {
    pub motd: String,
    pub gametype: String,
    pub map: String,
    /// players represents how many players are currently online in the given server
    pub players: usize,
    pub max_players: usize,
    /// The port that the server is running on
    pub host_port: u16,
    pub host_ip: String,
}