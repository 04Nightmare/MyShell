#[allow(unused_imports)]
use std::io::{self, Write};

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
            let mut found = false;
            for item in &builtin_commands{
                if item == &arg{
                    println!("{} is a shell builtin", arg);
                    found = true;
                    break;
                }
            }
            if !found{
                println!("{}: not found", arg);
            }
        }else{
            println!("{}: command not found", command.trim());
        }
    }
}


