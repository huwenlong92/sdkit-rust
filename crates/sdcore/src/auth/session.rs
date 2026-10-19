use axum::http::HeaderMap;
use rand_core::{OsRng, RngCore};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use super::token::session_token;

pub const DEFAULT_SESSION_TTL_SECONDS: u64 = 30 * 60;

#[derive(Clone)]
pub struct Session<T = ()> {
    pub token: String,
    pub data: T,
    pub expires_at: SystemTime,
}

pub type TypedSessionStore<T> = Arc<Mutex<Vec<Session<T>>>>;
pub type SessionStore = TypedSessionStore<()>;

pub fn session_store() -> SessionStore {
    typed_session_store()
}

pub fn typed_session_store<T>() -> TypedSessionStore<T> {
    Arc::new(Mutex::new(Vec::new()))
}

pub fn create_session(username: &str, ttl_seconds: u64) -> Session {
    create_session_with_data(username, (), ttl_seconds)
}

pub fn create_session_with_data<T>(subject: &str, data: T, ttl_seconds: u64) -> Session<T> {
    Session {
        token: create_session_token(subject),
        data,
        expires_at: expires_at(ttl_seconds),
    }
}

pub fn is_authorized<T>(
    sessions: &TypedSessionStore<T>,
    headers: &HeaderMap,
    cookie_name: &str,
    alternate_header_name: &str,
) -> bool {
    let Some(token) = session_token(headers, cookie_name, alternate_header_name) else {
        return false;
    };
    contains_session(sessions, &token).unwrap_or(false)
}

pub fn contains_session<T>(sessions: &TypedSessionStore<T>, token: &str) -> Result<bool, String> {
    let now = SystemTime::now();
    sessions
        .lock()
        .map(|mut sessions| {
            sessions.retain(|session| session.expires_at > now);
            sessions.iter().any(|session| session.token == token)
        })
        .map_err(|e| e.to_string())
}

pub fn renew_session<T>(
    sessions: &TypedSessionStore<T>,
    token: &str,
    ttl_seconds: u64,
) -> Result<bool, String> {
    let now = SystemTime::now();
    let next_expires_at = expires_at(ttl_seconds);
    sessions
        .lock()
        .map(|mut sessions| {
            sessions.retain(|session| session.expires_at > now);
            if let Some(session) = sessions.iter_mut().find(|session| session.token == token) {
                session.expires_at = next_expires_at;
                true
            } else {
                false
            }
        })
        .map_err(|e| e.to_string())
}

pub fn session_data<T: Clone>(
    sessions: &TypedSessionStore<T>,
    token: &str,
) -> Result<Option<T>, String> {
    let now = SystemTime::now();
    sessions
        .lock()
        .map(|mut sessions| {
            sessions.retain(|session| session.expires_at > now);
            sessions
                .iter()
                .find(|session| session.token == token)
                .map(|session| session.data.clone())
        })
        .map_err(|e| e.to_string())
}

pub fn remove_session<T>(sessions: &TypedSessionStore<T>, token: &str) -> Result<(), String> {
    sessions
        .lock()
        .map(|mut sessions| {
            sessions.retain(|session| session.token != token);
        })
        .map_err(|e| e.to_string())
}

fn create_session_token(_subject: &str) -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn expires_at(ttl_seconds: u64) -> SystemTime {
    SystemTime::now() + Duration::from_secs(ttl_seconds)
}
