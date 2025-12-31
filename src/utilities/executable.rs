use std::{env, fs, os::unix::fs::PermissionsExt};

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
        Ok(metadata) => metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0),
        Err(_) => false,
    }
}

pub fn print_all_exec(executables: Vec<String>) {
    for exe in &executables {
        print!("{}  ", exe);
    }
    print!("\r\n");
}
