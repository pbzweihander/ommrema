use chrono::{Duration, Utc};
use oauth2::TokenResponse;
use once_cell::sync::Lazy;
use poem::{
    http::{header, HeaderMap, StatusCode},
    web::{headers, Query, Redirect, TypedHeader},
    FromRequest, IntoResponse, Route,
};
use serde::{Deserialize, Serialize};

use crate::{
    config::{CONFIG, HTTP_CLIENT},
    handler::error::WrapRespErr,
};

static OAUTH_CLIENT: Lazy<oauth2::basic::BasicClient> = Lazy::new(|| {
    oauth2::basic::BasicClient::new(
        oauth2::ClientId::new(CONFIG.discord_client_id.clone()),
        Some(oauth2::ClientSecret::new(
            CONFIG.discord_client_secret.clone(),
        )),
        oauth2::AuthUrl::new("https://discord.com/oauth2/authorize".to_string()).unwrap(),
        Some(oauth2::TokenUrl::new("https://discord.com/api/oauth2/token".to_string()).unwrap()),
    )
    .set_redirect_uri(oauth2::RedirectUrl::from_url(
        CONFIG.public_url.join("auth/authorized").unwrap(),
    ))
});

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    pub username: String,
    pub exp: i64,
}

impl<'a> FromRequest<'a> for User {
    async fn from_request(
        req: &'a poem::Request,
        body: &mut poem::RequestBody,
    ) -> poem::Result<Self> {
        let cookies = TypedHeader::<headers::Cookie>::from_request(req, body).await?;
        let session_cookie = cookies
            .get("session")
            .ok_or_else(|| poem::Error::from_response(Redirect::see_other("/").into_response()))?;

        let mut jwt_validation = jsonwebtoken::Validation::default();
        jwt_validation.validate_exp = true;
        let user_data =
            jsonwebtoken::decode::<User>(session_cookie, &CONFIG.jwt_secret.1, &jwt_validation)
                .map_err(|_| {
                    poem::Error::from_response(Redirect::see_other("/").into_response())
                })?;
        let user = user_data.claims;
        Ok(user)
    }
}

#[poem::handler]
#[tracing::instrument]
async fn get_auth_redirect() -> Redirect {
    let (auth_url, _csrf_token) = OAUTH_CLIENT
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("identify".to_string()))
        .add_scope(oauth2::Scope::new("guilds.members.read".to_string()))
        .url();
    Redirect::see_other(auth_url)
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    username: String,
}

#[derive(Deserialize)]
struct DiscordGuildMember {
    roles: Vec<String>,
}

#[poem::handler]
#[tracing::instrument]
async fn get_authorized(
    Query(req): Query<AuthRequest>,
) -> Result<(HeaderMap, Redirect), (StatusCode, eyre::Report)> {
    let token = OAUTH_CLIENT
        .exchange_code(oauth2::AuthorizationCode::new(req.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
        .wrap_resp_err(StatusCode::BAD_REQUEST, "failed to authorize")?;
    let access_token = token.access_token().secret();

    let resp = HTTP_CLIENT
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token)
        .send()
        .await
        .wrap_resp_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to request Discord",
        )?;
    let resp = resp.error_for_status().wrap_resp_err(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Discord returned error response from user API",
    )?;
    let resp = resp.bytes().await.wrap_resp_err(
        StatusCode::INTERNAL_SERVER_ERROR,
        "failed to read Discord user API response",
    )?;
    let user = serde_json::from_slice::<DiscordUser>(resp.as_ref()).wrap_resp_err(
        StatusCode::INTERNAL_SERVER_ERROR,
        "failed to parse Discord user API response",
    )?;

    let resp = HTTP_CLIENT
        .get(format!(
            "https://discord.com/api/users/@me/guilds/{}/member",
            CONFIG.discord_guild_id
        ))
        .bearer_auth(access_token)
        .send()
        .await
        .wrap_resp_err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "failed to request Discord",
        )?;
    let resp = resp.error_for_status().wrap_resp_err(
        StatusCode::UNAUTHORIZED,
        "Discord return error response from guild API",
    )?;
    let resp = resp.bytes().await.wrap_resp_err(
        StatusCode::INTERNAL_SERVER_ERROR,
        "failed to read Discord guild API response",
    )?;
    let guild_member = serde_json::from_slice::<DiscordGuildMember>(resp.as_ref()).wrap_resp_err(
        StatusCode::UNAUTHORIZED,
        "failed to parse Discord guild API response",
    )?;

    if !guild_member.roles.contains(&CONFIG.discord_guild_role_id) {
        return Err((
            StatusCode::UNAUTHORIZED,
            eyre::eyre!("You does not have desired role"),
        ));
    }

    let now = Utc::now();
    let exp = (now + Duration::days(1)).timestamp();

    let user = User {
        username: user.username,
        exp,
    };

    let session_token =
        jsonwebtoken::encode(&Default::default(), &user, &CONFIG.jwt_secret.0).unwrap();

    let cookie = format!(
        "session={}; SameSite=Lax; Path=/; Domain={}",
        session_token,
        CONFIG.public_url.domain().unwrap()
    );

    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, cookie.parse().unwrap());

    Ok((headers, Redirect::see_other("/")))
}

pub fn create_route() -> Route {
    Route::new()
        .at("/", poem::get(get_auth_redirect))
        .at("/authorized", poem::get(get_authorized))
}
