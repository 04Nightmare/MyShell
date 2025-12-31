use std::fs::{self, File};
use std::io::{BufReader, Read};

use crate::utilities::input_parser::input_line_parsing;
use crate::utilities::redirect::{handle_redirect, handle_redirect_append};

pub fn history_command(input: &str) {
    let parsed_history_commands = input_line_parsing(input);
    let args = &parsed_history_commands[1..];
    let outer_file = File::open("history.txt");
    match outer_file {
        Ok(outer_file) => {
            let mut reader = BufReader::new(outer_file);
            let mut outer_contents = String::new();
            reader.read_to_string(&mut outer_contents).unwrap();
            let lines: Vec<String> = outer_contents
                .trim()
                .lines()
                .map(|l| l.to_string())
                .collect();

            if args.is_empty() {
                for (i, line) in lines.iter().enumerate() {
                    println!("    {} {}", i + 1, line);
                }
                return;
            }

            match args[0].as_str() {
                "-r" => {
                    if args.len() == 1 {
                        return;
                    }
                    let file = File::open(&args[1]);
                    match file {
                        Ok(file) => {
                            let mut contents = String::new();
                            let mut buf_read = BufReader::new(file);
                            buf_read.read_to_string(&mut contents).unwrap();
                            let temp = "history.txt".to_string();
                            handle_redirect_append(&temp, contents.as_bytes());
                        }
                        Err(_) => {
                            eprintln!("Cannot open file");
                            return;
                        }
                    }
                }
                "-w" => {
                    if args.len() == 1 {
                        return;
                    }
                    let file = File::open("history.txt");
                    match file {
                        Ok(file) => {
                            let mut contents = String::new();
                            let mut buf_read = BufReader::new(file);
                            buf_read.read_to_string(&mut contents).unwrap();
                            handle_redirect(&args[1], contents.as_bytes());
                        }
                        Err(_) => {
                            eprintln!("Cannot open file");
                            return;
                        }
                    }
                }
                "-a" => {
                    if args.len() == 1 {
                        return;
                    }
                    let file = File::open("history.txt");
                    match file {
                        Ok(file) => {
                            let mut contents = String::new();
                            let mut buf_read = BufReader::new(file);
                            buf_read.read_to_string(&mut contents).unwrap();
                            handle_redirect_append(&args[1], contents.as_bytes());
                        }
                        Err(_) => {
                            eprintln!("Cannot open file");
                            return;
                        }
                    }
                    let remove = fs::remove_file("history.txt");
                    match remove {
                        Ok(remove) => remove,
                        Err(_) => {}
                    }
                }
                val => match val.parse::<usize>() {
                    Ok(n) => {
                        if let Some(start_line) = lines.len().checked_sub(n) {
                            let mut strt_idx = start_line;
                            for line in lines[start_line..].iter() {
                                strt_idx += 1;
                                println!("    {} {}", strt_idx, line);
                            }
                        } else {
                            for (i, line) in lines.iter().enumerate() {
                                println!("    {} {}", i + 1, line);
                            }
                            return;
                        };
                    }
                    Err(_) => {
                        eprintln!("Unknown argument: {}", val);
                    }
                },
            }
        }
        Err(_) => eprint!("cant open file"),
    }
}
