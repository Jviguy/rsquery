//! An async minecraft query library implementing raknet pings and generic long querying.
//!
//! This crate is mainly meant for use with Minecraft Bedrock Edition, but is usable on java servers with a long query.
//! Example
//! ```no_run
//! use rsquery::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Error> {
//!    // Returns rsquery::ShortQuery which implements Debug.
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
use std::io::{Result, ErrorKind, Error};
use std::net::SocketAddr;
use hex::FromHex;
use crate::model::{ShortQuery, LongQuery, packet};
use std::time::{SystemTime, UNIX_EPOCH};
use byteorder::{WriteBytesExt, BigEndian};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::str;
use std::mem::size_of;

#[cfg(test)]
mod tests;
pub mod model;

pub struct Client<A: ToSocketAddrs> {
    socket: Arc<UdpSocket>,
    remote: A,
}

impl<A: ToSocketAddrs> Client<A> {
    pub async fn new(remote: A) -> Result<Self> {
        let socket =  Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        Ok(Client {
            socket,
            remote,
        })
    }

    /// A fast and easy query using raknet unconnected ping and pong.<br>
    /// Example
    /// ```rs
    /// let data = Client::new("velvetpractice.live").short_query().await?;
    /// ```
    pub async fn short_query(&self) -> Result<ShortQuery> {
        // Writing
        let mut random = rand::thread_rng();
        //Initalize Buf with 0x01 being the ID_UNCONNECTED_PING
        let mut buf: Vec<u8> = vec![0x01];
        //Write the current time stamp
        buf.write_i64::<BigEndian>(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64)?;
        //Hex literal for Offline Message Data ID
        let offline_msg_data = Vec::from_hex("00ffff00fefefefefdfdfdfd12345678").expect("Failed to read binary string!");
        buf.extend(&offline_msg_data);
        //Write a random client id
        buf.write_u64::<BigEndian>(random.gen::<u64>())?;
        //Send query to remote socket
        self.socket.send_to(buf.as_slice(), &self.remote).await?;
        //purge temporary buf out of scope
        drop(buf);
        // begin reading
        let mut buf = [0u8; u16::MAX as usize];
        //Read data into temp buffer ^^
        let len = self.socket.recv(&mut buf).await?;
        //Split the data into a vector made of Strings
        let data: Vec<String> = String::from_utf8_lossy(&buf[offline_msg_data.len()+19..=len])
            .split(";").map(|s| String::from(s)).collect();
        let mut gamemode = None;
        let mut motd = vec![data[1].clone()];
        if data.len() > 7 {
            motd.push(data[7].clone());
            gamemode = Some(data[8].clone())
        }
        Ok(ShortQuery {
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

    pub async fn long_query() {

    }
    /// Generates a challenge token for a given session id
    /// Example
    /// ```rs
    /// let token: i32 = Client::new(...).gen_challenge_token(rand::thread_rng().gen())
    /// ```
    /// That example generates a challenge token for a random session id.
    pub async fn gen_challenge_token(&self, sid: i32) -> Result<i32> {
        let mut buf: Vec<u8> = Vec::new();
        //Writes query protocol magic to the buf always 0xFEFD
        buf.write_u16::<BigEndian>(packet::MAGIC)?;
        //Sending a handshake so the server sends back a challenge token for our given session id (always 0x09)
        buf.write_u8(packet::HANDSHAKE)?;
        //Writing the sid to the buf
        buf.write_i32::<BigEndian>(sid)?;
        //Use locally bound port to send to remote.
        self.socket.send_to(buf.as_slice(), &self.remote).await?;
        //remote buf from mem
        drop(buf);
        //Begin reading the data
        let mut buf = [0u8; (u16::MAX >> 2) as usize];
        let len = self.socket.recv(&mut buf).await?;
        match buf[0] {
            packet::HANDSHAKE => {
               Ok(str::from_utf8(&buf[1+size_of::<i32>()..=len]).expect("failed to read challenge token.").parse().unwrap())
            },
            _ => Err(Error::new(ErrorKind::InvalidData, "Wrong packet received perhaps an already opened session? (expected 0x01 Handshake)"))
        }
    }
}
