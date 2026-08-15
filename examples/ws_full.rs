use orbitflare_robinhood_sdk::primitives::b256;
use orbitflare_robinhood_sdk::{Filter, Result, WsClientBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = WsClientBuilder::new()
        .url("wss://robinhood.rpc.orbitflare.com")
        .build()
        .await?;

    let mut heads = client.new_heads_subscribe().await?;

    let transfer_topic = b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
    let mut transfers = client
        .logs_subscribe(&Filter::new().event_signature(transfer_topic))
        .await?;

    println!("watching new heads and ERC-20 transfers...");

    loop {
        tokio::select! {
            Some(head) = heads.next() => {
                println!("block {} (gas used {})", head.number, head.gas_used);
            }
            Some(log) = transfers.next() => {
                let tx = log.transaction_hash.unwrap_or_default();
                println!("transfer on {} in {tx}", log.address());
            }
        }
    }
}
