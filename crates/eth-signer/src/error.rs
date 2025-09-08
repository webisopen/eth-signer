use alloy::{
    network::Ethereum,
    rpc::json_rpc::ErrorPayload,
    signers::{
        aws::AwsSignerError,
        gcp::{GcpSignerError, gcloud_sdk},
        local::LocalSignerError,
    },
};
use axum::response::{IntoResponse, Response};

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

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error("Invalid signer type '{0}'")]
    InvalidSignerType(String),

    #[error("Require config key '{0}' not found")]
    RequireConfigKeyNotFound(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            self.to_string(),
        )
            .into_response()
    }
}

impl From<Error> for ErrorPayload {
    fn from(error: Error) -> ErrorPayload {
        ErrorPayload::internal_error_message(error.to_string().into())
    }
}
