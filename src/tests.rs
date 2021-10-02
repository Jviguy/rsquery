use crate::Client;
use std::io::Result;
use tokio::time::Instant;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn raknet_ping() -> Result<()> {
    let client = Client::new("velvetpractice.live:19132").await?;
    let start = Instant::now();
    let data = client.raknet_ping().await?;
    println!("short finished in {}ms\n{:?}", start.elapsed().as_millis(), data);
    Ok(())
}

#[tokio::test]
async fn long_query() -> Result<()> {
    let client = Client::new("velvetpractice.live:19132").await?;
    let start = Instant::now();
    let data = client.long_query().await?;
    println!("long finished in {}ms\n{:?}", start.elapsed().as_millis(), data);
    Ok(())
}

#[tokio::test]
async fn slice_index() -> Result<()> {
    let mut source: Vec<u8> = vec![0x01, 0x02];
    source.write(&crate::packet::PLAYER_KEY).await?;
    println!("index: {:?}", crate::utils::slice_index(source.as_slice(), &crate::packet::PLAYER_KEY));
    Ok(())
}