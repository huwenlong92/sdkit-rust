use serde::de::DeserializeOwned;
use std::path::Path;

/// Load a caller-owned configuration shape without including contents in errors.
pub fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let contents = std::fs::read_to_string(path)
        .map_err(|_| format!("cannot read configuration {}", path.display()))?;
    toml::from_str(&contents).map_err(|_| format!("invalid configuration {}", path.display()))
}
