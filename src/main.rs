#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::Command;
use pathsearch::find_executable_in_path;

fn main() {
    let builtin_commands = vec!["exit", "echo", "type"];
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        let trimmed_command = command.trim();
        if trimmed_command == "exit" {
            break;
        }else if trimmed_command.starts_with("echo") {
            println!("{}",command.split_off(5).trim());
        }else if trimmed_command.starts_with("type"){
            let arg = command.split_off(5).trim().to_string();
            if builtin_commands.contains(&arg.as_str()){
                println!("{} is a shell builtin", arg);
            }else if let Some(path) = find_executable_in_path(&arg){
                println!("{} is {}", arg, path.display());
            }else{
                println!("{}: not found", arg);
            }
        }else {
            let command_split: Vec<&str> = trimmed_command.split_whitespace().collect();
            if command_split.is_empty(){
                println!("{}: command not", trimmed_command);
            }else{
                let dynamic_args = &command_split[1..];
                let output = Command::new(command_split[0])
                    .args(dynamic_args)
                    .output();

                match output {
                    Ok(output) => {println!("{:?}",output.stdout.as_slice());}
                    Err(_) => {println!("{}: command not found", trimmed_command)}
                }
                
            }
        }
    }
}




