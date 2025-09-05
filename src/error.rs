use alloy::{
    network::Ethereum,
    signers::{
        aws::AwsSignerError,
        gcp::{GcpSignerError, gcloud_sdk},
        local::LocalSignerError,
    },
};
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    AlloySigner(#[from] alloy::signers::Error),

    #[error(transparent)]
    AlloyLocalSigner(#[from] LocalSignerError),

    #[error(transparent)]
    AwsSigner(#[from] Box<AwsSignerError>),

    #[error(transparent)]
    GcpSigner(#[from] Box<GcpSignerError>),

    #[error(transparent)]
    GcloudSDK(#[from] gcloud_sdk::error::Error),

    #[error(transparent)]
    TransactionBuilder(#[from] alloy::network::TransactionBuilderError<Ethereum>),

    #[error("Invalid signer type '{0}'")]
    InvalidSignerType(String),

    #[error("Require config key '{0}' not found")]
    RequireConfigKeyNotFound(&'static str),

    #[error("Invalid rpc method '{0}'")]
    InvalidRpcMethod(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            self.to_string(),
        )
            .into_response()
    }
}

#[derive(Debug)]
pub struct RPCError {
    id: u64,
    jsonrpc: String,
    error: Error,
}

pub type RPCResult<T> = std::result::Result<T, RPCError>;

impl IntoResponse for RPCError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrResponse {
            id: u64,
            jsonrpc: String,
            error: String,
        }

        let err_string = self.error.to_string();
        tracing::info!(err_string);

        {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrResponse {
                    id: self.id,
                    jsonrpc: self.jsonrpc,
                    error: err_string,
                }),
            )
                .into_response()
        }
    }
}

impl Error {
    pub fn rpc_error(self, id: u64, jsonrpc: String) -> RPCError {
        RPCError {
            id,
            jsonrpc,
            error: self,
        }
    }
}
