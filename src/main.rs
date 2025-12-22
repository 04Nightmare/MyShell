use pathsearch::find_executable_in_path;
use std::env;
use std::fs::File;
#[allow(unused_imports)]
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];
fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let shell_command: Vec<&str> = input.trim().split_whitespace().collect();
        if shell_command.is_empty() {
            //println!("{}: command not found", input.trim());
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
    if filepath == &">".to_string() {
        println!("no filename");
        return;
    }
    let file = File::create(filepath);
    match file {
        Ok(mut file) => {
            file.write_all(filecontent).unwrap();
        }
        Err(_) => {
            println!("cant create file");
        }
    }
}

fn pwd_command() {
    let current_path = env::current_dir();
    match current_path {
        Ok(path) => println!("{}", path.display()),
        Err(_) => println!(""),
    }
}

fn echo_command(input: &str) {
    let full_parsed_command = input_line_parsing(input);
    let _command = &full_parsed_command[0];
    let args = &full_parsed_command[1..];
    let filepath = &args[args.len() - 1];
    let print_upto_symbol = args
        .iter()
        .position(|symbol| symbol == ">" || symbol == "1>")
        .unwrap_or(args.len());
    if full_parsed_command
        .iter()
        .any(|com| com == ">" || com == "1>")
    {
        let write_to_file = args[..print_upto_symbol].join(" ");
        handle_redirect(filepath, write_to_file.as_bytes());
        return;
    }
    println!("{}", args[..print_upto_symbol].join(" "));
}

fn type_command(args: &[&str]) {
    if !args.is_empty() && BUILTIN_COMMANDS.contains(&args[0]) {
        println!("{} is a shell builtin", args[0]);
        return;
    } else if !args.is_empty() {
        if let Some(path) = find_executable_in_path(&args[0]) {
            println!("{} is {}", args[0], path.display());
            return;
        }
    }
    println!("{}: not found", args[0]);
}

fn not_shell_builtin_command(input: &str) {
    let full_parsed_command = input_line_parsing(input);
    let command = &full_parsed_command[0];
    if full_parsed_command.len() > 1 {
        let args = &full_parsed_command[1..];
        let filepath = &args[args.len() - 1];
        let args_upto_symbol = args
            .iter()
            .position(|symbol| symbol == ">" || symbol == "1>")
            .unwrap_or(args.len());
        if full_parsed_command.is_empty() {
            println!("{}: command not found", input);
        } else {
            let output = Command::new(command)
                .args(args[..args_upto_symbol].to_vec())
                .output();
            match output {
                Ok(output) => {
                    let stdout_str = String::from_utf8_lossy(&output.stdout);
                    let stderr_str = String::from_utf8_lossy(&output.stderr);
                    if !output.status.success() {
                        eprint!("{}", stderr_str);
                        return;
                    } else if args.iter().any(|s| s == ">" || s == "1>") {
                        handle_redirect(filepath, stdout_str.as_bytes());
                        return;
                    }
                    io::stdout().write_all(stdout_str.as_bytes()).unwrap();
                }
                Err(_e) => eprintln!("{}: command not found", command),
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
                    Err(_) => println!("cd: can't get to home directory"),
                }
            }
            None => println!("cd: can't get to home directory"),
        }
    } else {
        let cd_path = Path::new(ab_path);
        let is_path = env::set_current_dir(&cd_path);
        match is_path {
            Ok(_) => {}
            Err(_) => println!("cd: {}: No such file or directory", ab_path),
        }
    }
}
