use std::hash::{DefaultHasher, Hash, Hasher};

use crate::error::RPCResult;
use crate::prelude::*;
use crate::signer::SignerConfig;
use alloy::{
    eips::eip2718::Encodable2718,
    network::TransactionBuilder,
    primitives::TxKind,
    rpc::types::{TransactionInput, TransactionRequest},
};
use tracing::info;

use axum::{
    Router,
    extract::{Json, State},
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

async fn pub_key(config: State<SignerConfig>) -> Result<String> {
    let addr = config.address().await?;
    Ok(addr.to_string())
}

#[derive(Debug, Deserialize)]
struct SignRequest {
    id: u64,
    jsonrpc: String,
    method: String,
    params: [TransactionRequest; 1],
}

#[derive(Debug, Serialize)]
struct SignReponse {
    id: u64,
    jsonrpc: String,
    result: String,
}

async fn sign(
    config: State<SignerConfig>,
    Json(SignRequest {
        id,
        jsonrpc,
        method,
        params: [request],
    }): Json<SignRequest>,
) -> RPCResult<Json<SignReponse>> {
    let rpc_err_map = |e: Error| e.rpc_error(id, jsonrpc.clone());

    {
        let TransactionRequest {
            from, to, input, ..
        } = request.clone();
        let TransactionInput { input, data } = input;
        let input = input.map(|i| i.to_string());
        let data = data.map(|d| d.to_string());

        let from = from.unwrap_or_default();
        let to = to.map(|kind| match kind {
            TxKind::Create => String::from("create"),
            TxKind::Call(addr) => addr.to_string(),
        });
        info!(from = from.to_string(), to, input, data, "sign request");
    };

    if method != "eth_signTransaction" {
        return Err(rpc_err_map(Error::InvalidRpcMethod(method)));
    }

    let mut req_hash = DefaultHasher::new();
    request.clone().hash(&mut req_hash);

    let wallet = config.wallet().await.map_err(rpc_err_map)?;
    let tx_envelop = request
        .build(&wallet)
        .await
        .map_err(Error::TransactionBuilder)
        .map_err(rpc_err_map)?;

    let mut tx_hash = DefaultHasher::new();
    tx_envelop.tx_hash().hash(&mut tx_hash);

    info!(
        req_hash = req_hash.finish(),
        tx_hash = tx_hash.finish(),
        "sign tx"
    );

    let mut encoded_tx = Vec::<u8>::new();

    tx_envelop.encode_2718(&mut encoded_tx);

    let hex_string: String = encoded_tx.iter().map(|b| format!("{:02x?}", b)).collect();

    Ok(Json(SignReponse {
        id,
        jsonrpc,
        result: format!("0x{}", hex_string),
    }))
}

pub fn routes(state: SignerConfig) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "OK" }))
        .route("/pub", get(pub_key))
        .route("/", post(sign))
        .with_state(state)
}
