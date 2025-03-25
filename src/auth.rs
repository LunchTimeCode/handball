use openid::{
    Client, CompactJson, CustomClaims, Discovered, IdToken, Options, StandardClaims, Token,
};
use rocket::{
    fairing::{AdHoc, Fairing},
    Request, State,
};
use std::collections::{HashMap, HashSet};

use crate::Environment;
use anyhow::anyhow;
use base64::Engine;
use rocket::http::{Cookie, CookieJar, SameSite, Status};
use rocket::request::{FromRequest, Outcome};
use rocket::response::Redirect;
use rocket::serde::{Deserialize, Serialize};
use rocket::time::Duration;
use std::env;
use url::Url;

pub type OpenIDClient = Client<Discovered, Claims>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    #[serde(flatten)]
    standard: StandardClaims,
}

impl CompactJson for Claims {}

impl CustomClaims for Claims {
    fn standard_claims(&self) -> &StandardClaims {
        &self.standard
    }
}

const SCOPES: &str = "openid email username";

pub mod cookie_name {
    pub const HANDBALL: &str = "HANDBALL";
}

#[derive(Debug)]
pub struct NotAuthenticated;

#[derive(Debug, Serialize, Deserialize)]
pub struct User;

#[async_trait]
impl<'r> FromRequest<'r> for User {
    type Error = NotAuthenticated;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let user = request.cookies().get(cookie_name::HANDBALL);
        match user {
            Some(_) => Outcome::Success(Self),
            None => {
                debug!("no token cookie found");
                return Outcome::Failure((Status::Unauthorized, NotAuthenticated));
            }
        }
    }
}

pub fn fairing() -> impl Fairing {
    AdHoc::try_on_ignite("openid connect", |rocket| async {
        match discover().await {
            Ok(oidc_client) => Ok(rocket
                .manage(Options {
                    scope: Some(SCOPES.into()),
                    ..Default::default()
                })
                .manage(oidc_client)),
            Err(err) => {
                error!("Failed to discover openid connect provider: {err}");
                Err(rocket)
            }
        }
    })
}

#[derive(Clone)]
struct Config {
    issuer_url: Url,
    redirect_url: Url,
    client_id: String,
    client_secret: String,
}

impl Config {
    fn from_env() -> anyhow::Result<Self> {
        let origin = env::var("ORIGIN")?;
        Ok(Self {
            issuer_url: env::var("AUTH_ISSUER")?.parse()?,
            redirect_url: format!("{origin}/login/finalize").parse()?,
            client_id: env::var("AUTH_CLIENT_ID")?,
            client_secret: env::var("AUTH_CLIENT_SECRET")?,
        })
    }
}

async fn discover() -> anyhow::Result<OpenIDClient> {
    let config = Config::from_env()?;
    let oidc_client = OpenIDClient::discover(
        config.client_id,
        config.client_secret,
        Some(config.redirect_url.to_string()),
        config.issuer_url,
    )
    .await?;
    Ok(oidc_client)
}

#[get("/login?<invitation>&<organization>")]
pub async fn accept_invite(
    oidc_client: &State<OpenIDClient>,
    oidc_options: &State<Options>,
    invitation: Option<&str>,
    organization: Option<&str>,
) -> Redirect {
    let mut url = oidc_client.auth_url(oidc_options);
    if let (Some(invite), Some(org)) = (invitation, organization) {
        url.query_pairs_mut()
            .append_pair("invitation", invite)
            .append_pair("organization", org);
    }
    Redirect::to(url.to_string())
}

#[derive(FromForm)]
pub struct Finalize<'r> {
    code: Option<&'r str>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[get("/login/finalize?<finalize..>")]
pub async fn finalize(
    oidc_client: &State<OpenIDClient>,
    finalize: Finalize<'_>,
    cookies: &CookieJar<'_>,
    environment: &State<Environment>,
) -> Result<Redirect, Status> {
    let Some(code) = finalize.code else {
        if finalize.error.is_some() && finalize.error_description.is_some() {
            return Ok(Redirect::to("/error/email_not_verified"));
        }
        return Err(Status::Unauthorized);
    };
    let token = match request_token(oidc_client, code).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            error!("no id_token found");
            return Err(Status::Unauthorized);
        }
        Err(err) => {
            warn!("{}", err);
            return Err(Status::Unauthorized);
        }
    };
    let Some(IdToken::Decoded { .. }) = token.id_token else {
        unreachable!("token must be decoded at this point");
    };
    let cookie = Cookie::build(cookie_name::HANDBALL, "");
    cookies.add(
        cookie
            .same_site(SameSite::Lax)
            .max_age(Duration::seconds(20))
            .finish(),
    );
    Ok(Redirect::to(
        finalize.state.unwrap_or_else(|| String::from("/")),
    ))
}

async fn request_token(
    oidc_client: &OpenIDClient,
    code: &str,
) -> anyhow::Result<Option<Token<Claims>>> {
    let mut token: Token<Claims> = oidc_client.request_token(code).await?.into();
    match token.id_token.as_mut() {
        Some(id_token) => {
            oidc_client
                .decode_token(id_token)
                .map_err(|err| anyhow!("cannot decode token: {err}"))?;
            oidc_client
                .validate_token(id_token, None, None)
                .map_err(|err| anyhow!("invalid token: {err}"))?;
        }
        None => return Ok(None),
    }
    Ok(Some(token))
}
