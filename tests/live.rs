#![cfg(feature = "ws")]

use orbitflare_robinhood_sdk::{
    BlockNumberOrTag, Filter, RetryPolicy, RpcClient, RpcClientBuilder, WsClient, WsClientBuilder,
};
use std::time::Duration;

const ROBINHOOD_CHAIN_ID: u64 = 4663;

fn bounded_retry() -> RetryPolicy {
    RetryPolicy {
        max_attempts: 2,
        ..Default::default()
    }
}

fn live_rpc() -> RpcClient {
    RpcClientBuilder::new()
        .timeout(Duration::from_secs(15))
        .retry(bounded_retry())
        .build()
        .unwrap()
}

async fn live_ws() -> WsClient {
    WsClientBuilder::new()
        .retry(bounded_retry())
        .build()
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "live network test"]
async fn rpc_chain_id_is_robinhood() {
    let chain_id = live_rpc().get_chain_id().await.unwrap();
    assert_eq!(chain_id, ROBINHOOD_CHAIN_ID);
}

#[tokio::test]
#[ignore = "live network test"]
async fn rpc_block_number_advances() {
    let client = live_rpc();
    let first = client.get_block_number().await.unwrap();
    assert!(first > 0);

    tokio::time::sleep(Duration::from_secs(3)).await;
    let second = client.get_block_number().await.unwrap();
    assert!(second > first, "chain did not advance: {first} -> {second}");
}

#[tokio::test]
#[ignore = "live network test"]
async fn rpc_latest_block_is_typed() {
    let block = live_rpc()
        .get_block_by_number(BlockNumberOrTag::Latest, false)
        .await
        .unwrap()
        .expect("latest block should exist");
    assert!(block.header.number > 0);
    assert!(block.header.timestamp > 0);
    assert!(block.header.gas_limit > 0);
}

#[tokio::test]
#[ignore = "live network test"]
async fn rpc_gas_price_nonzero() {
    let price = live_rpc().get_gas_price().await.unwrap();
    assert!(price > 0);
}

#[tokio::test]
#[ignore = "live network test"]
async fn rpc_fee_history_typed() {
    let history = live_rpc()
        .fee_history(4, BlockNumberOrTag::Latest, &[25.0, 75.0])
        .await
        .unwrap();
    assert!(history.oldest_block > 0);
    assert!(!history.base_fee_per_gas.is_empty());
}

#[tokio::test]
#[ignore = "live network test"]
async fn rpc_block_transactions_roundtrip() {
    let client = live_rpc();
    let tip = client.get_block_number().await.unwrap();

    let mut found = None;
    for n in (tip.saturating_sub(50)..=tip).rev() {
        let block = client.get_block_by_number(n, false).await.unwrap();
        if let Some(block) = block
            && let Some(hash) = block.transactions.hashes().next()
        {
            found = Some(hash);
            break;
        }
    }
    let hash = found.expect("no transactions in the last 50 blocks");

    let tx = client
        .get_transaction_by_hash(hash)
        .await
        .unwrap()
        .expect("transaction should exist");
    assert!(tx.block_number.is_some());

    let receipt = client
        .get_transaction_receipt(hash)
        .await
        .unwrap()
        .expect("receipt should exist");
    assert_eq!(receipt.transaction_hash, hash);
}

#[tokio::test]
#[ignore = "live network test"]
async fn rpc_request_escape_hatch() {
    let version = live_rpc()
        .request("web3_clientVersion", serde_json::json!([]))
        .await
        .unwrap();
    let version = version.as_str().unwrap();
    assert!(version.contains("nitro"), "unexpected client: {version}");
}

#[tokio::test]
#[ignore = "live network test"]
async fn ws_new_heads_stream_typed() {
    let client = live_ws().await;
    let mut sub = client.new_heads_subscribe().await.unwrap();

    let first = tokio::time::timeout(Duration::from_secs(30), sub.next())
        .await
        .expect("no head within 30s")
        .expect("stream closed");
    let second = tokio::time::timeout(Duration::from_secs(30), sub.next())
        .await
        .expect("no second head within 30s")
        .expect("stream closed");

    assert!(second.number > first.number);

    sub.unsubscribe().await;
}

#[tokio::test]
#[ignore = "live network test"]
async fn ws_logs_subscription_accepted() {
    let client = live_ws().await;
    let sub = client.logs_subscribe(&Filter::new()).await.unwrap();
    sub.unsubscribe().await;
}

#[tokio::test]
#[ignore = "live network test"]
async fn ws_unsubscribe_stops_events() {
    let client = live_ws().await;
    let mut keep = client.new_heads_subscribe().await.unwrap();
    let drop_me = client.new_heads_subscribe().await.unwrap();

    drop_me.unsubscribe().await;
    tokio::time::timeout(Duration::from_secs(30), keep.next())
        .await
        .expect("no head within 30s after sibling unsubscribe")
        .expect("stream closed");
}
