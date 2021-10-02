#![feature(async_closure)]

//! An async minecraft query library implementing raknet pings and generic long querying.
//!
//! This crate is mainly meant for use with Minecraft Bedrock Edition, but is usable on java servers with a long query.
//! Example
//! ```no_run
//! use rsquery::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!    // Returns rsquery::model::ShortQuery which implements Debug.
//!    println!("{:?}", Client::new("velvetpractice.live").await?.short_query().await?);
//!    Ok(())
//! }
//! ```
//! This crate works off of a custom Client struct and two response structs listed here:<br>
//! [Client](crate::Client)<br>
//! [ShortQuery](crate::model::ShortQuery)<br>
//! [LongQuery](crate::model::LongQuery)<br>

use std::sync::Arc;
use tokio::net::{UdpSocket, ToSocketAddrs};
use std::io::{Result, ErrorKind, Error, Write, Cursor};
use hex::FromHex;
use crate::model::{ShortQuery, LongQuery, packet, RakNetPong};
use std::time::{SystemTime, UNIX_EPOCH};
use byteorder::{WriteBytesExt, BigEndian, LittleEndian, ReadBytesExt};
use rand::Rng;
use std::str;
use std::collections::HashMap;
use tokio::sync::Mutex;
use crate::utils::read_nulltermed_str;

#[cfg(test)]
mod tests;
pub mod model;
mod utils;

pub struct Client<A: ToSocketAddrs> {
    socket: Arc<UdpSocket>,
    remote: A,
}

impl<A: ToSocketAddrs> Client<A> {

    /// Constructs a new Client targeted to that said remote.
    ///
    /// This function is async because as
    /// of now this struct keeps a locally binded socket open while it is in use.
    /// Meaning you have to await it and error check to see if the local socket successfully bound.
    ///
    /// # [Errors]
    /// - On bind failure
    ///
    /// # [Example]
    /// ```no_run
    /// use rsquery::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Error> {
    ///     let client = Client::new("ip:port").await?;
    ///     // Client successfully bound you can now safely use it
    ///     Ok(())
    ///     // Client is dropped now and the socket should be closed
    /// }
    /// ```
    pub async fn new(remote: A) -> Result<Self> {
        let socket =  Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        Ok(Client {
            socket,
            remote,
        })
    }

    /// Returns the given remote this client is currently pointing too
    pub fn remote(&self) -> &A {
        &self.remote
    }

    /// Used to make one client reusable.
    ///
    /// Requires the client to be borrowed mutably and then sets the remote to the given parameter.
    ///
    /// # [Example]
    /// ```no_run
    /// let mut client = Client::new("ip:port").await?;
    /// // Short Query one server.
    /// let data1 = client.short_query().await?;
    /// // Set the new remote.
    /// client.set_remote("ip:port");
    /// // Long Query another server
    /// let data2 = client.long_query().await?;
    /// ```
    pub fn set_remote(&mut self, remote: A) {
        self.remote = remote;
    }

    /// A fast and easy query using raknet unconnected ping and pong.
    ///
    /// Uses the locally bound socket (Client.socket) to send a raknet Unconnected_Ping to the given remote.
    ///
    /// For information on the data returned view [RakNetPong](crate::model::RakNetPong)
    ///
    /// # [Errors]
    /// - Polling for timeout
    /// - Invalid Data
    /// - Connection Failure
    ///
    /// # [Example]
    /// ```no_run
    /// // Open local binded port and query the given server address.
    /// let data = Client::new("ip:port").await?.short_query().await?;
    /// // Prints out the amount of players on that server at the time of querying.
    /// println!("player_count: {}", data.player_count); // EX: player_count: 5
    /// ```
    pub async fn raknet_ping(&self) -> Result<RakNetPong> {
        // Writing
        let mut random = rand::thread_rng();
        let offline_msg_data = Vec::from_hex("00ffff00fefefefefdfdfdfd12345678").expect("Failed to read binary string!");
        {
            //Initalize Buf with 0x01 being the ID_UNCONNECTED_PING
            let mut buf: Vec<u8> = vec![0x01];
            //Write the current time stamp
            buf.write_i64::<BigEndian>(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64)?;
            //Hex literal for Offline Message Data ID
            buf.extend(&offline_msg_data);
            //Write a random client id
            buf.write_u64::<BigEndian>(random.gen::<u64>())?;
            //Send query to remote socket
            self.socket.send_to(buf.as_slice(), &self.remote).await?;
        }; //purge temporary buf out of scope
        // begin reading
        let mut buf = [0u8; u16::MAX as usize];
        //Read data into temp buffer ^^
        let len = self.socket.recv(&mut buf).await?;
        //Split the data into a vector made of Strings
        let data: Vec<String> = String::from_utf8_lossy(&buf[offline_msg_data.len()+19..=len])
            .split(';').map(String::from).collect();
        let mut gamemode = None;
        let mut motd = vec![data[1].clone()];
        if data.len() > 7 {
            motd.push(data[7].clone());
            gamemode = Some(data[8].clone())
        }
        Ok(RakNetPong {
            game_edition: data[0].clone(),
            motd,
            protocol_version: data[2].parse().unwrap(),
            game_version: data[3].clone(),
            player_count: data[4].parse().unwrap(),
            max_player_count: data[5].parse().unwrap(),
            server_uid: data[6].clone(),
            game_mode: gamemode,
            game_mode_integer: None,
            port: None,
            port_v6: None
        })
    }

    /// A slightly slower query implementation, but returns more detailed data.
    ///
    /// Uses the locally bound socket (Client.socket) to send a HandShake request and a Stat request.
    ///
    /// This returns data like a list of player names the server engine and much more
    ///
    /// view [LongQuery](crate::model::LongQuery) for details
    ///
    /// # [Errors]
    /// - Polling for timeout
    /// - Invalid Data
    /// - Connection Failure
    ///
    /// # [Example]
    /// ```no_run
    /// // Open local binded port and long query the given server address
    /// let data = Client::new("ip:port").await?.long_query().await?;
    /// // Prints out the Vec<String> using Debug trait.
    /// println!("players: {:?}", data.players) // EX: players: ["Timmy", "Bobby2454"]
    /// ```
    pub async fn long_query(&self) -> Result<LongQuery> {
        let mut random = rand::thread_rng();
        let ses_id: i32 = random.gen();
        let challenge_token = self.gen_challenge_token(ses_id).await?;
        //Send Request
        {
            let mut buf: Vec<u8> = Vec::new();
            // Write Query Magic
            buf.write_u16::<BigEndian>(packet::MAGIC)?;
            // Write STAT for the packet id
            buf.write_u8(packet::STAT)?;
            // Write Session Id
            buf.write_i32::<BigEndian>(ses_id & 0x0F0F0F0F)?;
            // Write challenge token
            buf.write_i32::<BigEndian>(challenge_token)?;
            // Padding
            buf.write_all([0x00].repeat(4).as_slice())?;
            // Send STAT request to remote
            self.socket.send_to(buf.as_slice(), &self.remote).await?;
        };
        //Reading
        let mut buf = [0u8; u16::MAX as usize];
        let len = self.socket.recv(&mut buf).await?;
        //check if the packet id is STAT
        match buf[0] {
            packet::STAT => {
                let data = &buf[16..=len];
                let mut reg_data = &buf[16..=len];
                let players: Mutex<Vec<String>> = Mutex::new(Vec::new());
                let raw_data: Mutex<HashMap<&str, String>> = Mutex::new(HashMap::new());
                let player_index = utils::slice_index(data, &packet::PLAYER_KEY);
                if let Some(pi) = player_index {
                    reg_data = &data[0..=pi];
                };
                let a = async || -> Result<()> {
                    let mut arr = reg_data.split(|byte| byte == &0x00u8).collect::<Vec<&[u8]>>();
                    if arr.len() % 2 != 0 {
                        arr.pop();
                    }
                    let mut i: usize = 1;
                    for k in arr.iter().step_by(2) {
                        raw_data
                            .lock().await
                            .insert(str::from_utf8(*k).expect("Unable to decode key string"),
                                    str::from_utf8(arr[i]).expect("Unable to decode value string").to_string());
                        i += 2;
                    }
                    Ok(())
                };
                let b = async || -> Result<()> {
                    if let Some(pi) = player_index {
                        let tmp = &data[pi+packet::PLAYER_KEY.len()..data.len()-3];
                        players.lock().await.extend(tmp.split(|byte| byte == &0x00u8)
                            .map(|arr| str::from_utf8(arr).expect("Failure decoding string!").to_string()));
                    };
                    Ok(())
                };
                tokio::try_join!(a(), b())?;
                let reader = raw_data.lock().await;
                let players = players.lock().await.to_vec();
                Ok(LongQuery {
                    server_software: reader.get("server_engine").expect("Failed to find server_engine").clone(),
                    plugins: reader.get("plugins").expect("Failed to find plugins").clone(),
                    version: reader.get("version").expect("Failed to find version").clone(),
                    whitelist: reader.get("whitelist").expect("Failed to find whitelist").clone(),
                    players,
                    player_count: reader.get("numplayers").expect("Failed to find numplayers").parse().expect("Invalid Player Count!"),
                    max_players: reader.get("maxplayers").expect("Failed to find maxplayers").parse().expect("Invalid Max Player Count!"),
                    game_name: reader.get("game_id").expect("Failed to find gamename").clone(),
                    game_mode: reader.get("gametype").expect("Failed to find gametype").clone(),
                    map_name: reader.get("map").expect("Failed to find map").clone(),
                    host_name: reader.get("hostname").expect("Failed to find server_engine").clone(),
                    host_ip: reader.get("hostip").expect("Failed to find hostip").clone(),
                    host_port: reader.get("hostport").expect("Failed to find server_engine").parse().expect("Invalid Host Port!")
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Unexpected packet was received while awaiting 0x00 STAT"))
        }
    }

    /// A slightly faster implementation of the long query found in BASIC STAT for GS3
    ///
    /// this function uses the locally bound socket to do a full HANDSHAKE and STAT interaction
    ///
    /// This returns data like the player count and gametype
    ///
    /// view [ShortQuery](crate::model::ShortQuery) for details
    ///
    /// # [Errors]
    /// - Polling for timeout
    /// - Invalid Data
    /// - Connection Failure
    ///
    /// # [Example]
    /// ```no_run
    /// // Open local binded port and long query the given server address
    /// let data = Client::new("ip:port").await?.short_query().await?;
    /// // Prints out the usize using Display trait.
    /// println!("players: {}", data.players) // EX: players: 2
    /// ```
    pub async fn short_query(&self) -> Result<ShortQuery> {
        let mut random = rand::thread_rng();
        let ses_id: i32 = random.gen();
        let challenge_token = self.gen_challenge_token(ses_id).await?;
        {
            let mut buf: Vec<u8> = Vec::new();
            // Write Query Magic
            buf.write_u16::<BigEndian>(packet::MAGIC)?;
            // Write STAT for the packet id
            buf.write_u8(packet::STAT)?;
            // Write Session Id
            buf.write_i32::<BigEndian>(ses_id & 0x0F0F0F0F)?;
            // Write challenge token
            buf.write_i32::<BigEndian>(challenge_token)?;
            // Send STAT request to remote
            self.socket.send_to(buf.as_slice(), &self.remote).await?;
        };
        //Reading
        let mut buf = [0u8; u16::MAX as usize];
        let len = self.socket.recv(&mut buf).await?;
        match buf[0] {
            packet::STAT => {
                let mut buf = Cursor::new(&buf[5..len]);
                let motd = read_nulltermed_str(&mut buf).await?;
                let gametype = read_nulltermed_str(&mut buf).await?;
                let map = read_nulltermed_str(&mut buf).await?;
                let players = read_nulltermed_str(&mut buf).await?.parse().unwrap();
                let max_players = read_nulltermed_str(&mut buf).await?.parse().unwrap();
                let host_port = buf.read_u16::<LittleEndian>()?;
                let host_ip = read_nulltermed_str(&mut buf).await?;
                Ok(ShortQuery {
                    motd,
                    gametype,
                    map,
                    players,
                    max_players,
                    host_port,
                    host_ip
                })
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Unexpected packet was received while awaiting 0x00 STAT")),
        }
    }

    /// Generates a challenge token for a given session id
    /// # [Example]
    /// with a random session id
    ///
    /// ```no_run
    /// let token: i32 = Client::new("ip:port").await?.gen_challenge_token(rand::thread_rng().gen()).await?;
    /// ```
    pub async fn gen_challenge_token(&self, sid: i32) -> Result<i32> {
        let mut buf: Vec<u8> = Vec::new();
        //Writes query protocol magic to the buf always 0xFEFD
        buf.write_u16::<BigEndian>(packet::MAGIC)?;
        //Sending a handshake so the server sends back a challenge token for our given session id (always 0x09)
        buf.write_u8(packet::HANDSHAKE)?;
        //Writing the sid to the buf
        buf.write_i32::<BigEndian>(sid & 0x0F0F0F0F)?;
        //Use locally bound port to send to remote.
        self.socket.send_to(buf.as_slice(), &self.remote).await?;
        //remove buf from mem
        drop(buf);
        //Begin reading the data
        let mut buf = [0u8; (u16::MAX >> 2) as usize];
        let len = self.socket.recv(&mut buf).await?;
        match buf[0] {
            packet::HANDSHAKE => {
                Ok(String::from_utf8_lossy(&buf[5..len-1]).parse().expect("Invalid Challenge Token Received"))
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Wrong packet received perhaps an already opened session? (expected 0x01 Handshake)"))
        }
    }
}
