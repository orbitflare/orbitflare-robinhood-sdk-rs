use orbitflare_robinhood_sdk::primitives::address;
use orbitflare_robinhood_sdk::{Result, RpcClientBuilder};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let client = RpcClientBuilder::new().build()?;

    let block = client.get_block_number().await?;
    println!("block: {block}");

    let wallet = address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045");
    let balance = client.get_balance(wallet).await?;
    println!("balance: {balance} wei");

    let gas_price = client.get_gas_price().await?;
    println!("gas price: {gas_price} wei");

    let chain_id = client.get_chain_id().await?;
    println!("chain id: {chain_id}");

    let syncing = client.request("eth_syncing", json!([])).await?;
    println!("syncing: {syncing}");

    let raw = client
        .request_raw(r#"{"jsonrpc":"2.0","id":1,"method":"web3_clientVersion","params":[]}"#)
        .await?;
    println!("client version: {raw}");

    Ok(())
}
