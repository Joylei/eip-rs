use anyhow::Result;
use futures_util::StreamExt;
use rseip::client::discover;

#[tokio::main]
pub async fn main() -> Result<()> {
    let stream = discover("192.168.0.22:0".parse()?, "192.168.0.255:44818".parse()?).await?;
    tokio::pin!(stream);
    let item = stream.next().await.unwrap();
    println!("{:?}", item);

    Ok(())
}
