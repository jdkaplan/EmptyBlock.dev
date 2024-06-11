use gloo::net::http::Request;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::types::CsrfToken;

// TODO: Extract shared web+server types to a third crate.

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub user_id: Uuid,
    pub profile: Profile,

    pub csrf_token: CsrfToken,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
}

impl Session {
    pub async fn load() -> eyre::Result<Option<Self>> {
        let res = Request::get("/session").send().await?;

        if res.status() == StatusCode::UNAUTHORIZED {
            tracing::warn!({ ?res }, "no user logged in");
            return Ok(None);
        }

        if !res.ok() {
            tracing::error!({ ?res }, "whoami");
            return Ok(None);
        }

        let user: Session = res.json().await?;
        Ok(Some(user))
    }

    pub async fn load_ok() -> Option<Self> {
        match Self::load().await {
            Ok(v) => v,
            Err(err) => {
                tracing::debug!({ ?err }, "load User");
                None
            }
        }
    }
}
