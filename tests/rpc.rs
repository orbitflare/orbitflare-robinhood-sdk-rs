use orbitflare_robinhood_sdk::primitives::{Address, B256, U256, address, b256, bytes};
use orbitflare_robinhood_sdk::rpc_types::TransactionInput;
use orbitflare_robinhood_sdk::{
    BlockNumberOrTag, Filter, ReceiptResponse, RetryPolicy, RpcClientBuilder, TransactionRequest,
};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

async fn mock_rpc(
    body: serde_json::Value,
) -> (MockServer, orbitflare_robinhood_sdk::rpc::RpcClient) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(&server)
        .await;
    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();
    (server, client)
}

fn hash_hex(b: u8) -> String {
    format!("0x{}", format!("{b:02x}").repeat(32))
}

fn addr_hex(b: u8) -> String {
    format!("0x{}", format!("{b:02x}").repeat(20))
}

fn bloom_hex() -> String {
    format!("0x{}", "00".repeat(256))
}

#[tokio::test]
async fn get_block_number_returns_u64() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x1b4","id":1})).await;
    let block = client.get_block_number().await.unwrap();
    assert_eq!(block, 436);
}

#[tokio::test]
async fn get_chain_id_returns_u64() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0xa4b1","id":1})).await;
    let chain_id = client.get_chain_id().await.unwrap();
    assert_eq!(chain_id, 42161);
}

#[tokio::test]
async fn get_balance_returns_u256_wei() {
    let (server, client) =
        mock_rpc(json!({"jsonrpc":"2.0","result":"0xde0b6b3a7640000","id":1})).await;
    let balance = client
        .get_balance(Address::repeat_byte(0xaa))
        .await
        .unwrap();
    assert_eq!(balance, U256::from(1_000_000_000_000_000_000u128));

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["params"][0], addr_hex(0xaa));
    assert_eq!(body["params"][1], "latest");
}

#[tokio::test]
async fn get_transaction_count_returns_nonce() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x2a","id":1})).await;
    let nonce = client.get_transaction_count(Address::ZERO).await.unwrap();
    assert_eq!(nonce, 42);
}

#[tokio::test]
async fn get_gas_price_returns_wei() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x3b9aca00","id":1})).await;
    let price = client.get_gas_price().await.unwrap();
    assert_eq!(price, 1_000_000_000);
}

#[tokio::test]
async fn get_block_returns_none_for_missing() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":null,"id":1})).await;
    let block = client
        .get_block_by_number(BlockNumberOrTag::Number(1), false)
        .await
        .unwrap();
    assert!(block.is_none());
}

#[tokio::test]
async fn get_block_returns_typed_block() {
    let (_server, client) = mock_rpc(json!({
        "jsonrpc":"2.0",
        "result":{
            "number":"0x1b4",
            "hash":hash_hex(0x11),
            "parentHash":hash_hex(0x22),
            "sha3Uncles":hash_hex(0x33),
            "miner":addr_hex(0x44),
            "stateRoot":hash_hex(0x55),
            "transactionsRoot":hash_hex(0x66),
            "receiptsRoot":hash_hex(0x77),
            "logsBloom":bloom_hex(),
            "difficulty":"0x0",
            "gasLimit":"0x1c9c380",
            "gasUsed":"0x5208",
            "timestamp":"0x64",
            "extraData":"0x",
            "mixHash":hash_hex(0x88),
            "nonce":"0x0000000000000000",
            "baseFeePerGas":"0x7",
            "size":"0x220",
            "transactions":[],
            "uncles":[]
        },
        "id":1
    }))
    .await;
    let block = client.get_block_by_number(436u64, false).await.unwrap();
    let block = block.unwrap();
    assert_eq!(block.header.number, 436);
    assert_eq!(block.header.hash, B256::repeat_byte(0x11));
    assert_eq!(block.header.base_fee_per_gas, Some(7));
}

#[tokio::test]
async fn get_block_sends_hex_number() {
    let (server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":null,"id":1})).await;
    client.get_block_by_number(436u64, true).await.unwrap();

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["method"], "eth_getBlockByNumber");
    assert_eq!(body["params"][0], "0x1b4");
    assert_eq!(body["params"][1], true);
}

#[tokio::test]
async fn get_transaction_by_hash_returns_typed_transaction() {
    let (_server, client) = mock_rpc(json!({
        "jsonrpc":"2.0",
        "result":{
            "hash":"0x88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b",
            "nonce":"0x0",
            "blockHash":hash_hex(0x11),
            "blockNumber":"0x64",
            "transactionIndex":"0x0",
            "from":"0xa7d9ddbe1f17865597fbd27ec712455208b6b76d",
            "to":"0xf02c1c8e6114b1dbe8937a39260b5b0a374432bb",
            "value":"0xf3dbb76162000",
            "gas":"0x2e1a3",
            "gasPrice":"0x4a817c800",
            "input":"0x",
            "v":"0x25",
            "r":"0x1b5e176d927f8e9ab405058b2d2457392da3e20f328b16ddabcebc33eaac5fea",
            "s":"0x4ba69724e8f69de52f0125ad8b3c5c2cef33019bac3249e2c0a2192766d1721c",
            "type":"0x0"
        },
        "id":1
    }))
    .await;
    let tx = client
        .get_transaction_by_hash(b256!(
            "88df016429689c079f3b2f6ad39fa052532c56795b733da78a91ebe6a713944b"
        ))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(tx.block_number, Some(100));
    assert_eq!(
        tx.inner.signer(),
        address!("a7d9ddbe1f17865597fbd27ec712455208b6b76d")
    );
}

#[tokio::test]
async fn get_transaction_receipt_returns_typed_receipt() {
    let (_server, client) = mock_rpc(json!({
        "jsonrpc":"2.0",
        "result":{
            "transactionHash":hash_hex(0x11),
            "transactionIndex":"0x0",
            "blockHash":hash_hex(0x22),
            "blockNumber":"0x64",
            "from":addr_hex(0x33),
            "to":addr_hex(0x44),
            "cumulativeGasUsed":"0x5208",
            "gasUsed":"0x5208",
            "contractAddress":null,
            "logs":[],
            "logsBloom":bloom_hex(),
            "status":"0x1",
            "effectiveGasPrice":"0x4a817c800",
            "type":"0x0"
        },
        "id":1
    }))
    .await;
    let receipt = client
        .get_transaction_receipt(B256::repeat_byte(0x11))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(receipt.block_number, Some(100));
    assert!(receipt.status());
    assert_eq!(receipt.gas_used, 21000);
}

#[tokio::test]
async fn get_logs_returns_typed_logs() {
    let (_server, client) = mock_rpc(json!({
        "jsonrpc":"2.0",
        "result":[
            {"address":addr_hex(0xaa),"topics":[],"data":"0x","blockNumber":"0x1","blockHash":hash_hex(0x21),"transactionHash":hash_hex(0x31),"transactionIndex":"0x0","logIndex":"0x0","removed":false},
            {"address":addr_hex(0xbb),"topics":[hash_hex(0x11)],"data":"0x01","blockNumber":"0x2","blockHash":hash_hex(0x22),"transactionHash":hash_hex(0x32),"transactionIndex":"0x1","logIndex":"0x1","removed":false}
        ],
        "id":1
    }))
    .await;
    let logs = client.get_logs(&Filter::new()).await.unwrap();
    assert_eq!(logs.len(), 2);
    assert_eq!(logs[0].address(), Address::repeat_byte(0xaa));
    assert_eq!(logs[1].topics()[0], B256::repeat_byte(0x11));
}

#[tokio::test]
async fn get_logs_sends_alloy_filter() {
    let (server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":[],"id":1})).await;

    let filter = Filter::new()
        .from_block(100u64)
        .to_block(BlockNumberOrTag::Latest)
        .address(address!("1111111111111111111111111111111111111111"))
        .event_signature(b256!(
            "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
        ))
        .topic2(b256!(
            "2222222222222222222222222222222222222222222222222222222222222222"
        ));

    client.get_logs(&filter).await.unwrap();

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    let params = &body["params"][0];

    assert_eq!(params["fromBlock"], "0x64");
    assert_eq!(params["toBlock"], "latest");
    assert_eq!(
        params["address"],
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(
        params["topics"][0],
        "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
    );
    assert!(params["topics"][1].is_null());
    assert_eq!(
        params["topics"][2],
        "0x2222222222222222222222222222222222222222222222222222222222222222"
    );
}

#[tokio::test]
async fn call_returns_bytes() {
    let (server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x0000002a","id":1})).await;
    let request = TransactionRequest {
        to: Some(address!("1111111111111111111111111111111111111111").into()),
        input: TransactionInput::new(bytes!("70a08231")),
        ..Default::default()
    };
    let result = client.call(&request).await.unwrap();
    assert_eq!(result, bytes!("0000002a"));

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["method"], "eth_call");
    assert_eq!(
        body["params"][0]["to"],
        "0x1111111111111111111111111111111111111111"
    );
    assert_eq!(body["params"][0]["input"], "0x70a08231");
    assert!(body["params"][0].get("from").is_none());
    assert_eq!(body["params"][1], "latest");
}

#[tokio::test]
async fn estimate_gas_returns_u64() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x5208","id":1})).await;
    let request = TransactionRequest {
        to: Some(Address::repeat_byte(0x11).into()),
        value: Some(U256::from(1)),
        ..Default::default()
    };
    let gas = client.estimate_gas(&request).await.unwrap();
    assert_eq!(gas, 21000);
}

#[tokio::test]
async fn get_code_returns_bytes() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x6080604052","id":1})).await;
    let code = client.get_code(Address::repeat_byte(0xcc)).await.unwrap();
    assert_eq!(code, bytes!("6080604052"));
}

#[tokio::test]
async fn send_raw_transaction_returns_hash() {
    let (server, client) = mock_rpc(json!({
        "jsonrpc":"2.0",
        "result":hash_hex(0x11),
        "id":1
    }))
    .await;
    let hash = client
        .send_raw_transaction(&[0xf8, 0x6c, 0x01])
        .await
        .unwrap();
    assert_eq!(hash, B256::repeat_byte(0x11));

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["params"][0], "0xf86c01");
}

#[tokio::test]
async fn fee_history_returns_typed() {
    let (server, client) = mock_rpc(json!({
        "jsonrpc":"2.0",
        "result":{
            "oldestBlock":"0x64",
            "baseFeePerGas":["0x1","0x2"],
            "gasUsedRatio":[0.5],
            "reward":[["0x1","0x2"]]
        },
        "id":1
    }))
    .await;
    let history = client
        .fee_history(4, BlockNumberOrTag::Latest, &[25.0, 75.0])
        .await
        .unwrap();
    assert_eq!(history.oldest_block, 100);
    assert_eq!(history.base_fee_per_gas, vec![1, 2]);

    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["params"][0], "0x4");
    assert_eq!(body["params"][1], "latest");
    assert_eq!(body["params"][2][0], 25.0);
}

#[tokio::test]
async fn request_custom_method() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x1","id":1})).await;
    let result = client.request("net_version", json!([])).await.unwrap();
    assert_eq!(result, "0x1");
}

#[tokio::test]
async fn request_raw_body() {
    let (_server, client) = mock_rpc(json!({"jsonrpc":"2.0","result":"ok","id":1})).await;
    let result = client
        .request_raw(r#"{"jsonrpc":"2.0","id":1,"method":"web3_clientVersion","params":[]}"#)
        .await
        .unwrap();
    assert_eq!(result, "ok");
}

#[tokio::test]
async fn block_tag_defaults_and_override() {
    let (_, default_client) = mock_rpc(json!({"jsonrpc":"2.0","result":"0x0","id":1})).await;
    assert_eq!(default_client.block_tag(), BlockNumberOrTag::Latest);

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","result":"0x0","id":1})),
        )
        .mount(&server)
        .await;
    let finalized = RpcClientBuilder::new()
        .url(&server.uri())
        .block_tag(BlockNumberOrTag::Finalized)
        .build()
        .unwrap();
    assert_eq!(finalized.block_tag(), BlockNumberOrTag::Finalized);

    finalized.get_balance(Address::ZERO).await.unwrap();
    let requests = server.received_requests().await.unwrap();
    let body: serde_json::Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert_eq!(body["params"][1], "finalized");
}

#[tokio::test]
async fn rpc_error_extracted() {
    let (_server, client) = mock_rpc(json!({
        "jsonrpc":"2.0",
        "error":{"code":-32600,"message":"invalid request"},
        "id":1
    }))
    .await;
    let err = client.get_block_number().await.unwrap_err();
    match err {
        orbitflare_robinhood_sdk::Error::Rpc { code, message } => {
            assert_eq!(code, -32600);
            assert!(message.contains("invalid request"));
        }
        other => panic!("expected Rpc error, got: {other}"),
    }
}

#[tokio::test]
async fn rpc_error_from_non_200_json_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            json!({"jsonrpc":"2.0","error":{"code":-32404,"message":"IP not whitelisted"}}),
        ))
        .mount(&server)
        .await;
    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();
    let err = client.get_block_number().await.unwrap_err();
    match err {
        orbitflare_robinhood_sdk::Error::Rpc { code, message } => {
            assert_eq!(code, -32404);
            assert!(message.contains("IP not whitelisted"));
        }
        other => panic!("expected Rpc error, got: {other}"),
    }
}

#[tokio::test]
async fn server_error_is_retryable() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
        .mount(&server)
        .await;
    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();
    let err = client.get_block_number().await.unwrap_err();
    assert!(err.is_retryable());
}

#[tokio::test]
async fn rate_limit_429() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429).append_header("Retry-After", "10"))
        .mount(&server)
        .await;
    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    let err = client.get_block_number().await.unwrap_err();
    match err {
        orbitflare_robinhood_sdk::Error::RateLimited { retry_after } => {
            assert_eq!(retry_after, Some(Duration::from_secs(10)));
        }
        other => panic!("expected RateLimited, got: {other}"),
    }
}

#[tokio::test]
async fn failover_to_second_endpoint() {
    let bad = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .mount(&bad)
        .await;

    let good = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","result":"0x2a","id":1})),
        )
        .mount(&good)
        .await;

    let client = RpcClientBuilder::new()
        .urls(&[&bad.uri(), &good.uri()])
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();
    let block = client.get_block_number().await.unwrap();
    assert_eq!(block, 42);
}

#[tokio::test]
async fn failover_on_non_retryable_error() {
    let bad = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            json!({"jsonrpc":"2.0","error":{"code":-32404,"message":"IP not whitelisted"}}),
        ))
        .mount(&bad)
        .await;

    let good = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","result":"0x63","id":1})),
        )
        .mount(&good)
        .await;

    let client = RpcClientBuilder::new()
        .urls(&[&bad.uri(), &good.uri()])
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();
    let block = client.get_block_number().await.unwrap();
    assert_eq!(block, 99);
}

#[tokio::test]
async fn retries_before_failing() {
    let counter = Arc::new(AtomicU32::new(0));
    let counter_clone = counter.clone();

    struct CountingResponder(Arc<AtomicU32>);
    impl Respond for CountingResponder {
        fn respond(&self, _: &wiremock::Request) -> ResponseTemplate {
            let n = self.0.fetch_add(1, Ordering::SeqCst);
            if n < 2 {
                ResponseTemplate::new(500).set_body_string("down")
            } else {
                ResponseTemplate::new(200)
                    .set_body_json(json!({"jsonrpc":"2.0","result":"0x309","id":1}))
            }
        }
    }

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(CountingResponder(counter_clone))
        .mount(&server)
        .await;

    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .retry(RetryPolicy {
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(50),
            multiplier: 1.5,
            max_attempts: 5,
        })
        .build()
        .unwrap();

    let block = client.get_block_number().await.unwrap();
    assert_eq!(block, 777);
    assert!(counter.load(Ordering::SeqCst) >= 3);
}

#[tokio::test]
async fn quarantined_endpoint_skipped_on_second_request() {
    let bad = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .mount(&bad)
        .await;

    let good = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","result":"0x1","id":1})),
        )
        .mount(&good)
        .await;

    let client = RpcClientBuilder::new()
        .urls(&[&bad.uri(), &good.uri()])
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();

    let _ = client.get_block_number().await.unwrap();
    let block = client.get_block_number().await.unwrap();
    assert_eq!(block, 1);
}

#[tokio::test]
async fn error_includes_endpoint_context() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("boom"))
        .mount(&server)
        .await;

    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();

    let err = client.get_block_number().await.unwrap_err();
    let msg = format!("{err}");
    assert!(
        msg.contains("127.0.0.1"),
        "error should contain endpoint: {msg}"
    );
}

#[tokio::test]
async fn api_key_injected_into_url() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","result":"0x1","id":1})),
        )
        .mount(&server)
        .await;

    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .api_key("ORBIT-TEST-KEY")
        .build()
        .unwrap();

    client.get_block_number().await.unwrap();

    let requests = server.received_requests().await.unwrap();
    let req_url = requests[0].url.to_string();
    assert!(
        req_url.contains("api_key=ORBIT-TEST-KEY"),
        "URL should contain api_key: {req_url}"
    );
}

#[tokio::test]
async fn fallback_url_method() {
    let bad = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("down"))
        .mount(&bad)
        .await;

    let good = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","result":"0x37","id":1})),
        )
        .mount(&good)
        .await;

    let client = RpcClientBuilder::new()
        .url(&bad.uri())
        .fallback_url(&good.uri())
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();

    let block = client.get_block_number().await.unwrap();
    assert_eq!(block, 55);
}

#[tokio::test]
async fn timeout_triggers_error() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"jsonrpc":"2.0","result":"0x1","id":1}))
                .set_delay(Duration::from_secs(5)),
        )
        .mount(&server)
        .await;

    let client = RpcClientBuilder::new()
        .url(&server.uri())
        .timeout(Duration::from_millis(100))
        .retry(RetryPolicy {
            max_attempts: 1,
            ..Default::default()
        })
        .build()
        .unwrap();

    let err = client.get_block_number().await.unwrap_err();
    assert!(err.is_retryable());
}
