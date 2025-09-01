mod error;
pub mod oauth;
pub mod slides;
pub mod splitter;

use std::{collections::HashMap, time::Duration};

use tracing::{Level, info};
use worker::*;

/// Creates a cookie string with the given name, value, and max-age (in seconds).
fn cookie(name: &str, value: &str, max_age: u64) -> String {
    format!("{name}={value}; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age={max_age}")
}

/// Retrieves the value of a cookie by name from the "Cookie" header string.
fn get_cookie(cookies: &str, name: &str) -> Option<String> {
    cookies
        .split(';')
        .filter_map(|cookie| {
            let cookie = cookie.trim();
            cookie.split_once('=')
        })
        .find_map(|(k, v)| if k == name { Some(v.to_string()) } else { None })
}

#[event(start)]
pub fn init() {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .without_time()
        .with_line_number(true)
        .pretty()
        .init();

    #[cfg(feature = "panic-hook")]
    console_error_panic_hook::set_once();

    info!("Worker initialized");
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::ok("Hello from Text2Deck!"))
        .get("/health", |_, _| Response::ok("OK"))
        .get_async("/oauth/start", |_, ctx| async move {
            let (auth_url, state, verifier) = oauth::start(&ctx).await?;

            let mut resp = Response::redirect(auth_url)?;
            let headers = resp.headers_mut();
            headers.set("Set-Cookie", &cookie("state", &state, 600))?;
            headers.append("Set-Cookie", &cookie("verifier", &verifier, 600))?;

            Ok(resp)
        })
        .get_async("/oauth/callback", |req, ctx| async move {
            let url = req.url()?;
            let query_pairs: HashMap<_, _> = url.query_pairs().into_owned().collect();

            let code = query_pairs.get("code").ok_or("missing code")?.to_string();
            let state = query_pairs.get("state").ok_or("missing state")?.to_string();

            let cookies = req.headers().get("Cookie")?.unwrap_or_default();
            let state_c = get_cookie(&cookies, "state").ok_or("no state cookie")?;
            if state != state_c {
                return Response::error("state mismatch", 400);
            }

            let verifier = get_cookie(&cookies, "verifier").ok_or("no verifier cookie")?;
            let token = oauth::exchange(&ctx, &code, &verifier).await?;
            let session_id = oauth::generate_session_id();
            let kv = ctx.kv("TOKENS")?;

            const TWO_WEEKS_SECS: u64 = 14 * 24 * 60 * 60;
            kv.put(&session_id, &token)?
                .expiration_ttl(TWO_WEEKS_SECS)
                .execute()
                .await?;

            let mut resp = Response::redirect(Url::parse("/app")?)?;
            resp.headers_mut()
                .set("Set-Cookie", &cookie("sid", &session_id, TWO_WEEKS_SECS))?;

            Ok(resp)
        })
        .run(req, env)
        .await
}
