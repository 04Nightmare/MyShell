use std::{
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

pub const MAX_HISTORY: usize = 1000;

struct HistoryState {
    entries: Vec<String>,
    backing_file: PathBuf,
    /// entries[..backing_synced_len] are already in backing_file
    backing_synced_len: usize,
    /// entries[..append_synced_len] have been flushed via `-a`/`-w`
    append_synced_len: usize,
}

fn absolutize(p: &Path) -> PathBuf {
    if p.is_absolute() {
        p.to_path_buf()
    } else if let Ok(cwd) = std::env::current_dir() {
        cwd.join(p)
    } else {
        p.to_path_buf()
    }
}

fn backing_file_path() -> PathBuf {
    // Resolved once at startup to an absolute path, so later `cd` does not
    // relocate the central history file.
    let raw = std::env::var("HISTFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("history.txt"));
    absolutize(&raw)
}

fn history_store() -> &'static Mutex<HistoryState> {
    static STORE: OnceLock<Mutex<HistoryState>> = OnceLock::new();
    STORE.get_or_init(|| {
        Mutex::new(HistoryState {
            entries: Vec::new(),
            backing_file: backing_file_path(),
            backing_synced_len: 0,
            append_synced_len: 0,
        })
    })
}

fn enforce_cap_locked(state: &mut HistoryState) {
    if state.entries.len() > MAX_HISTORY {
        let drain = state.entries.len() - MAX_HISTORY;
        state.entries.drain(..drain);
        state.backing_synced_len = state.backing_synced_len.saturating_sub(drain);
        state.append_synced_len = state.append_synced_len.saturating_sub(drain);
    }
}

fn is_backing_locked(state: &HistoryState, path: &str) -> bool {
    // Compare absolute paths: backing is absolute since init, arg may be
    // relative to the current directory (which can change via `cd`).
    absolutize(Path::new(path)) == state.backing_file.as_path()
}

fn read_lines_from_file(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().flatten().collect())
}

/// Load the single history file into memory. Called once at shell start.
pub fn init_history() {
    let path = backing_file_path();
    let mut state = history_store().lock().unwrap();
    // Refresh backing path in case HISTFILE changed (tests set it per-run).
    state.backing_file = path.clone();
    if let Ok(file) = File::open(&path) {
        let reader = BufReader::new(file);
        let mut lines: Vec<String> = reader.lines().flatten().collect();
        if lines.len() > MAX_HISTORY {
            lines.drain(..lines.len() - MAX_HISTORY);
        }
        state.entries = lines;
        state.backing_synced_len = state.entries.len();
        state.append_synced_len = state.entries.len();
    } else {
        state.entries.clear();
        state.backing_synced_len = 0;
        state.append_synced_len = 0;
    }
}

/// Append a new command to in-memory history (cap 1000). Empty inputs ignored.
pub fn push_history(cmd: &str) {
    if cmd.is_empty() {
        return;
    }
    let mut state = history_store().lock().unwrap();
    state.entries.push(cmd.to_string());
    enforce_cap_locked(&mut state);
}

/// 1-based from the end: 1 = most recent.
pub fn fetch_history_commands(history_index: usize) -> Option<String> {
    let state = history_store().lock().unwrap();
    let index = state.entries.len().checked_sub(history_index)?;
    state.entries.get(index).cloned()
}

pub fn get_history() -> Vec<String> {
    history_store().lock().unwrap().entries.clone()
}

/// `history -r FILE`: read FILE lines into memory.
pub fn history_read_from_file(path: &str) -> std::io::Result<()> {
    let lines = read_lines_from_file(path)?;
    let mut state = history_store().lock().unwrap();
    state.entries.extend(lines);
    enforce_cap_locked(&mut state);
    Ok(())
}

/// `history -w FILE`: overwrite FILE with full in-memory history.
pub fn history_write_to_file(path: &str) -> std::io::Result<()> {
    let (contents, len, is_backing) = {
        let state = history_store().lock().unwrap();
        let body = if state.entries.is_empty() {
            String::new()
        } else {
            format!("{}\n", state.entries.join("\n"))
        };
        (body, state.entries.len(), is_backing_locked(&state, path))
    };
    let mut file = File::create(path)?;
    file.write_all(contents.as_bytes())?;
    let mut state = history_store().lock().unwrap();
    // After a full write there is nothing "new" left for `-a`.
    state.append_synced_len = len;
    if is_backing {
        state.backing_synced_len = len;
    }
    Ok(())
}

/// `history -a FILE`: append only new entries to FILE.
pub fn history_append_to_file(path: &str) -> std::io::Result<()> {
    let (new_entries, is_backing) = {
        let state = history_store().lock().unwrap();
        let start = if is_backing_locked(&state, path) {
            state.backing_synced_len
        } else {
            state.append_synced_len
        };
        (
            state.entries[start.min(state.entries.len())..].to_vec(),
            is_backing_locked(&state, path),
        )
    };
    // Open with create so the file exists even when there is nothing new.
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    for line in &new_entries {
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
    }
    let mut state = history_store().lock().unwrap();
    state.append_synced_len = state.entries.len();
    if is_backing {
        state.backing_synced_len = state.entries.len();
    }
    Ok(())
}

/// Append session-new commands to the single history file. Called before exit.
pub fn persist_history_on_exit() {
    let (new_entries, path) = {
        let state = history_store().lock().unwrap();
        let start = state.backing_synced_len.min(state.entries.len());
        (
            state.entries[start..].to_vec(),
            state.backing_file.clone(),
        )
    };
    if new_entries.is_empty() {
        return;
    }
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&path) {
        for line in &new_entries {
            let _ = file.write_all(line.as_bytes());
            let _ = file.write_all(b"\n");
        }
        if let Ok(mut state) = history_store().lock() {
            state.backing_synced_len = state.entries.len();
            state.append_synced_len = state.entries.len();
        }
    }
}
