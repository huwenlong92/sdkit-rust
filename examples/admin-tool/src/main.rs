use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;

#[sdcore::tokio::main]
async fn main() -> sdcore::Result<()> {
    let app = Router::new().route("/health", get(|| async { Json(json!({"status": "ok"})) }));
    sdcore::app::serve("localhost:9001", app).await
}
