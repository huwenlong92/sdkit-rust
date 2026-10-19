#![cfg(feature = "auth")]
use sdcore::auth::{
    clear_session_cookie_with_security, create_session_with_data, remove_session, renew_session,
    session_cookie_with_security, session_data, typed_session_store,
};

#[test]
fn typed_session_store_keeps_data_and_renews_expiry() {
    let store = typed_session_store::<String>();
    let session = create_session_with_data("admin", "admin".to_string(), 30);
    let token = session.token.clone();
    let original_expires_at = session.expires_at;
    store.lock().unwrap().push(session);

    assert_eq!(
        session_data(&store, &token).unwrap(),
        Some("admin".to_string())
    );
    assert!(renew_session(&store, &token, 3600).unwrap());

    let renewed_expires_at = store.lock().unwrap()[0].expires_at;
    assert!(renewed_expires_at > original_expires_at);

    remove_session(&store, &token).unwrap();
    assert_eq!(session_data(&store, &token).unwrap(), None);
}

#[test]
fn secure_session_cookies_include_secure_attribute() {
    let (_, cookie) = session_cookie_with_security("session", "secret", 60, true);
    let value = cookie.to_str().unwrap();

    assert!(value.contains("HttpOnly"));
    assert!(value.contains("SameSite=Lax"));
    assert!(value.contains("; Secure"));
}

#[test]
fn development_session_cookies_can_omit_secure_attribute() {
    let (_, cookie) = session_cookie_with_security("session", "secret", 60, false);
    let value = cookie.to_str().unwrap();
    let (_, cleared) = clear_session_cookie_with_security("session", false);
    let cleared = cleared.to_str().unwrap();

    assert!(!value.contains("; Secure"));
    assert!(!cleared.contains("; Secure"));
    assert!(cleared.contains("Max-Age=0"));
}
