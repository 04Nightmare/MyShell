use std::{env, fs};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub fn get_path_directories() -> Vec<std::path::PathBuf> {
    return env::var_os("PATH")
        .unwrap_or_default()
        .to_string_lossy()
        .split(':')
        .map(std::path::PathBuf::from)
        .collect();
}

pub fn find_executable(char_slice: &str) -> Vec<String> {
    let mut results = Vec::new();
    for dir in get_path_directories() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(execname) = path.file_name().and_then(|n| n.to_str()) {
                    if execname.starts_with(char_slice) && is_executable(&path) {
                        results.push(execname.to_string());
                    }
                }
            }
        }
    }
    results.sort();
    results.dedup();
    return results;
}

pub fn is_executable(path: &std::path::Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => {
            if !metadata.is_file() {
                return false;
            }
            #[cfg(unix)]
            {
                metadata.permissions().mode() & 0o111 != 0
            }
            #[cfg(windows)]
            {
                matches!(
                    path.extension().and_then(|e| e.to_str()),
                    Some("exe" | "cmd" | "bat" | "com")
                )
            }
        }
        Err(_) => false,
    }
}

pub fn print_all_exec(executables: Vec<String>) {
    for exe in &executables {
        print!("{}  ", exe);
    }
    print!("\r\n");
}
