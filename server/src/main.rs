use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{FromRef, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use eyre::{Context, OptionExt};
use sea_orm::{Database, DatabaseConnection};
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

mod orm;

const COMMIT_HASH: &str = include_str!(concat!(env!("OUT_DIR"), "/commit_hash"));
const SOURCE_URL: &str = include_str!(concat!(env!("OUT_DIR"), "/source_url"));
const BUILD_PROFILE: &str = include_str!(concat!(env!("OUT_DIR"), "/build_profile"));

#[cfg(debug_assertions)]
const TRACING_LEVEL: tracing::Level = tracing::Level::DEBUG;

#[cfg(not(debug_assertions))]
const TRACING_LEVEL: tracing::Level = tracing::Level::INFO;

#[derive(FromRef, Clone)]
struct AppState {
    globals: Arc<Globals>,
    db: DatabaseConnection,
}

struct Globals {
    source_url: String,
    commit_hash: String,
    build_profile: String,
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

    let globals = Arc::new(Globals {
        source_url: String::from(SOURCE_URL),
        commit_hash: String::from(COMMIT_HASH),
        build_profile: String::from(BUILD_PROFILE),
    });

    // TODO: Use clap when this gets more complicated.
    let args: Vec<String> = env::args().collect();
    let static_dir = args
        .get(1)
        .map(PathBuf::from)
        .ok_or_eyre("missing required argument: static_dir")?;
    let spa = static_dir.join("index.html");

    let statics = ServeDir::new(static_dir).fallback(ServeFile::new(spa));

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/hello", get(hello))
                .fallback(not_found),
        )
        .route("/about", get(about))
        .fallback_service(statics)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { db, globals });

    let addr = "0.0.0.0:8080";
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Goodbye! ✌");
    Ok(())
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
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

async fn about(State(globals): State<Arc<Globals>>) -> impl IntoResponse {
    let page = markup::new! {
        @markup::doctype()
        html [lang="en"] {
            head {
                meta [charset="utf-8"];
                meta [name="viewport", content="width=device-width,initial-scale=1"];

                title { "About EmptyBlock.dev" }
            }
            body {
                h1 { "About" }
                p {
                    a [href="/"] { "EmptyBlock.dev" }
                    " is a web development playground. The dream is to have a collection of apps that are somehow useful, interesting, or fun to work on."
                }
                p { "It'll get there eventually, I'm sure 😎" }
                p {
                    "Code for this site is available under the terms of the "
                    a [href="https://blueoakcouncil.org/license/1.0.0"] { "Blue Oak Model License" }
                    ". Check out the "
                    a [href = {&globals.source_url}] { "source code" }
                    "!"
                }

                h2 { "Third-party software" }
                p { "This site is proudly built on top of free and open source software. Thank you to everyone who has contributed to the frameworks, libraries, tools, and everything else that makes it possible to create this." }

                p {
                    a [href="/third_party_licenses"] {
                        "View all third-party licenses"
                    }
                }

                h2 { "Build info" }
                p {
                    {&globals.commit_hash} " (" {&globals.build_profile} ")"
                }
            }
        }
    };

    axum::response::Html(page.to_string())
}

fn init_tracing() -> eyre::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(TRACING_LEVEL)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("Signal received, starting graceful shutdown");
}
