use std::env;

pub fn pwd_command() {
    let current_path = env::current_dir();
    match current_path {
        Ok(path) => print!("{}\n", path.display()),
        Err(_) => print!("\n"),
    }
}
