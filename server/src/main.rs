use std::env;

use axum::routing::get;
use axum::Router;
use eyre::OptionExt;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[cfg(debug_assertions)]
const TRACING_LEVEL: tracing::Level = tracing::Level::DEBUG;

#[cfg(not(debug_assertions))]
const TRACING_LEVEL: tracing::Level = tracing::Level::INFO;

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_tracing()?;

    // TODO: Use clap when this gets more complicated.
    let args: Vec<String> = env::args().collect();
    let static_dir = args
        .get(1)
        .ok_or_eyre("missing required argument: static_dir")?;

    let statics = ServeDir::new(static_dir);
    let app = Router::new()
        .route("/api/hello", get(|| async { "Hello, World!" }))
        .fallback_service(statics)
        .layer(TraceLayer::new_for_http());

    let addr = "0.0.0.0:8080";
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    // TODO: Add graceful shutdown to actually see this print.
    tracing::info!("Goodbye! ✌");
    Ok(())
}

fn init_tracing() -> eyre::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(TRACING_LEVEL)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
