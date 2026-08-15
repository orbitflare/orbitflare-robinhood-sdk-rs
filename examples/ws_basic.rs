use orbitflare_robinhood_sdk::{Result, WsClientBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let client = WsClientBuilder::new().build().await?;

    let mut sub = client.new_heads_subscribe().await?;

    while let Some(head) = sub.next().await {
        println!("block: {}", head.number);
    }

    sub.unsubscribe().await;

    Ok(())
}
