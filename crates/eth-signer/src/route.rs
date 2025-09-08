use std::hash::{DefaultHasher, Hash};

use crate::prelude::*;
use crate::signer::SignerConfig;
use alloy::{
    eips::eip2718::Encodable2718,
    network::TransactionBuilder,
    primitives::TxKind,
    rpc::{
        json_rpc::{Request as JrpcRequest, Response as JrpcResponse, ResponsePayload},
        types::{TransactionInput, TransactionRequest},
    },
};
use tracing::info;

use axum::{
    Router,
    extract::{Json, State},
    response::IntoResponse,
    routing::{get, post},
};

const SIGN_TX_METHOD: &str = "eth_signTransaction";

async fn pub_key(config: State<SignerConfig>) -> Result<String> {
    let addr = config.address().await?;
    Ok(addr.to_string())
}

async fn rpc_request(
    config: State<SignerConfig>,
    Json(request): Json<JrpcRequest<[TransactionRequest; 1]>>,
) -> impl IntoResponse {
    let response = rpc(config, request).await;
    if response.is_success() {
        (axum::http::StatusCode::OK, Json(response))
    } else {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(response),
        )
    }
}

async fn rpc(
    config: State<SignerConfig>,
    JrpcRequest { meta, params }: JrpcRequest<[TransactionRequest; 1]>,
) -> JrpcResponse {
    let span = tracing::debug_span!("rpc", method = %meta.method, id = %meta.id);
    let _guard = span.enter();
    if meta.method != SIGN_TX_METHOD {
        tracing::error!("invalid method");
        return JrpcResponse::method_not_found(meta.id);
    }

    let Some(request) = params.first() else {
        tracing::error!("invalid params");
        return JrpcResponse::invalid_params(meta.id);
    };

    JrpcResponse {
        id: meta.id,
        payload: match sign(config, request.clone()).await {
            Ok(result) => ResponsePayload::Success(result),
            Err(e) => {
                tracing::error!("sign error: {}", e);
                ResponsePayload::Failure(e.into())
            }
        },
    }
}

async fn sign(
    config: State<SignerConfig>,
    request: TransactionRequest,
) -> Result<Box<serde_json::value::RawValue>> {
    let TransactionRequest {
        from, to, input, ..
    } = request.clone();
    let TransactionInput { input, data } = input;
    let input = input.map(|i| i.to_string());
    let data = data.map(|d| d.to_string());

    let to = to.map(|kind| match kind {
        TxKind::Create => String::from("create"),
        TxKind::Call(addr) => addr.to_string(),
    });
    let span = tracing::info_span!(
        "sign",
        from = %from.unwrap_or_default(),
        to = %to.unwrap_or_default(),
        input = %input.unwrap_or_default(),
        data = %data.unwrap_or_default()
    );
    let _guard = span.enter();

    let mut req_hash = DefaultHasher::new();
    request.clone().hash(&mut req_hash);

    let wallet = config.wallet().await?;

    let tx_envelop = request.build(&wallet).await?;

    let mut tx_hash = DefaultHasher::new();
    tx_envelop.tx_hash().hash(&mut tx_hash);

    let mut encoded_tx = Vec::<u8>::new();

    tx_envelop.encode_2718(&mut encoded_tx);

    let hex_string: String = encoded_tx.iter().map(|b| format!("{:02x?}", b)).collect();

    let raw_string = format!("0x{}", hex_string);

    info!(hex = %raw_string, "sign tx");

    Ok(serde_json::value::RawValue::from_string(
        serde_json::to_string(&raw_string)?,
    )?)
}

pub fn routes(state: SignerConfig) -> Router {
    Router::new()
        .route("/healthz", get(|| async { "OK" }))
        .route("/pub", get(pub_key))
        .route("/", post(rpc_request))
        .with_state(state)
}
