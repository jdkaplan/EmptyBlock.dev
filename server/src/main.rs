use std::env;
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{FromRef, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::cookie::{Cookie, Key};
use axum_extra::extract::PrivateCookieJar;
use axum_extra::headers::{Header, Referer};
use axum_extra::TypedHeader;
use base64::prelude::*;
use eyre::{Context, OptionExt};
use http::HeaderValue;
use oauth2::{AuthorizationCode, ClientId, ClientSecret, RedirectUrl};
use sea_orm::{Database, DatabaseConnection};
use serde::Deserialize;
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;
use tower_sessions::cookie::SameSite;
use tower_sessions::{Session, SessionManagerLayer};
use tower_sessions_sqlx_store::sqlx::PgPool;
use tower_sessions_sqlx_store::PostgresStore;

use crate::auth::{AuthService, AuthenticateParams};
use crate::recurse::RecurseClient;

const OAUTH_RETURN_KEY: &str = "oauth_return";

const OAUTH_STATE_COOKIE: &str = "ebd_oauth_state";

mod auth;
mod orm;
mod recurse;

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
    auth_svc: AuthService,
    cookie_key: Key,
    http_client: reqwest::Client,
}

struct Globals {
    source_url: String,
    commit_hash: String,
    build_profile: String,
}

type AppResult<T> = Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
enum AppError {
    #[error(transparent)]
    Session(#[from] tower_sessions::session::Error),

    #[error(transparent)]
    Other(#[from] eyre::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self);
        tracing::debug!("{:#}", self);

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
    color_eyre::install()?;
    init_tracing()?;

    // TODO: Use config-rs or figment when this gets more complicated.
    let db_url = must_env("DATABASE_URL")?;

    let db_pool = PgPool::connect(&db_url).await?;
    let db_conn: DatabaseConnection = Database::connect(db_url).await?;

    let rc_api_client_id = must_env("RC_API_CLIENT_ID")?;
    let rc_api_client_secret = must_env("RC_API_CLIENT_SECRET")?;
    let rc_api_redirect_uri = must_env("RC_API_REDIRECT_URI")?;

    let cookie_key = {
        let encoded = must_env("COOKIE_KEY")?;
        let bytes = BASE64_STANDARD.decode(encoded)?;
        Key::from(&bytes)
    };

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

    let session_store = PostgresStore::new(db_pool)
        .with_schema_name("tower_sessions")
        .expect("static schema name")
        .with_table_name("sessions")
        .expect("static table name");

    // Session cookie properties:
    // - The default name is "id", so choose something more descriptive.
    // - SameSite=Lax allows the login flow (OAuth redirect out -> redirect in) to come back with
    //   the same session ID.
    // - Use the same key to sign the cookies.
    let session_layer = SessionManagerLayer::new(session_store)
        .with_name("ebd_session_id")
        .with_same_site(SameSite::Lax)
        .with_signed(cookie_key.clone());

    let http_client = reqwest::Client::new();

    let auth_svc = AuthService {
        db: db_conn.clone(),
        recurse: RecurseClient::new(
            http_client.clone(),
            ClientId::new(rc_api_client_id),
            ClientSecret::new(rc_api_client_secret),
            RedirectUrl::new(rc_api_redirect_uri)?,
        ),
    };

    let app = Router::new()
        .nest(
            "/api",
            Router::new()
                .route("/hello", get(hello))
                .fallback(not_found),
        )
        .route("/oauth/start", get(oauth_start))
        .route("/oauth/callback", get(oauth_callback))
        .route("/about", get(about))
        .layer(session_layer)
        .fallback_service(statics)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            db: db_conn,
            globals,
            auth_svc,
            cookie_key,
            http_client,
        });

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

async fn oauth_start(
    State(auth): State<AuthService>,
    Back { return_path }: Back,
    session: Session,
    cookies: PrivateCookieJar,
) -> impl IntoResponse {
    let (auth_url, oauth_state) = auth.recurse_authorize_url();

    // Keep track of where we should return to afterward.
    if let Some(path) = return_path {
        tracing::warn!({ ?path, ?session }, "return path");
        if let Err(err) = session.insert(OAUTH_RETURN_KEY, path).await {
            tracing::error!({ ?err }, "could not set OAuth return path");
        }
    }

    // Set the OAuth state token (to prevent CSRF) to prove that we started this flow.
    (
        cookies.add(Cookie::new(
            OAUTH_STATE_COOKIE,
            oauth_state.secret().to_owned(),
        )),
        Redirect::to(auth_url.as_str()),
    )
}

#[derive(Debug, Clone, Deserialize)]
struct OauthCallback {
    code: String,
    state: String,
}

async fn oauth_callback(
    State(auth): State<AuthService>,
    session: Session,
    cookies: PrivateCookieJar,
    Query(query): Query<OauthCallback>,
) -> AppResult<impl IntoResponse> {
    let user = auth
        .authenticate(AuthenticateParams {
            code: AuthorizationCode::new(query.code),
            cookie_state: cookies
                .get(OAUTH_STATE_COOKIE)
                .map(|c| oauth2::CsrfToken::new(c.value().to_owned())),
            query_state: oauth2::CsrfToken::new(query.state),
        })
        .await
        .wrap_err("authenticate")?;

    user.start_session(&session).await?;

    match session.get::<String>(OAUTH_RETURN_KEY).await? {
        Some(return_path) => {
            tracing::warn!({ ?return_path, ?session }, "going back to");
            Ok(Redirect::to(&return_path))
        }
        None => {
            tracing::warn!({ ?session }, "no referer to go to?");
            Ok(Redirect::to("/"))
        }
    }
}

pub struct Back {
    return_path: Option<String>,
}

#[axum::async_trait]
impl<S> axum::extract::FromRequestParts<S> for Back
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let value = TypedHeader::<Referer>::from_request_parts(parts, state)
            .await
            .ok();

        let return_path = match value {
            None => None,
            Some(value) => {
                let mut paths = Vec::<HeaderValue>::new();
                value.encode(&mut paths);
                first_nonempty(&paths)
            }
        };

        Ok(Self { return_path })
    }
}

fn first_nonempty(values: &[HeaderValue]) -> Option<String> {
    let v = values.first()?;
    let v = v.to_str().ok()?;

    if v.is_empty() {
        None
    } else {
        Some(v.to_string())
    }
}

fn init_tracing() -> eyre::Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_max_level(TRACING_LEVEL)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
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
