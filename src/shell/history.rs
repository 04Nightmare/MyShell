use crate::utilities::history::{
    get_history, history_append_to_file, history_read_from_file, history_write_to_file,
};
use crate::utilities::input_parser::input_line_parsing;

pub fn history_command(input: &str) {
    let parsed_history_commands = input_line_parsing(input);
    let args = &parsed_history_commands[1..];
    let lines = get_history();

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
            if let Err(_) = history_read_from_file(&args[1]) {
                eprintln!("Cannot open file");
            }
        }
        "-w" => {
            if args.len() == 1 {
                return;
            }
            if let Err(_) = history_write_to_file(&args[1]) {
                eprintln!("Cannot open file");
            }
        }
        "-a" => {
            if args.len() == 1 {
                return;
            }
            if let Err(_) = history_append_to_file(&args[1]) {
                eprintln!("Cannot open file");
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
