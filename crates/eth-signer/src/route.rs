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
use serde_json::Value;
use tracing::info;

use axum::{
    Router,
    body::Bytes,
    extract::{Json, State},
    response::IntoResponse,
    routing::{get, post},
};

const SIGN_TX_METHOD: &str = "eth_signTransaction";
const HEALTH_STATUS: &str = "health_status";

type Params = Vec<Box<serde_json::value::RawValue>>;

async fn pub_key(config: State<SignerConfig>) -> Result<String> {
    let addr = config.address().await?;
    Ok(addr.to_string())
}

fn fix_missing_params(original_bytes: Bytes) -> Bytes {
    let mut value: Value = match serde_json::from_slice(&original_bytes) {
        Ok(v) => v,
        Err(_) => return original_bytes,
    };

    if let Value::Object(map) = &mut value {
        if !map.contains_key("params") {
            map.insert("params".to_string(), Value::Array(vec![]));
        }
    } else {
        return original_bytes;
    }

    let fixed_bytes = match serde_json::to_vec(&value) {
        Ok(b) => b,
        Err(_) => return original_bytes,
    };

    Bytes::from(fixed_bytes)
}

async fn rpc_request(config: State<SignerConfig>, raw_body: Bytes) -> impl IntoResponse {
    let fixed_bytes = fix_missing_params(raw_body);
    let request: JrpcRequest<Params> = serde_json::from_slice(&fixed_bytes).unwrap();

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
    JrpcRequest { meta, params }: JrpcRequest<Params>,
) -> JrpcResponse {
    let span = tracing::debug_span!("rpc", method = %meta.method, id = %meta.id);
    let _guard = span.enter();

    match meta.method.as_ref() {
        SIGN_TX_METHOD => {
            let Some(raw) = params.first() else {
                tracing::error!("invalid params");
                return JrpcResponse::invalid_params(meta.id.clone());
            };

            let request: TransactionRequest = match serde_json::from_str(raw.get()) {
                Ok(req) => req,
                Err(e) => {
                    tracing::error!("invalid params, deserialize error: {}", e);
                    return JrpcResponse::invalid_params(meta.id.clone());
                }
            };

            JrpcResponse {
                id: meta.id,
                payload: match sign(config, request).await {
                    Ok(result) => ResponsePayload::Success(result),
                    Err(e) => {
                        tracing::error!("sign error: {}", e);
                        ResponsePayload::Failure(e.into())
                    }
                },
            }
        }
        HEALTH_STATUS => JrpcResponse {
            id: meta.id.clone(),
            payload: ResponsePayload::Success(*Box::new(
                serde_json::value::RawValue::from_string(
                    serde_json::to_string(&"ok").expect("failed to serialize"),
                )
                .expect("failed to create raw value"),
            )),
        },
        _ => {
            tracing::error!("invalid method");
            JrpcResponse::method_not_found(meta.id.clone())
        }
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
