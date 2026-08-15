#![cfg(feature = "ws")]

use orbitflare_robinhood_sdk::WsClientBuilder;

#[tokio::test]
async fn connect_fails_on_dead_endpoint() {
    let err = WsClientBuilder::new()
        .url("ws://localhost:1")
        .build()
        .await
        .err()
        .expect("expected error");
    assert!(matches!(err, orbitflare_robinhood_sdk::Error::Transport(_)));
}
