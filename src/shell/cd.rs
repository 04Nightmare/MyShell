use std::{env, path::Path};

pub fn cd_command(ab_path: &str) {
    if ab_path == "~" {
        let home_dir = env::home_dir();
        match home_dir {
            Some(home_dir_path) => {
                let home_dir_path_string = home_dir_path.display().to_string();
                let cd_path = Path::new(&home_dir_path_string);
                match env::set_current_dir(&cd_path) {
                    Ok(_) => {}
                    Err(_) => print!("cd: can't get to home directory\n"),
                }
            }
            None => print!("cd: can't get to home directory\n"),
        }
    } else {
        let cd_path = Path::new(ab_path);
        let is_path = env::set_current_dir(&cd_path);
        match is_path {
            Ok(_) => {}
            Err(_) => print!("cd: {}: No such file or directory\n", ab_path),
        }
    }
}
