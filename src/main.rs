#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::process::Command;
use pathsearch::find_executable_in_path;

const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type"];
fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let shell_command: Vec<&str> = input.trim().split_whitespace().collect();
        if shell_command.is_empty(){
            println!("{}: command not found", input.trim());
        }else{
            let args = &shell_command[1..];
    
            match shell_command[0]{
                "exit" => break,
                "echo" => echo_command(args),
                "type" => type_command(args),
                "pwd" => pwd_command(),
                _ => check_command(&input),
            }
        }
    }
}


fn pwd_command(){
    let current_path = env::current_dir();
    match current_path{
        Ok(path) => println!("{}", path.display()),
        Err(_) => println!(""),
    }
}


fn echo_command(args: &[&str]){
    println!("{}", args.join(" "));
}

fn type_command(args: &[&str]){
    if !args.is_empty() && BUILTIN_COMMANDS.contains(&args[0]){
        println!("{} is a shell builtin", args[0]);
        return;
    }else if !args.is_empty(){
        if let Some(path) = find_executable_in_path(&args[0]){
            println!("{} is {}", args[0], path.display());
            return;
        }
    }
    println!("{}: not found", args[0]);
}


fn check_command(cmd: &str){
    let command_split: Vec<&str> = cmd.split_whitespace().collect();
    if command_split.len() > 1{
        let dynamic_args = &command_split[1..];
        if command_split.is_empty(){
            println!("{}: command not found", cmd);
        }else{
            let mut output = Command::new(command_split[0])
                .args(dynamic_args)
                .spawn()
                .unwrap();
            output.wait().unwrap();
            return;
        }
    }
    println!("{}: command not found", &command_split[0])
}


