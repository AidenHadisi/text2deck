mod error;
mod oauth;
mod slides;
mod splitter;

use crate::slides::CreateSlidesRequest;
use std::collections::HashMap;
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
        .get("/", |_, _| {
            // Serve the main HTML file
            let html = include_str!("../../web/index.html");
            Response::from_html(html)
        })
        .get("/app", |_, _| {
            // Serve the main HTML file
            let html = include_str!("../../web/index.html");
            Response::from_html(html)
        })
        .get("/pkg/*", |_req, _| {
            // For now, return instructions to build the WASM files
            let instructions = r#"
<!DOCTYPE html>
<html>
<head>
    <title>Build Required</title>
    <style>
        body { font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }
        .code { background: #f5f5f5; padding: 10px; border-radius: 4px; font-family: monospace; }
    </style>
</head>
<body>
    <h1>🔧 Build Required</h1>
    <p>The WASM files need to be built first. Run these commands:</p>
    <div class="code">
        cd web<br>
        wasm-pack build --target web --out-dir pkg<br>
        cd ../worker<br>
        wrangler dev
    </div>
    <p>Or use the build script: <code>./build.sh</code></p>
</body>
</html>
            "#;
            Response::from_html(instructions)
        })
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
        .post_async("/api/create-slides", |mut req, ctx| async move {
            // Get session ID from cookie
            let cookies = req.headers().get("Cookie")?.unwrap_or_default();
            let session_id = get_cookie(&cookies, "sid").ok_or("no session cookie")?;

            // Get token from KV store
            let kv = ctx.kv("TOKENS")?;
            let token_data = kv.get(&session_id).text().await?.ok_or("invalid session")?;
            let token: oauth::Token = serde_json::from_str(&token_data)
                .map_err(|e| worker::Error::from(format!("Failed to parse token: {}", e)))?;

            // Parse request body
            let slides_request: CreateSlidesRequest = req
                .json()
                .await
                .map_err(|e| worker::Error::from(format!("Invalid request body: {}", e)))?;

            // Create slides
            match slides::create_slides_from_text(&token, &slides_request).await {
                Ok(presentation_id) => {
                    let presentation_url = format!(
                        "https://docs.google.com/presentation/d/{}/edit",
                        presentation_id
                    );
                    let response = serde_json::json!({
                        "presentation_id": presentation_id,
                        "presentation_url": presentation_url,
                        "message": "Slides created successfully"
                    });
                    Response::from_json(&response)
                }
                Err(e) => {
                    let error_response = serde_json::json!({
                        "error": e.to_string(),
                        "message": "Failed to create slides"
                    });
                    Ok(Response::from_json(&error_response)?.with_status(400))
                }
            }
        })
        .get("/api/splitters", |_, _| {
            let splitters = serde_json::json!({
                "splitters": [
                    {
                        "type": "newline",
                        "name": "New Line Splitter",
                        "description": "Splits text by individual lines"
                    },
                    {
                        "type": "empty_line",
                        "name": "Empty Line Splitter",
                        "description": "Splits text by empty lines (paragraphs)"
                    },
                    {
                        "type": "max_words",
                        "name": "Max Words Splitter",
                        "description": "Splits text by maximum word count per slide",
                        "config": {
                            "max_words": "number (default: 50)"
                        }
                    },
                    {
                        "type": "max_chars",
                        "name": "Max Characters Splitter",
                        "description": "Splits text by maximum character count per slide",
                        "config": {
                            "max_chars": "number (default: 500)"
                        }
                    }
                ]
            });
            Response::from_json(&splitters)
        })
        .run(req, env)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::basic_cookie(
        "session",
        "abc123",
        3600,
        "session=abc123; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=3600"
    )]
    #[case::empty_value(
        "empty",
        "",
        0,
        "empty=; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=0"
    )]
    #[case::special_characters(
        "test-cookie",
        "value_123",
        7200,
        "test-cookie=value_123; Path=/; HttpOnly; SameSite=Lax; Secure; Max-Age=7200"
    )]
    fn test_cookie_creation(
        #[case] name: &str,
        #[case] value: &str,
        #[case] max_age: u64,
        #[case] expected: &str,
    ) {
        assert_eq!(cookie(name, value, max_age), expected);
    }

    #[rstest]
    #[case::single_cookie("session=abc123", "session", Some("abc123"))]
    #[case::multiple_cookies_first(
        "session=abc123; user=john; theme=dark",
        "session",
        Some("abc123")
    )]
    #[case::multiple_cookies_middle("session=abc123; user=john; theme=dark", "user", Some("john"))]
    #[case::multiple_cookies_last("session=abc123; user=john; theme=dark", "theme", Some("dark"))]
    #[case::with_spaces_first(" session=abc123 ; user=john ", "session", Some("abc123"))]
    #[case::with_spaces_second(" session=abc123 ; user=john ", "user", Some("john"))]
    #[case::not_found("session=abc123; user=john", "nonexistent", None)]
    #[case::empty_string("", "session", None)]
    #[case::malformed_no_value("session; user=john", "session", None)]
    #[case::malformed_but_valid_other("session; user=john", "user", Some("john"))]
    #[case::duplicate_names_returns_first(
        "session=first; session=second",
        "session",
        Some("first")
    )]
    #[case::empty_value_exists("session=; user=john", "session", Some(""))]
    #[case::empty_value_other_valid("session=; user=john", "user", Some("john"))]
    #[case::value_with_equals("token=abc=def=123", "token", Some("abc=def=123"))]
    #[case::case_sensitive_uppercase("Session=abc123; session=def456", "Session", Some("abc123"))]
    #[case::case_sensitive_lowercase("Session=abc123; session=def456", "session", Some("def456"))]
    #[case::case_sensitive_not_found("Session=abc123; session=def456", "SESSION", None)]
    #[case::only_semicolons(";;;", "anything", None)]
    fn test_get_cookie(#[case] cookies: &str, #[case] name: &str, #[case] expected: Option<&str>) {
        let result = get_cookie(cookies, name);
        let expected = expected.map(|s| s.to_string());
        assert_eq!(result, expected);
    }
}
