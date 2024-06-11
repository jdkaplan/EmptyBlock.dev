use std::ops::Add;
use std::str::FromStr;

use axum::async_trait;
use axum::extract::FromRequestParts;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use http::request::Parts;
use http::{HeaderName, HeaderValue};
use oauth2::{AccessToken, AuthorizationCode, RefreshToken};
use once_cell::sync::Lazy;
use rand::RngCore;
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};
use tower_sessions::{Expiry, Session};
use url::Url;
use uuid::Uuid;

use crate::recurse;
use crate::recurse::{Profile, RecurseClient};

type TowerSessionsResult<T> = Result<T, tower_sessions::session::Error>;

#[derive(Clone)]
pub struct AuthService {
    pub db: DatabaseConnection,
    pub recurse: RecurseClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub profile: Profile,

    pub access_token: AccessToken,
    pub refresh_token: Option<RefreshToken>,
}

const USER_KEY: &str = "user";

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let session = Session::from_request_parts(req, state).await?;

        // Reset the expiration each time the user comes back.
        let expires_at = OffsetDateTime::now_utc().add(Duration::days(7));
        session.set_expiry(Some(Expiry::AtDateTime(expires_at)));

        match session.get(USER_KEY).await {
            Ok(Some(user)) => Ok(user),
            _ => Err((http::StatusCode::UNAUTHORIZED, "Unauthorized")),
        }
    }
}

impl User {
    pub async fn start_session(&self, session: &Session) -> TowerSessionsResult<()> {
        // Swap the logged-out session's ID for a different one. This avoids session fixation
        // attacks that rely on reuse of the unauthenticated session.
        session.cycle_id().await?;

        session.insert(USER_KEY, self.clone()).await?;

        let csrf_token = CsrfToken::new();
        session.insert(CSRF_TOKEN_KEY, csrf_token).await?;

        Ok(())
    }
}

const CSRF_TOKEN_KEY: &str = "csrf_token";
const CSRF_TOKEN_HEADER: &str = "X-Csrf-Token";

#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
pub struct CsrfToken(String);

impl PartialEq for CsrfToken {
    fn eq(&self, other: &Self) -> bool {
        subtle::ConstantTimeEq::ct_eq(self.0.as_bytes(), other.0.as_bytes()).into()
    }
}

impl CsrfToken {
    pub fn new() -> Self {
        let mut bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut bytes);
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub async fn load_from(session: &Session) -> TowerSessionsResult<Option<Self>> {
        session.get(CSRF_TOKEN_KEY).await
    }
}

impl axum_extra::headers::Header for CsrfToken {
    fn name() -> &'static HeaderName {
        static NAME: Lazy<HeaderName> =
            Lazy::new(|| HeaderName::from_str("X-Csrf-Token").expect("const str"));
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum_extra::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i HeaderValue>,
    {
        let value = values
            .next()
            .ok_or_else(axum_extra::headers::Error::invalid)?;

        let value = value
            .to_str()
            .map_err(|_| axum_extra::headers::Error::invalid())?;

        Ok(Self(value.to_owned()))
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        if let Ok(value) = HeaderValue::from_str(&self.0) {
            values.extend(std::iter::once(value))
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for CsrfToken
where
    S: Send + Sync,
{
    type Rejection = (http::StatusCode, &'static str);

    async fn from_request_parts(req: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match req.headers.get(CSRF_TOKEN_HEADER) {
            Some(token) => match token.to_str() {
                Ok(token) => Ok(CsrfToken(token.to_owned())),
                Err(err) => {
                    tracing::warn!({ ?err }, "extract CSRF token header");
                    Err((http::StatusCode::UNAUTHORIZED, "Unauthorized"))
                }
            },
            _ => Err((http::StatusCode::UNAUTHORIZED, "Unauthorized")),
        }
    }
}

#[derive(Debug)]
pub struct AuthenticateParams {
    pub code: AuthorizationCode,
    pub cookie_state: Option<oauth2::CsrfToken>,
    pub query_state: oauth2::CsrfToken,
}

#[derive(Debug, thiserror::Error)]
pub enum AuthenticateError {
    #[error("invalid CSRF state")]
    Csrf {
        query: oauth2::CsrfToken,
        cookie: Option<oauth2::CsrfToken>,
    },

    #[error(transparent)]
    Recurse(#[from] recurse::Error),

    #[error(transparent)]
    SeaOrm(#[from] sea_orm::DbErr),
}

#[derive(Debug, thiserror::Error)]
pub enum RefreshError {
    #[error(transparent)]
    Recurse(#[from] crate::recurse::Error),

    #[error(transparent)]
    SeaOrm(#[from] sea_orm::DbErr),
}

impl AuthService {
    pub fn recurse_authorize_url(&self) -> (Url, oauth2::CsrfToken) {
        self.recurse.authorize_url()
    }

    pub async fn authenticate(&self, req: AuthenticateParams) -> Result<User, AuthenticateError> {
        let Some(cookie_state) = req.cookie_state else {
            return Err(AuthenticateError::Csrf {
                query: req.query_state,
                cookie: req.cookie_state,
            });
        };

        if req.query_state.secret() != cookie_state.secret() {
            return Err(AuthenticateError::Csrf {
                query: req.query_state,
                cookie: Some(cookie_state),
            });
        }

        let auth = self.recurse.authenticate(req.code).await?;

        let user = upsert_user(&self.db, &auth.profile).await?;

        Ok(User {
            id: user.id,

            profile: auth.profile,
            access_token: auth.access_token,
            refresh_token: auth.refresh_token,
        })
    }

    pub async fn refresh(&self, refresh_token: RefreshToken) -> Result<User, RefreshError> {
        let auth = self.recurse.refresh(refresh_token).await?;

        let user = upsert_user(&self.db, &auth.profile).await?;

        Ok(User {
            id: user.id,

            profile: auth.profile,
            access_token: auth.access_token,
            refresh_token: auth.refresh_token,
        })
    }

    async fn get_user(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<crate::orm::users::Model>, sea_orm::DbErr> {
        use crate::orm::prelude::*;
        use sea_orm::prelude::*;

        Users::find_by_id(*user_id).one(&self.db).await
    }
}

async fn upsert_user(
    db: &DatabaseConnection,
    profile: &Profile,
) -> Result<crate::orm::users::Model, sea_orm::DbErr> {
    use sea_orm::prelude::*;
    use sea_orm::ActiveValue;
    use sea_query::OnConflict;

    use crate::orm::prelude::*;
    use crate::orm::users;

    let user = users::ActiveModel {
        recurse_user_id: ActiveValue::Set(profile.id),
        ..Default::default()
    };

    let _ = Users::insert(user)
        .on_conflict(OnConflict::new().do_nothing().to_owned())
        .exec_without_returning(db)
        .await?;

    let user = Users::find()
        .filter(users::Column::RecurseUserId.eq(profile.id))
        .one(db)
        .await?
        .expect("upsert succeeded");

    Ok(user)
}
