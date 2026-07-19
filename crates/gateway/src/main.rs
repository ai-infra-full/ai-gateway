use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{
    Router,
    extract::Request,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    routing::post,
};
use tower::ServiceExt;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::gateway::GatewayService;

mod gateway;
mod provider;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("gateway=debug,tower_http=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let openai_router = Router::new().route("/", post(handler));
    let gemini_router = Router::new().route("/", post(handler));
    let anthropic_router = Router::new().route("/", post(handler));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/openai", openai_router)
        .nest("/gemini", gemini_router)
        .nest("/anthropic", anthropic_router)
        .with_state(GatewayService::default());

    tracing::debug!(?app, "registered routes");

    let address = std::env::var("GATEWAY_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_owned())
        .parse::<SocketAddr>()
        .context("invalid GATEWAY_ADDRESS")?;
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .with_context(|| format!("failed to bind gateway to {address}"))?;

    tracing::info!(%address, "gateway listening");
    axum::serve(listener, app)
        .await
        .context("gateway server failed")?;

    Ok(())
}

async fn root() -> &'static str {
    "AI Gateway"
}

async fn health() -> &'static str {
    "ok"
}

async fn handler(State(gateway): State<GatewayService>, request: Request) -> Response {
    match gateway.oneshot(request).await {
        Ok(response) => response,
        Err(error) => {
            tracing::error!(%error, "gateway request failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
        }
    }
}
