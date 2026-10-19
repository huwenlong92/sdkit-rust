use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;

pub fn session_cookie(
    name: &str,
    token: &str,
    max_age: u64,
) -> (axum::http::HeaderName, HeaderValue) {
    session_cookie_with_security(name, token, max_age, false)
}

pub fn session_cookie_with_security(
    name: &str,
    token: &str,
    max_age: u64,
    secure: bool,
) -> (axum::http::HeaderName, HeaderValue) {
    let secure_attribute = if secure { "; Secure" } else { "" };
    let value = format!(
        "{name}={token}; Path=/; Max-Age={max_age}; HttpOnly; SameSite=Lax{secure_attribute}"
    );
    (
        SET_COOKIE,
        HeaderValue::from_str(&value).expect("valid session cookie"),
    )
}

pub fn clear_session_cookie(name: &str) -> (axum::http::HeaderName, HeaderValue) {
    clear_session_cookie_with_security(name, false)
}

pub fn clear_session_cookie_with_security(
    name: &str,
    secure: bool,
) -> (axum::http::HeaderName, HeaderValue) {
    let secure_attribute = if secure { "; Secure" } else { "" };
    let value = format!("{name}=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{secure_attribute}");
    (
        SET_COOKIE,
        HeaderValue::from_str(&value).expect("valid session cookie"),
    )
}
