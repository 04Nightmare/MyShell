use std::{
    fs::{File, OpenOptions},
    io::Write,
    str::FromStr,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Redirect {
    Stdout,
    StdoutAppend,
    Stderr,
    StderrAppend,
}

impl FromStr for Redirect {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ">" | "1>" => Ok(Redirect::Stdout),
            ">>" | "1>>" => Ok(Redirect::StdoutAppend),
            "2>" => Ok(Redirect::Stderr),
            "2>>" => Ok(Redirect::StderrAppend),
            _ => Err(()),
        }
    }
}

pub fn get_filepath(args: &[String]) -> Option<&String> {
    let position = args.iter().position(|s| s.parse::<Redirect>().is_ok());
    let filepath = if let Some(position) = position {
        args.get(position + 1)
    } else {
        None
    };
    return filepath;
}

pub fn handle_redirect(filepath: &String, filecontent: &[u8]) {
    let file = File::create(filepath);
    match file {
        Ok(mut file) => {
            file.write_all(filecontent.trim_ascii()).unwrap();
            file.write_all(b"\n").unwrap();
        }
        Err(_) => {
            println!("cant create file");
        }
    }
}

pub fn handle_redirect_append(filepath: &String, filecontent: &[u8]) {
    let file = OpenOptions::new().create(true).append(true).open(filepath);
    match file {
        Ok(mut file) => {
            file.write_all(filecontent.trim_ascii()).unwrap();
            if !filecontent.is_empty() {
                file.write_all(b"\n").unwrap();
            }
        }
        Err(_) => {
            println!("cant create file");
        }
    }
}
