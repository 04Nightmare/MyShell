#[allow(unused_imports)]
use std::io::{self, Write};
use pathsearch::find_executable_in_path;

fn main() {
    let builtin_commands = vec!["exit", "echo", "type"];
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.trim() == "exit" {
            break;
        }else if command.trim().starts_with("echo") {
            println!("{}",command.split_off(5).trim());
        }else if command.trim().starts_with("type"){
            let arg = command.split_off(5).trim().to_string();
            if builtin_commands.contains(&arg.as_str()){
                println!("{} is a shell builtin", arg);
            }else if let Some(path) = find_executable_in_path(&arg){
                println!("{} is {}", arg, path.display());
            }else{
                println!("{}: not found", arg);
            }
        }else{
            println!("{}: command not found", command.trim());
        }
    }
}




