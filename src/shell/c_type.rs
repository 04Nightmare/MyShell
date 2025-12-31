use pathsearch::find_executable_in_path;

use crate::{BUILTIN_COMMANDS, utilities::input_parser::input_line_parsing};

pub fn type_command(input: &str) {
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
