use axum::http::header::{AUTHORIZATION, COOKIE};
use axum::http::HeaderMap;

pub fn session_token(
    headers: &HeaderMap,
    cookie_name: &str,
    alternate_header_name: &str,
) -> Option<String> {
    bearer_token(headers)
        .or_else(|| header_value(headers, alternate_header_name))
        .or_else(|| cookie_value(headers, cookie_name))
}

pub fn bearer_token(headers: &HeaderMap) -> Option<String> {
    header_value(headers, AUTHORIZATION.as_str())
        .and_then(|value| value.strip_prefix("Bearer ").map(str::to_string))
}

pub fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = headers.get(COOKIE)?.to_str().ok()?;
    for pair in cookie.split(';') {
        let (cookie_name, value) = pair.trim().split_once('=')?;
        if cookie_name == name {
            return Some(value.to_string());
        }
    }
    None
}

pub fn header_value(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
}
