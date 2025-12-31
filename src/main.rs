mod shell;
mod terminal;
mod utilities;

#[allow(unused_imports)]
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::process::{ChildStdout, Command, Stdio};

//Shell Command Functions
use crate::shell::{
    c_type::type_command, cd::cd_command, diff_command::not_shell_builtin_command,
    echo::echo_command, history::history_command, pwd::pwd_command,
};
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

//Terminal Functions
use crate::terminal::read_keypress::read_inputs_keypress;

//Utility Functions
use crate::utilities::{
    executable::find_executable, input_parser::input_line_parsing, redirect::handle_redirect,
};
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

const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd", "history"];

//Main Shell entry.
fn main() -> std::io::Result<()> {
    let remove = fs::remove_file("history.txt");
    match remove {
        Ok(remove) => remove,
        Err(_) => {}
    }
    if let Ok(histfile) = std::env::var("HISTFILE") {
        let file = File::open(histfile);
        match file {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                let temp = "history.txt".to_string();
                handle_redirect(&temp, contents.as_bytes());
            }
            Err(_) => {}
        }
    }
    loop {
        print!("\r$ ");
        io::stdout().flush().unwrap();

        let input = read_inputs_keypress();

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
                    if let Ok(histfile) = std::env::var("HISTFILE") {
                        let file = File::open("history.txt");
                        match file {
                            Ok(mut file) => {
                                let mut contents = String::new();
                                file.read_to_string(&mut contents).unwrap();
                                handle_redirect(&histfile, contents.as_bytes());
                            }
                            Err(_) => {}
                        }
                    }
                    std::process::exit(0);
                }
                "echo" => echo_command(input.trim()),
                "type" => type_command(input.trim()),
                "pwd" => pwd_command(),
                "cd" => cd_command(&args[0]),
                "history" => history_command(input.trim()),
                _ => not_shell_builtin_command(input.trim()),
            }
        }
    }
}
