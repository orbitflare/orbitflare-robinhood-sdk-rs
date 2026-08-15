mod credentials;
mod endpoint;

pub mod error;
pub mod retry;

#[cfg(feature = "rpc")]
pub mod rpc;

#[cfg(feature = "ws")]
pub mod ws;

pub use error::{Error, Result};
pub use retry::RetryPolicy;

#[cfg(feature = "rpc")]
pub use rpc::{Block, RpcClient, RpcClientBuilder, Transaction, TransactionReceipt};

#[cfg(feature = "ws")]
pub use ws::{WsClient, WsClientBuilder, WsSubscription};

#[cfg(feature = "rpc")]
pub use alloy_network::{ReceiptResponse, TransactionResponse};

#[cfg(feature = "rpc")]
pub use alloy_primitives as primitives;
#[cfg(feature = "rpc")]
pub use alloy_rpc_types_eth as rpc_types;

#[cfg(feature = "rpc")]
pub use alloy_primitives::{Address, B256, Bytes, U256};
#[cfg(feature = "rpc")]
pub use alloy_rpc_types_eth::{
    BlockNumberOrTag, FeeHistory, Filter, Header, Log, TransactionRequest,
};
