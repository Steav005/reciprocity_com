use oauth2::basic::{BasicClient, BasicErrorResponse};

use oauth2::reqwest::async_http_client;
use oauth2::{AuthUrl, ClientId, ClientSecret, TokenResponse, TokenUrl, RefreshToken, AccessToken, RequestTokenError, RedirectUrl, HttpRequest};
use oauth2::url::{ParseError, Url};
use oauth2::http::{Method, HeaderMap, StatusCode};
use oauth2::http::header::InvalidHeaderValue;
use crate::messages::{User, Auth};
pub use reqwest;

#[derive(Debug)]
pub enum OAuthError {
    ParseError(ParseError),
    RequestError(RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, BasicErrorResponse>),
    RequestErrorApi(oauth2::reqwest::Error<reqwest::Error>),
    ResponseError(StatusCode),
    NoRefreshTokenInResponse(),
    UserParseError(serde_json::Error),
    InvalidHeader(InvalidHeaderValue),
}

impl From<ParseError> for OAuthError {
    fn from(e: ParseError) -> Self {
        OAuthError::ParseError(e)
    }
}

impl From<RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, BasicErrorResponse>> for OAuthError {
    fn from(e: RequestTokenError<oauth2::reqwest::Error<reqwest::Error>, BasicErrorResponse>) -> Self {
        OAuthError::RequestError(e)
    }
}

impl From<oauth2::reqwest::Error<reqwest::Error>> for OAuthError {
    fn from(e: oauth2::reqwest::Error<reqwest::Error>) -> Self {
        OAuthError::RequestErrorApi(e)
    }
}

impl From<InvalidHeaderValue> for OAuthError{
    fn from(e: InvalidHeaderValue) -> Self {
        OAuthError::InvalidHeader(e)
    }
}

impl From<serde_json::Error> for OAuthError{
    fn from(e: serde_json::Error) -> Self {
        OAuthError::UserParseError(e)
    }
}

pub async fn get_token(access: Auth) -> Result<(AccessToken, RefreshToken), OAuthError> {
    //TODO LOG
    //TODO Get Values from Config

    let client_id = ClientId::new("815279319513432134".to_string());
    let secret = ClientSecret::new("Qv-fhG16dbe7zAY8GcMia73Tb8_za87G".to_string());
    let auth_url = AuthUrl::new("https://discord.com/api/oauth2/authorize".to_string())?;
    let token_url = TokenUrl::new("https://discord.com/api/oauth2/token".to_string())?;

    let client = BasicClient::new(client_id, Some(secret), auth_url, Some(token_url))
        .set_redirect_uri(RedirectUrl::new("http://localhost:1887".to_string())?);

    let resp = match access {
        Auth::Token(re) => {
            client.exchange_refresh_token(&re).request_async(async_http_client).await?
        }
        Auth::Code(c) => {
            client.exchange_code(c).request_async(async_http_client).await?
        }
    };

    let access = resp.access_token().clone();
    let refresh = resp.refresh_token().ok_or(OAuthError::NoRefreshTokenInResponse())?.clone();

    Ok((access, refresh))
}

pub async fn get_user_id(token: AccessToken) -> Result<User, OAuthError>{
    //TODO Log
    //TODO Get values from config

    let mut headers = HeaderMap::new();
    headers.insert("Authorization", format!("Bearer {}", token.secret()).parse()?);

    let req = HttpRequest{
        url: Url::parse("https://discordapp.com/api/users/@me")?,
        method: Method::GET,
        headers,
        body: vec![]
    };

    let resp = async_http_client(req).await?;
    if resp.status_code.as_u16() != 200_u16{
        return Err(OAuthError::ResponseError(resp.status_code));
    }
    let user: User = serde_json::from_slice(resp.body.as_slice())?;

    Ok(user)
}