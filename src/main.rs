#[allow(unused_imports)]
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
//use anyhow::Ok;
use pathsearch::find_executable_in_path;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{ChildStdout, Command, Stdio};
use std::str::FromStr;
use std::{env, fs};

const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd", "history"];

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

fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let mut prefix = strings[0].clone();
    for s in strings.iter().skip(1) {
        let mut i = 0;
        let max = prefix.len().min(s.len());
        while i < max && prefix.as_bytes()[i] == s.as_bytes()[i] {
            i += 1;
        }
        prefix.truncate(i);
        if prefix.is_empty() {
            break;
        }
    }
    return prefix;
}

fn main() -> std::io::Result<()> {
    let mut history_count = 0;
    loop {
        print!("\r$ ");
        io::stdout().flush().unwrap();

        history_count += 1;
        let input = read_inputs_keypress(history_count);

        // let pipeline_input: Vec<String> = input.split("|").map(|s| s.trim().to_string()).collect();
        // if pipeline_input.len() > 1 {
        //     println!("This is pipeline input");
        //     continue;
        // }

        if input.contains("|") {
            pipe_command(&input);
            continue;
        }

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
                "exit" => {
                    fs::remove_file("history.txt")?;
                    std::process::exit(0)
                }
                "echo" => echo_command(input.trim()),
                "type" => type_command(input.trim()),
                "pwd" => pwd_command(),
                "cd" => cd_command(&args[0]),
                "history" => history_command(&args[0]),
                _ => not_shell_builtin_command(input.trim()),
            }
        }
    }
}

//Terminal Functions
fn read_inputs_keypress(history_count: i32) -> String {
    enable_raw_mode().unwrap();
    let mut buffer = String::new();
    let mut tab_flip = false;
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
                    let formatted_buffer = format!("{} {}", history_count, buffer);
                    handle_redirect_append(
                        &String::from("history.txt"),
                        formatted_buffer.as_bytes(),
                    );
                    io::stdout().flush().unwrap();
                    break;
                }

                KeyEvent {
                    code: KeyCode::Backspace,
                    ..
                } => {
                    if !buffer.is_empty() {
                        tab_flip = false;
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
                        tab_flip = false;
                        continue;
                    }
                    let (string_option, string_vector) = auto_complete(&buffer);
                    match string_vector.len() {
                        0 => {
                            println!("\x07");
                            redraw_entire_line("$ ", &buffer);
                            tab_flip = false;
                        }
                        1 => {
                            if let Some(complete_command) = string_option {
                                buffer = complete_command;
                                buffer.push(' ');
                                redraw_entire_line("$ ", &buffer);
                                tab_flip = false;
                            }
                        }
                        _ => {
                            let common = longest_common_prefix(&string_vector);
                            if common.len() > buffer.len() {
                                buffer = common;
                                redraw_entire_line("$ ", &buffer);
                                tab_flip = false;
                            } else {
                                if tab_flip == false {
                                    print!("\x07");
                                    redraw_entire_line("$ ", &buffer);
                                    tab_flip = true;
                                } else {
                                    print!("\r\n");
                                    print_all_exec(string_vector);
                                    redraw_entire_line("$ ", &buffer);
                                    tab_flip = false;
                                }
                            }
                            // if tab_flip == false {
                            //     print!("\x07");
                            //     redraw_entire_line("$ ", &buffer);
                            //     tab_flip = true;
                            //     continue;
                            // }
                            // print!("\r\n");
                            // print_all_exec(string_vector);
                            // redraw_entire_line("$ ", &buffer);
                            // tab_flip = false;
                            // continue;
                        }
                    }
                }

                KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: m,
                    ..
                } if m.is_empty() || m == KeyModifiers::SHIFT => {
                    tab_flip = false;
                    buffer.push(c);
                    print!("{}", c);
                    io::stdout().flush().unwrap();
                }

                KeyEvent {
                    code: KeyCode::Char('c'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                } => {
                    tab_flip = false;
                    print!("^C\r\n");
                    buffer.clear();
                    redraw_entire_line("$ ", &buffer);
                    //disable_raw_mode().unwrap();
                    //continue;
                    //std::process::exit(0);
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

fn type_command(input: &str) {
    let full_parsed_command = input_line_parsing(input);
    let args = &full_parsed_command[1];
    if !args.is_empty() && BUILTIN_COMMANDS.contains(&args.as_str()) {
        print!("{} is a shell builtin\n", args);
        return;
    } else if !args.is_empty() {
        if let Some(path) = find_executable_in_path(&args) {
            print!("{} is {}\n", args, path.display());
            return;
        }
    }
    print!("{}: not found\n", args);
}

fn history_command(args: &str) {
    let file = File::open("history.txt");
    match file {
        Ok(file) => {
            let reader = BufReader::new(file);
            let lines: Vec<String> = reader.lines().flatten().collect();

            if args.is_empty() {
                for line in lines {
                    println!("{}", line);
                }
                return;
            }

            let n: usize = match args.parse() {
                Ok(n) => n,
                Err(_) => {
                    eprintln!("History: {}: numeric argument needed", args);
                    return;
                }
            };

            let start_line = lines.len().saturating_sub(n);
            for line in &lines[start_line..] {
                println!("{}", line);
            }
        }
        Err(_) => eprint!("cant open file"),
    }
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

fn pipe_command(input: &str) {
    let pipeline_input: Vec<String> = input.split("|").map(|s| s.trim().to_string()).collect();
    let mut previous_stdout: Option<ChildStdout> = None;
    let mut child_processes = Vec::new();
    let pipeline_input_length = pipeline_input.len();
    for cmd_string in &pipeline_input {
        let parsed = input_line_parsing(cmd_string);
        let Some((program, args)) = parsed.split_first() else {
            return;
        };
        if program.trim().is_empty() {
            return;
        }
        match program.trim() {
            "type" => type_command(cmd_string),
            "exit" => {
                std::process::exit(0);
            }
            "cd" => cd_command(program),
            _ => {
                let mut output = Command::new(program);
                output.args(args);
                if let Some(prev_stdout) = previous_stdout.take() {
                    output.stdin(Stdio::from(prev_stdout));
                }
                if cmd_string != &pipeline_input[pipeline_input_length - 1] {
                    output.stdout(Stdio::piped());
                }

                let mut child = match output.spawn() {
                    Ok(child) => child,
                    Err(_) => {
                        eprintln!("{}: command not found", parsed[0]);
                        return;
                    }
                };

                previous_stdout = child.stdout.take();
                child_processes.push(child);
            }
        }
    }

    for mut child in child_processes {
        let _ = child.wait();
    }
}
