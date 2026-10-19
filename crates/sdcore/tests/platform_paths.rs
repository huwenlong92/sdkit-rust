use sdcore::{
    paths::{resolve_executable, user_data_dir},
    process::ProcessRequest,
};
use std::ffi::OsStr;

#[test]
fn executable_lookup_handles_spaces_and_explicit_paths_without_a_shell() {
    let root = std::env::temp_dir().join(format!(
        "sdcore-paths-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let bin = root.join("tools with spaces");
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(&bin).unwrap();
    let file = bin.join(if cfg!(windows) {
        "connector.exe"
    } else {
        "connector"
    });
    std::fs::write(&file, "fixture").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert!(resolve_executable(&file, OsStr::new("")).is_none());
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let expected = file.canonicalize().unwrap();
    assert_eq!(
        resolve_executable(&file, OsStr::new("")),
        Some(expected.clone())
    );
    assert_eq!(
        resolve_executable("connector", &std::env::join_paths([&bin]).unwrap()),
        Some(expected)
    );
    assert!(resolve_executable("missing", &std::env::join_paths([bin]).unwrap()).is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn application_paths_reject_directory_traversal() {
    for name in ["", "..", ".", "../other", "other/tool", "other\\tool"] {
        assert!(user_data_dir(name).is_err());
    }
}

#[test]
fn changing_isolated_home_replaces_previous_home_variables() {
    let request = ProcessRequest::new("connector")
        .with_isolated_home("first")
        .with_isolated_home("second");
    assert!(request.clear_env);
    let values: Vec<_> = request
        .envs
        .iter()
        .filter(|(key, _)| key == "HOME")
        .collect();
    assert_eq!(values.len(), 1);
    assert_eq!(values[0].1, "second");
    #[cfg(windows)]
    for name in ["USERPROFILE", "APPDATA", "LOCALAPPDATA", "TEMP", "TMP"] {
        assert_eq!(
            request.envs.iter().find(|(key, _)| key == name).unwrap().1,
            "second"
        );
    }
}
