use std::net::SocketAddr;

use anyhow::{Context, Result};
use axum::{Router, routing::get};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("gateway=debug,tower_http=debug")),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let openai_router = Router::new().route("/", get(openai));
    let gemini_router = Router::new().route("/", get(gemini));
    let anthropic_router = Router::new().route("/", get(anthropic));

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .nest("/openai", openai_router)
        .nest("/gemini", gemini_router)
        .nest("/anthropic", anthropic_router);

    tracing::debug!(?app, "registered routes");

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
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

async fn openai() -> &'static str {
    "OpenAI router"
}

async fn gemini() -> &'static str {
    "Gemini router"
}

async fn anthropic() -> &'static str {
    "Anthropic router"
}
