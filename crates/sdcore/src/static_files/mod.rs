use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub type StaticAsset = (&'static [u8], &'static str);

pub fn asset_response(asset: StaticAsset) -> Response {
    ([(CONTENT_TYPE, asset.1)], asset.0).into_response()
}

pub fn not_found_json() -> Response {
    (
        StatusCode::NOT_FOUND,
        [(CONTENT_TYPE, "application/json; charset=utf-8")],
        r#"{"error":"not found"}"#,
    )
        .into_response()
}
