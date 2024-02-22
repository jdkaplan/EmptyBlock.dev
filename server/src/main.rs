use std::env;

use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use eyre::{Context, OptionExt};
use sea_orm::{Database, DatabaseConnection};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

mod orm;

#[cfg(debug_assertions)]
const TRACING_LEVEL: tracing::Level = tracing::Level::DEBUG;

#[cfg(not(debug_assertions))]
const TRACING_LEVEL: tracing::Level = tracing::Level::INFO;

#[derive(FromRef, Clone)]
struct AppState {
    db: DatabaseConnection,
}

type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
struct AppError(#[from] eyre::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An internal server error prevented this request from being handled.",
        )
            .into_response()
    }
}

fn must_env(name: &str) -> eyre::Result<String> {
    env::var(name).wrap_err_with(|| format!("missing required env var: {}", name))
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    init_tracing()?;

    // TODO: Use config-rs or figment when this gets more complicated.
    let db_url = must_env("DATABASE_URL")?;
    let db: DatabaseConnection = Database::connect(db_url).await?;

    let state = AppState { db };

    // TODO: Use clap when this gets more complicated.
    let args: Vec<String> = env::args().collect();
    let static_dir = args
        .get(1)
        .ok_or_eyre("missing required argument: static_dir")?;

    let statics = ServeDir::new(static_dir);
    let app = Router::new()
        .route("/api/hello", get(hello))
        .fallback_service(statics)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = "0.0.0.0:8080";
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    // TODO: Add graceful shutdown to actually see this print.
    tracing::info!("Goodbye! ✌");
    Ok(())
}

async fn hello(State(db): State<DatabaseConnection>) -> AppResult<impl IntoResponse> {
    use orm::prelude::*;
    use sea_orm::prelude::*;
    use sea_orm::query::{QueryOrder, QuerySelect};

    let greeting = Greetings::find()
        .order_by_asc(Expr::cust("random()"))
        .limit(1)
        .one(&db)
        .await
        .wrap_err("random greeting")?;

    println!("{:?}", greeting);

    let Some(greeting) = greeting else {
        tracing::error!("no greetings registered");
        return Ok(String::from("Hi!"));
    };

    Ok(greeting.greeting)
}

fn init_tracing() -> eyre::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(TRACING_LEVEL)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
