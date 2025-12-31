use std::fs::File;

use crate::utilities::{
    input_parser::input_line_parsing,
    redirect::{Redirect, get_filepath, handle_redirect, handle_redirect_append},
};

pub fn echo_command(input: &str) {
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
