mod error;
pub mod oauth;

use tracing::Level;
use worker::{Context, Env, Request, Response, Result, Router, console_log, event};

#[event(start)]
pub fn init() {
    console_log!("Worker starting up...");
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .without_time()
        .with_line_number(true)
        .pretty()
        .init();

    #[cfg(feature = "panic-hook")]
    console_error_panic_hook::set_once();
}

#[event(fetch)]
pub async fn main(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        .get("/", |_, _| Response::ok("Hello from Text2Deck!"))
        .get("/health", |_, _| Response::ok("OK"))
        .run(req, env)
        .await
}
