use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
//use anyhow::Ok;
use pathsearch::find_executable_in_path;
use std::fs::{File, OpenOptions};
#[allow(unused_imports)]
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;
use std::{env, fs};

const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Redirect {
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

//Utility Functions
fn input_line_parsing(input: &str) -> Vec<String> {
    let mut result_args = Vec::new();
    let mut current_arg_buffer = String::new();
    let mut in_squote = false;
    let mut in_dquote = false;
    let mut is_escaped = false;

    let mut char_peek_iterator = input.chars().peekable();
    while let Some(current_char) = char_peek_iterator.next() {
        if is_escaped {
            current_arg_buffer.push(current_char);
            is_escaped = false;
        } else if current_char == '\\' {
            if in_squote {
                current_arg_buffer.push(current_char);
            } else if in_dquote {
                if let Some(&next_char) = char_peek_iterator.peek() {
                    if next_char == '"' || next_char == '\\' {
                        is_escaped = true;
                    } else {
                        current_arg_buffer.push(current_char);
                    }
                } else {
                    current_arg_buffer.push(current_char);
                }
            } else {
                is_escaped = true;
            }
        } else if current_char == '\'' && !in_dquote {
            in_squote = !in_squote;
        } else if current_char == '"' && !in_squote {
            in_dquote = !in_dquote;
        } else if current_char == ' ' && !in_squote && !in_dquote {
            if !current_arg_buffer.is_empty() {
                result_args.push(current_arg_buffer.clone());
                current_arg_buffer.clear();
            }
        } else {
            current_arg_buffer.push(current_char);
        }
    }
    if !current_arg_buffer.is_empty() {
        result_args.push(current_arg_buffer);
    }
    return result_args;
}

fn handle_redirect(filepath: &String, filecontent: &[u8]) {
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

fn handle_redirect_append(filepath: &String, filecontent: &[u8]) {
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

fn get_filepath(args: &[String]) -> Option<&String> {
    let position = args.iter().position(|s| s.parse::<Redirect>().is_ok());
    let filepath = if let Some(position) = position {
        args.get(position + 1)
    } else {
        None
    };
    return filepath;
}

fn get_path_directories() -> Vec<std::path::PathBuf> {
    return env::var_os("PATH")
        .unwrap_or_default()
        .to_string_lossy()
        .split(':')
        .map(std::path::PathBuf::from)
        .collect();
}

fn is_executable(path: &std::path::Path) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.is_file() && (metadata.permissions().mode() & 0o111 != 0),
        Err(_) => false,
    }
}

fn find_executable(char_slice: &str) -> Vec<String> {
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

fn auto_complete(buffer: &str) -> (Option<String>, Vec<String>) {
    let command_matched: Vec<String> = BUILTIN_COMMANDS
        .iter()
        .copied()
        .filter(|cmd| cmd.starts_with(buffer))
        .map(|s| s.to_string())
        .collect();
    if command_matched.len() == 1 {
        return (Some(command_matched[0].clone()), command_matched);
    } else {
        let matches = find_executable(&buffer);
        if !matches.is_empty() {
            return (Some(matches[0].clone()), matches);
        } else {
            (None, Vec::new())
        }
    }
}

fn print_all_exec(executables: Vec<String>) {
    for exe in &executables {
        print!("{}  ", exe);
    }
    print!("\r\n");
}

fn main() {
    loop {
        print!("\r$ ");
        io::stdout().flush().unwrap();

        let input = read_inputs_keypress();

        let shell_command: Vec<&str> = input.trim().split_whitespace().collect();
        if shell_command.is_empty() {
            //println!("{}: command not found", input.trim());
            continue;
        } else {
            let args = if shell_command.len() == 1 {
                &[""]
            } else {
                &shell_command[1..]
            };
            match shell_command[0] {
                "exit" => break,
                "echo" => echo_command(input.trim()),
                "type" => type_command(args),
                "pwd" => pwd_command(),
                "cd" => cd_command(&args[0]),
                _ => not_shell_builtin_command(input.trim()),
            }
        }
    }
}

//Terminal Functions
fn read_inputs_keypress() -> String {
    enable_raw_mode().unwrap();
    let mut buffer = String::new();
    let mut tab_count = false;
    loop {
        if let Event::Key(key) = event::read().unwrap() {
            match key {
                KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('j'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }
                | KeyEvent {
                    code: KeyCode::Char('m'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    print!("\r\n");
                    io::stdout().flush().unwrap();
                    break;
                }

                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    if !buffer.is_empty() {
                        buffer.pop();
                        print!("\x1b[D\x1b[K");
                        io::stdout().flush().unwrap();
                    }
                }

                KeyEvent {
                    code: KeyCode::Tab, ..
                } => {
                    if buffer.is_empty() {
                        println!("\x07");
                        redraw_entire_line("$ ", &buffer);
                        tab_count = false;
                        continue;
                    }
                    let (string_option, string_vector) = auto_complete(&buffer);
                    match string_vector.len() {
                        0 => {
                            println!("\x07");
                            redraw_entire_line("$ ", &buffer);
                            tab_count = false;
                        }
                        1 => {
                            if let Some(complete_command) = string_option {
                                buffer = complete_command;
                                buffer.push(' ');
                                redraw_entire_line("$ ", &buffer);
                                tab_count = false;
                            }
                        }
                        _ => {
                            if tab_count == false {
                                print!("\x07");
                                redraw_entire_line("$ ", &buffer);
                                tab_count = true;
                                continue;
                            }
                            print!("\r\n");
                            print_all_exec(string_vector);
                            redraw_entire_line("$ ", &buffer);
                            tab_count = false;
                            continue;
                        }
                    }
                }

                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: m,
                    ..
                } if m.is_empty() || m == KeyModifiers::SHIFT => {
                    buffer.push(c);
                    print!("{}", c);
                    io::stdout().flush().unwrap();
                }

                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    print!("^c\r\n");
                    disable_raw_mode().unwrap();
                    std::process::exit(0);
                }

                _ => {}
            }
        }
    }
    disable_raw_mode().unwrap();
    return buffer;
}

fn redraw_entire_line(prompt: &str, buffer: &str) {
    print!("\r\x1b[K{}{}", prompt, buffer);
    io::stdout().flush().unwrap();
}

//Shell Command Functions
fn pwd_command() {
    let current_path = env::current_dir();
    match current_path {
        Ok(path) => print!("{}\n", path.display()),
        Err(_) => print!("\n"),
    }
}

fn echo_command(input: &str) {
    let full_parsed_command = input_line_parsing(input);

    let args = &full_parsed_command[1..];
    let filepath = get_filepath(args);
    let symbol_position = args
        .iter()
        .position(|s| s.parse::<Redirect>().is_ok())
        .unwrap_or(args.len());

    let write_to_file = args[..symbol_position].join(" ");
    if args.get(symbol_position).map(|s| s == "2>" || s == "2>>") == Some(true) {
        if let Some(filepath) = filepath {
            File::create(filepath).unwrap();
        }
        print!("{}\n", write_to_file);
        return;
    }
    if let Some(filepath) = filepath {
        if args[symbol_position] == ">" || args[symbol_position] == "1>" {
            handle_redirect(filepath, write_to_file.as_bytes());
            return;
        }
        handle_redirect_append(filepath, write_to_file.as_bytes());
    } else {
        print!("{}\n", write_to_file);
    }
    return;
}

fn type_command(args: &[&str]) {
    if !args.is_empty() && BUILTIN_COMMANDS.contains(&args[0]) {
        print!("{} is a shell builtin\n", args[0]);
        return;
    } else if !args.is_empty() {
        if let Some(path) = find_executable_in_path(&args[0]) {
            print!("{} is {}\n", args[0], path.display());
            return;
        }
    }
    print!("{}: not found\n", args[0]);
}

//This is some of the worse code ever written. :)
//Dont let this judge my big personality. <3

fn not_shell_builtin_command(input: &str) {
    let full_parsed_command = input_line_parsing(input);
    let command = &full_parsed_command[0];
    if full_parsed_command.len() > 1 {
        let args = &full_parsed_command[1..];
        let filepath = get_filepath(args);
        let symbol_position = args
            .iter()
            .position(|s| s.parse::<Redirect>().is_ok())
            .unwrap_or(args.len());

        if full_parsed_command.is_empty() {
            print!("{}: command not found\n", input);
        } else {
            let output = Command::new(command)
                .args(&args[..symbol_position])
                .output();
            match output {
                Ok(output) => {
                    let stdout_str = &output.stdout;
                    let stderr_str = &output.stderr;
                    let redirect_symbol = args.get(symbol_position).map(String::as_str);

                    match redirect_symbol {
                        Some(">") | Some("1>") => {
                            if let Some(filepath) = filepath {
                                handle_redirect(filepath, stdout_str);
                            } else {
                                //or else if !stdout_str.is_empty()
                                print!("{}\n", String::from_utf8_lossy(stdout_str).trim());
                                return;
                            }
                            if !stderr_str.is_empty() {
                                eprint!("{}\n", String::from_utf8_lossy(stderr_str).trim());
                            }
                        }
                        Some("2>") => {
                            if let Some(filepath) = filepath {
                                handle_redirect(filepath, stderr_str);
                            } else {
                                //or else if !stderr_str.is_empty()
                                eprint!("{}\n", String::from_utf8_lossy(stderr_str).trim());
                                return;
                            }
                            if !stdout_str.is_empty() {
                                print!("{}\n", String::from_utf8_lossy(stdout_str).trim());
                            }
                        }
                        Some(">>") | Some("1>>") => {
                            if let Some(filepath) = filepath {
                                handle_redirect_append(filepath, stdout_str);
                            } else {
                                print!("{}\n", String::from_utf8_lossy(stdout_str).trim());
                                return;
                            }
                            if !stderr_str.is_empty() {
                                eprint!("{}\n", String::from_utf8_lossy(stderr_str).trim());
                            }
                        }
                        Some("2>>") => {
                            if let Some(filepath) = filepath {
                                handle_redirect_append(filepath, stderr_str);
                            } else {
                                //or else if !stderr_str.is_empty()
                                eprint!("{}\n", String::from_utf8_lossy(stderr_str).trim());
                                return;
                            }
                            if !stdout_str.is_empty() {
                                print!("{}\n", String::from_utf8_lossy(stdout_str).trim());
                            }
                        }
                        _ => {
                            if !stdout_str.is_empty() {
                                print!("{}\n", String::from_utf8_lossy(stdout_str).trim());
                            }
                            if !stderr_str.is_empty() {
                                eprint!("{}\n", String::from_utf8_lossy(stderr_str).trim());
                            }
                        }
                    }
                }
                Err(_e) => eprint!("{}: command not found\n", command),
            }

            return;
        }
    }
    println!("{}: command not found", command);
}

fn cd_command(ab_path: &str) {
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
