use std::process::Command;

use crate::utilities::{
    input_parser::input_line_parsing,
    redirect::{Redirect, get_filepath, handle_redirect, handle_redirect_append},
};

pub fn not_shell_builtin_command(input: &str) {
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
