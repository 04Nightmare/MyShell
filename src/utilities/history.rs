use std::{
    fs::File,
    io::{BufRead, BufReader},
};

pub fn fetch_history_commands(history_index: usize) -> Option<String> {
    let file = File::open("history.txt");
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().flatten().collect();
            let index = lines.len().checked_sub(history_index)?;
            return lines.get(index).cloned();
        }
        Err(_) => {
            return None;
        }
    }
}
