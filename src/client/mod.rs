use oauth2::basic::BasicClient;
use oauth2::url::ParseError;
use oauth2::{AuthUrl, AuthorizationCode, ClientId, CsrfToken, RedirectUrl, Scope};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use url::Url;

#[derive(Debug, Clone)]
pub enum OAuthError {
    BrowserOpenError(String),
    ParseError(ParseError),
    AlreadyRunning(String),
    ReadLineError(String),
    RequestSplitError(),
    NoCodeInResponse(),
    NoCsrfInResponse(),
    MismatchCsrf(CsrfToken, CsrfToken),
    ReceivedNone(),
}

impl From<ParseError> for OAuthError {
    fn from(e: ParseError) -> Self {
        OAuthError::ParseError(e)
    }
}

pub async fn get_auth_code(cfg: Config) -> Result<AuthorizationCode, OAuthError> {
    //TODO LOG
    //TODO Get Value from Config

    // 815279319513432134
    let client_id = ClientId::new(cfg.client_id);
    let secret = None;
    //https://discord.com/api/oauth2/authorize
    let auth_url = AuthUrl::new(cfg.auth_url)?;
    let token_url = None;

    //http://localhost:1887
    let client = BasicClient::new(client_id, secret, auth_url, token_url)
        .set_redirect_uri(RedirectUrl::new(format!("http://{}", cfg.redirect_url))?);

    let (authorize_url, csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the user's public repos and email.
        .add_scope(Scope::new("identify".to_string()))
        .add_extra_param("response_type", "code")
        .url();

    webbrowser::open(authorize_url.as_str())
        .map_err(|e| OAuthError::BrowserOpenError(format!("{:?}", e)))?;

    // A very naive implementation of the redirect server.
    // 127.0.0.1:1887
    let listener = TcpListener::bind(cfg.redirect_url)
        .await
        .map_err(|e| OAuthError::AlreadyRunning(format!("{:?}", e)))?;
    loop {
        if let Ok((mut stream, _)) = listener.accept().await {
            let code;
            let state;
            {
                let mut reader = BufReader::new(&mut stream);

                let mut request_line = String::new();
                reader
                    .read_line(&mut request_line)
                    .await
                    .map_err(|e| OAuthError::ReadLineError(format!("{:?}", e)))?;

                let redirect_url = request_line
                    .split_whitespace()
                    .nth(1)
                    .ok_or(OAuthError::RequestSplitError())?;
                let url = Url::parse(&("http://localhost".to_string() + redirect_url))?;

                let code_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "code"
                    })
                    .ok_or(OAuthError::NoCodeInResponse())?;

                let (_, value) = code_pair;
                code = AuthorizationCode::new(value.into_owned());

                let state_pair = url
                    .query_pairs()
                    .find(|pair| {
                        let &(ref key, _) = pair;
                        key == "state"
                    })
                    .ok_or(OAuthError::NoCsrfInResponse())?;

                let (_, value) = state_pair;
                state = CsrfToken::new(value.into_owned());
            }

            let message = "This page can be closed now";
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-length: {}\r\n\r\n{}",
                message.len(),
                message
            );
            if stream.write_all(response.as_bytes()).await.is_err() {
                //TODO LOG but Ignore
            };

            if state.secret() != csrf_state.secret() {
                return Err(OAuthError::MismatchCsrf(state, csrf_state));
            }

            // The server will terminate itself after collecting the first code.
            return Ok(code);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client_id: String,
    pub auth_url: String,
    pub redirect_url: String,
}
