<p align="center">
  <img src="https://raw.githubusercontent.com/orbitflare/orbitflare-robinhood-sdk-rs/main/assets/banner.png" alt="orbitflare-robinhood-sdk" width="100%">
</p>

<p align="center">
  <a href="https://crates.io/crates/orbitflare-robinhood-sdk"><img src="https://img.shields.io/crates/v/orbitflare-robinhood-sdk.svg?style=flat-square&color=3CAB9C&labelColor=041815" alt="crates.io"></a>
  <a href="https://docs.orbitflare.com/sdk/overview"><img src="https://img.shields.io/badge/docs-orbitflare.com-3CAB9C?style=flat-square&labelColor=041815" alt="Documentation"></a>
</p>

# orbitflare-robinhood-sdk

Rust SDK for Robinhood Chain by [OrbitFlare](https://orbitflare.com) - RPC and WebSocket clients.

Built on [alloy](https://github.com/alloy-rs/alloy) types (`Address`, `U256`, `B256`, `Filter`, `TransactionRequest`, typed blocks, transactions, receipts, and logs), with OrbitFlare's own transport: endpoint failover, retry with backoff, and self-healing WebSocket subscriptions.

## Install

```bash
cargo add orbitflare-robinhood-sdk
```

Only the RPC client is enabled by default. Enable what you need:

```bash
cargo add orbitflare-robinhood-sdk --features ws
cargo add orbitflare-robinhood-sdk --features all
```

Or in your `Cargo.toml`:

```toml
[dependencies]
orbitflare-robinhood-sdk = { version = "0.1.0", features = ["all"] }
```

## RPC

```rust
use orbitflare_robinhood_sdk::primitives::address;
use orbitflare_robinhood_sdk::{Result, RpcClientBuilder};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let client = RpcClientBuilder::new().build()?;

    let block = client.get_block_number().await?;
    let balance = client
        .get_balance(address!("d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"))
        .await?;
    let gas_price = client.get_gas_price().await?;

    let syncing = client.request("eth_syncing", json!([])).await?;

    let raw = client
        .request_raw(r#"{"jsonrpc":"2.0","id":1,"method":"web3_clientVersion","params":[]}"#)
        .await?;

    Ok(())
}
```

### Typed helpers

All addresses, hashes, and quantities are alloy types. Common ones are re-exported at the crate root; the full crates are available as `orbitflare_robinhood_sdk::primitives` (alloy-primitives) and `orbitflare_robinhood_sdk::rpc_types` (alloy-rpc-types-eth).

| Method | Returns |
|---|---|
| `get_block_number()` | `u64` |
| `get_chain_id()` | `u64` |
| `get_balance(Address)` | `U256` wei |
| `get_transaction_count(Address)` | `u64` nonce |
| `get_gas_price()` | `u128` wei |
| `max_priority_fee_per_gas()` | `u128` wei |
| `get_block_by_number(impl Into<BlockNumberOrTag>, full_txs)` | `Option<Block>` |
| `get_transaction_by_hash(B256)` | `Option<Transaction>` |
| `get_transaction_receipt(B256)` | `Option<TransactionReceipt>` |
| `get_logs(&Filter)` | `Vec<Log>` |
| `get_code(Address)` | `Bytes` |
| `call(&TransactionRequest)` | `Bytes` |
| `estimate_gas(&TransactionRequest)` | `u64` |
| `send_raw_transaction(&[u8])` | `B256` tx hash |
| `fee_history(blocks, newest, percentiles)` | `FeeHistory` |
| `request(method, params)` | Any RPC method by name (`serde_json::Value`) |
| `request_raw(body)` | Raw JSON-RPC body string |

State queries (`get_balance`, `call`, ...) use the client's block tag, set via `.block_tag(BlockNumberOrTag::Finalized)` on the builder (default: `Latest`).

### Log filters

`get_logs` takes alloy's `Filter` directly:

```rust
use orbitflare_robinhood_sdk::primitives::{address, b256};
use orbitflare_robinhood_sdk::{BlockNumberOrTag, Filter};

let transfer_topic =
    b256!("ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

let filter = Filter::new()
    .from_block(1_000_000u64)
    .to_block(BlockNumberOrTag::Latest)
    .address(address!("1111111111111111111111111111111111111111"))
    .event_signature(transfer_topic);

let logs = client.get_logs(&filter).await?;
```

### Sending transactions

Build and sign with alloy (`alloy-signer`, `alloy-network`), then broadcast through the SDK:

```rust
let hash = client.send_raw_transaction(&signed_tx_rlp).await?;
let receipt = client.get_transaction_receipt(hash).await?;
```

## WebSocket

```rust
use orbitflare_robinhood_sdk::{Result, WsClientBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let client = WsClientBuilder::new().build().await?;

    let mut sub = client.new_heads_subscribe().await?;

    while let Some(head) = sub.next().await {
        println!("{}", head.number);
    }

    Ok(())
}
```

Subscriptions are typed: `new_heads_subscribe()` yields `Header`, `logs_subscribe(&Filter)` yields `Log`, `new_pending_transactions_subscribe()` yields `B256`. All auto-resubscribe on reconnect. `next_raw()` returns the raw `serde_json::Value` payload if you need it.

## Endpoint failover

All clients support multiple endpoints with automatic failover and health tracking.

```rust
let client = RpcClientBuilder::new()
    .url("https://robinhood.rpc.orbitflare.com")
    .fallback_urls(&[/* additional endpoints */])
    .build()?;
```

Failing endpoints are quarantined with exponential cooldown (10s, 20s, 40s, max 60s) and automatically retried once the cooldown expires. Healthy endpoints are always preferred.

## Retry

RPC calls retry on transient errors (5xx, 429, connection resets, JSON-RPC error code -32005) with exponential backoff before failing over to the next endpoint. 429 responses with a `Retry-After` header are respected.

WebSocket connections use active ping/pong to detect dead connections. Defaults are a ping every 10s and 3 missed pongs before the connection is considered dead, configurable via the builder:

```rust
let client = WsClientBuilder::new()
    .url("wss://robinhood.rpc.orbitflare.com")
    .ping_interval_secs(15)
    .max_missed_pongs(5)
    .build()
    .await?;
```

The WebSocket client reconnects automatically on disconnection and re-subscribes all active subscriptions after reconnecting.

Configure retry behavior:

```rust
use orbitflare_robinhood_sdk::RetryPolicy;
use std::time::Duration;

let client = RpcClientBuilder::new()
    .url("https://robinhood.rpc.orbitflare.com")
    .retry(RetryPolicy {
        initial_delay: Duration::from_millis(200),
        max_delay: Duration::from_secs(15),
        multiplier: 2.0,
        max_attempts: 5,
    })
    .build()?;
```

## Endpoints

Both builders default to the production OrbitFlare endpoints, so `.url()` is optional. Resolution order: `.url()` on the builder, then the environment variable, then the default.

| Client | Default |
|---|---|
| RPC | `https://robinhood.rpc.orbitflare.com` (`rpc::DEFAULT_RPC_URL`) |
| WebSocket | `wss://robinhood.rpc.orbitflare.com` (`ws::DEFAULT_WS_URL`) |

## Environment variables

| Variable | Used by | Purpose |
|---|---|---|
| `ORBITFLARE_LICENSE_KEY` | RPC, WebSocket | API key appended to endpoint URLs |
| `ORBITFLARE_ROBINHOOD_RPC_URL` | RPC | Overrides the default endpoint if `.url()` is not called |
| `ORBITFLARE_ROBINHOOD_WS_URL` | WebSocket | Overrides the default endpoint if `.url()` is not called |
