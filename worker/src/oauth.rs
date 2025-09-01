use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{Rng, distr::Alphanumeric};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use worker::{
    Date, Error, Fetch, Headers, Method, Request, RequestInit, Result, RouteContext, Url,
};

// OAuth URLs
const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";

// OAuth configuration
const GOOGLE_SCOPES: &str =
    "https://www.googleapis.com/auth/presentations https://www.googleapis.com/auth/drive.file";

// Security parameters
const STATE_LENGTH: usize = 24;
const VERIFIER_LENGTH: usize = 64;

/// Represents an OAuth 2.0 access token response from Google.
#[derive(Debug, Clone, Deserialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
    pub scope: String,
    pub created_at: u64,
}

/// Generates a cryptographically secure random string of the specified length.
fn generate_random_string(length: usize) -> String {
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

/// Generates a PKCE code challenge from a verifier string.
fn generate_pkce_challenge(verifier: &str) -> String {
    let digest = Sha256::digest(verifier.as_bytes());
    URL_SAFE_NO_PAD.encode(digest)
}

/// Initiates the OAuth 2.0 authorization flow with Google.
pub async fn start(ctx: &RouteContext<()>) -> Result<(Url, String, String)> {
    let client_id = ctx.var("GOOGLE_CLIENT_ID")?.to_string();
    let redirect_uri = ctx.var("GOOGLE_REDIRECT_URI")?.to_string();

    let state = generate_random_string(STATE_LENGTH);
    let verifier = generate_random_string(VERIFIER_LENGTH);
    let challenge = generate_pkce_challenge(&verifier);

    let mut url = Url::parse(GOOGLE_AUTH_URL)?;
    url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("response_type", "code")
        .append_pair("scope", GOOGLE_SCOPES)
        .append_pair("state", &state)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");

    Ok((url, state, verifier))
}

/// Exchanges an authorization code for access and refresh tokens.
pub async fn exchange(ctx: &RouteContext<()>, code: &str, verifier: &str) -> Result<Token> {
    let client_id = ctx.var("GOOGLE_CLIENT_ID")?.to_string();
    let client_secret = ctx.var("GOOGLE_CLIENT_SECRET")?.to_string();
    let redirect_uri = ctx.var("GOOGLE_REDIRECT_URI")?.to_string();

    let params = [
        ("code", code),
        ("client_id", &client_id),
        ("client_secret", &client_secret),
        ("redirect_uri", &redirect_uri),
        ("grant_type", "authorization_code"),
        ("code_verifier", verifier),
    ];

    let body = serde_urlencoded::to_string(&params).map_err(|e| Error::from(e.to_string()))?;

    let headers = Headers::new();
    headers.set("Content-Type", "application/x-www-form-urlencoded")?;

    let mut init = RequestInit::new();
    init.with_method(Method::Post)
        .with_body(Some(body.into()))
        .with_headers(headers);

    let request = Request::new_with_init(GOOGLE_TOKEN_URL, &init)?;
    let mut response = Fetch::Request(request).send().await?;

    let mut token: Token = response.json().await?;
    token.created_at = Date::now().as_millis() / 1000;

    Ok(token)
}
