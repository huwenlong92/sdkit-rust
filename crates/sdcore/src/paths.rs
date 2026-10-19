//! Platform paths for native command-line applications.
use std::{
    ffi::OsStr,
    io,
    path::{Path, PathBuf},
};

/// The per-user application data directory; does not create it.
pub fn user_data_dir(application: &str) -> io::Result<PathBuf> {
    if application.is_empty()
        || Path::new(application).components().count() != 1
        || application == "."
        || application == ".."
        || application.contains(['/', '\\'])
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid application directory name",
        ));
    }
    let absolute_env = |key| {
        std::env::var_os(key)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    };
    #[cfg(windows)]
    let base = absolute_env("LOCALAPPDATA")
        .or_else(|| absolute_env("USERPROFILE").map(|p| p.join("AppData/Local")));
    #[cfg(target_os = "macos")]
    let base = absolute_env("HOME").map(|p| p.join("Library/Application Support"));
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = absolute_env("XDG_DATA_HOME")
        .or_else(|| absolute_env("HOME").map(|p| p.join(".local/share")));
    #[cfg(not(any(unix, windows)))]
    let base: Option<PathBuf> = None;
    base.map(|p| p.join(application)).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "user data directory unavailable; specify an explicit data directory",
        )
    })
}

/// Resolve an explicit path or a bare program name using only the supplied PATH.
/// Windows .exe is supported without implicitly invoking a shell or batch file.
pub fn resolve_executable(program: impl AsRef<OsStr>, search_path: &OsStr) -> Option<PathBuf> {
    let program = Path::new(program.as_ref());
    let candidates: Vec<_> = if program.is_absolute() || program.components().count() > 1 {
        vec![program.to_path_buf()]
    } else {
        std::env::split_paths(search_path)
            .filter(|p| !p.as_os_str().is_empty())
            .map(|p| p.join(program))
            .collect()
    };
    for candidate in candidates {
        if let Some(path) = executable_file(&candidate) {
            return Some(path);
        }
        #[cfg(windows)]
        if candidate.extension().is_none() {
            if let Some(path) = executable_file(&candidate.with_extension("exe")) {
                return Some(path);
            }
        }
    }
    None
}

fn executable_file(path: &Path) -> Option<PathBuf> {
    let metadata = path.metadata().ok()?;
    if !metadata.is_file() {
        return None;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return None;
        }
    }
    path.canonicalize().ok()
}
