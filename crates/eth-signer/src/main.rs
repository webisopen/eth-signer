mod config;
mod error;
mod otel;
mod prelude;
mod route;
mod signer;

use axum::{
    Router,
    extract::{MatchedPath, Request},
};
use clap::Parser;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> prelude::Result<()> {
    let args = config::SignerOpts::parse();

    otel::init(args.debug);

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let lisenter = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {}", addr);

    let signer_config: signer::SignerConfig = args.try_into()?;
    tracing::info!("signer config: {:?}", signer_config);

    let routes = route::routes(signer_config.clone());
    let app = Router::new().merge(routes).layer(
        TraceLayer::new_for_http()
            .make_span_with(|req: &Request| {
                let method = req.method();
                let uri = req.uri();
                // axum automatically adds this extension.
                let matched_path = req
                    .extensions()
                    .get::<MatchedPath>()
                    .map(|matched_path| matched_path.as_str());
                tracing::debug_span!("request", %method, %uri, matched_path)
            })
            .on_failure(()),
    );

    axum::serve(
        lisenter,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
