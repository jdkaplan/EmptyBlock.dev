use oauth2::basic::{BasicClient, BasicTokenType};
use oauth2::reqwest::async_http_client;
use oauth2::{
    AccessToken, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EmptyExtraTokenFields, RedirectUrl, RefreshToken, RequestTokenError, StandardTokenResponse,
    TokenResponse, TokenUrl,
};
use serde::{Deserialize, Serialize};
use url::Url;

const RC_API_AUTHORIZE_URL: &str = "https://www.recurse.com/oauth/authorize";
const RC_API_TOKEN_URL: &str = "https://www.recurse.com/oauth/token";

#[derive(Debug, Clone)]
pub struct RecurseClient {
    pub http: reqwest::Client,
    pub oauth: BasicClient,
}

impl RecurseClient {
    pub fn new(
        http_client: reqwest::Client,
        client_id: ClientId,
        client_secret: ClientSecret,
        redirect_url: RedirectUrl,
    ) -> Self {
        let oauth = BasicClient::new(
            client_id,
            Some(client_secret),
            AuthUrl::new(String::from(RC_API_AUTHORIZE_URL)).expect("const URL"),
            Some(TokenUrl::new(String::from(RC_API_TOKEN_URL)).expect("const URL")),
        )
        .set_redirect_uri(redirect_url);

        Self {
            http: http_client,
            oauth,
        }
    }

    pub fn authorize_url(&self) -> (Url, CsrfToken) {
        self.oauth.authorize_url(CsrfToken::new_random).url()
    }
}

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Oauth2(
        #[from]
        RequestTokenError<
            oauth2::reqwest::AsyncHttpClientError,
            oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>,
        >,
    ),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
}

pub struct User {
    pub access_token: AccessToken,
    pub refresh_token: Option<RefreshToken>,

    pub profile: Profile,
}

impl User {
    fn new(
        resp: StandardTokenResponse<EmptyExtraTokenFields, BasicTokenType>,
        profile: Profile,
    ) -> Self {
        Self {
            access_token: resp.access_token().clone(),
            refresh_token: resp.refresh_token().cloned(),
            profile,
        }
    }
}

impl RecurseClient {
    pub async fn authenticate(&self, code: AuthorizationCode) -> Result<User, Error> {
        let resp = self
            .oauth
            .exchange_code(code)
            .request_async(async_http_client)
            .await?;

        let profile = get_profile(&self.http, resp.access_token()).await?;

        Ok(User::new(resp, profile))
    }
}

impl RecurseClient {
    pub async fn refresh(&self, refresh_token: RefreshToken) -> Result<User, Error> {
        let resp = self
            .oauth
            .exchange_refresh_token(&refresh_token)
            .request_async(async_http_client)
            .await?;

        let profile = get_profile(&self.http, resp.access_token()).await?;

        Ok(User::new(resp, profile))
    }
}

// https://github.com/recursecenter/wiki/wiki/Recurse-Center-API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: i64,
    pub name: String,
}

pub async fn get_profile(
    http_client: &reqwest::Client,
    access_token: &oauth2::AccessToken,
) -> reqwest::Result<Profile> {
    let resp = http_client
        .get("https://www.recurse.com/api/v1/profiles/me")
        .bearer_auth(access_token.secret())
        .send()
        .await?;

    let profile = resp.json::<Profile>().await?;
    Ok(profile)
}
