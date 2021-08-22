use crate::Client;
use std::io::Result;
use tokio::time::Instant;

#[actix_rt::test]
async fn it_works() -> Result<()> {
    let start = Instant::now();
    let data = Client::new("velvetpractice.live:19132").await?.short_query().await?;
    println!("finished in {}ms\n{:?}", start.elapsed().as_millis(), data);
    Ok(())
}