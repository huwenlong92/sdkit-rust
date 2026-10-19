use std::path::Path;
use std::process::Command;

pub fn run_shell(cwd: &Path, command: &str) -> Result<std::process::Output, String> {
    Command::new("sh")
        .arg("-lc")
        .arg(command)
        .current_dir(cwd)
        .output()
        .map_err(|err| err.to_string())
}
