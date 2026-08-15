use orbitflare_robinhood_sdk::primitives::{address, b256, utils::format_ether};
use orbitflare_robinhood_sdk::{BlockNumberOrTag, Filter, Result, RpcClientBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = RpcClientBuilder::new()
        .url("https://robinhood.rpc.orbitflare.com")
        .build()?;

    let wallet = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");

    let block = client.get_block_number().await?;
    let gas_price = client.get_gas_price().await?;
    println!("network block: {block}");
    println!("gas price: {:.2} gwei", gas_price as f64 / 1e9);

    let balance = client.get_balance(wallet).await?;
    let nonce = client.get_transaction_count(wallet).await?;
    println!("\n{wallet}");
    println!("  ETH: {}", format_ether(balance));
    println!("  nonce: {nonce}");

    if let Some(latest) = client
        .get_block_by_number(BlockNumberOrTag::Latest, false)
        .await?
    {
        println!("\nlatest block {}", latest.header.hash);
        println!("  transactions: {}", latest.transactions.len());
        println!("  gas used: {}", latest.header.gas_used);
    }

    let transfer_topic = b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");
    let filter = Filter::new()
        .from_block(block.saturating_sub(1000))
        .to_block(BlockNumberOrTag::Latest)
        .event_signature(transfer_topic);
    let logs = client.get_logs(&filter).await?;
    println!("\ntransfer logs in last 1000 blocks: {}", logs.len());
    for log in logs.iter().take(5) {
        println!(
            "  {} {}",
            log.block_number.unwrap_or_default(),
            log.address()
        );
    }

    let history = client
        .fee_history(4, BlockNumberOrTag::Latest, &[25.0, 75.0])
        .await?;
    println!("\nfee history from block {}", history.oldest_block);

    Ok(())
}
