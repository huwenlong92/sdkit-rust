#[test]
fn configuration_errors_do_not_expose_contents() {
    let path = std::env::temp_dir().join(format!("sdcore-config-{}.toml", std::process::id()));
    std::fs::write(&path, "secret = [sensitive-value").unwrap();
    let error = sdcore::config::load_toml::<serde_json::Value>(&path).unwrap_err();
    std::fs::remove_file(path).unwrap();
    assert!(!error.contains("sensitive-value"));
}
