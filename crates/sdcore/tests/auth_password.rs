#![cfg(feature = "auth")]
use sdcore::auth::{create_session, hash_password, verify_password};
#[test]
fn passwords_are_salted_and_sessions_are_random() {
    let a = hash_password("a-long-test-password").unwrap();
    let b = hash_password("a-long-test-password").unwrap();
    assert_ne!(a, b);
    assert!(a.starts_with("$argon2id$"));
    assert!(verify_password("a-long-test-password", &a));
    assert!(!verify_password("wrong", &a));
    assert!(!verify_password("wrong", "invalid"));
    let a = create_session("same", 60).token;
    let b = create_session("same", 60).token;
    assert_eq!(a.len(), 64);
    assert_ne!(a, b);
}
