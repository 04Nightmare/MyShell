#[allow(unused_imports)]
use std::io::{self, Write};
use std::env;
use std::path::Path;
use std::process::Command;
use pathsearch::find_executable_in_path;

const BUILTIN_COMMANDS: &[&str] = &["exit", "echo", "type", "pwd", "cd"];
fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        let shell_command: Vec<&str> = input.trim().split_whitespace().collect();
        if shell_command.is_empty(){
            //println!("{}: command not found", input.trim());
        }else{
            let args = if shell_command.len() == 1{
                &[""]
            }else{
                &shell_command[1..]
            };
            match shell_command[0]{
                "exit" => break,
                "echo" => echo_command(args),
                "type" => type_command(args),
                "pwd" => pwd_command(),
                "cd" => cd_command(&args[0]),
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

fn cd_command(ab_path: &str){
    if ab_path == "~"{
        let home_dir = env::home_dir();
        match home_dir{
            Some(home_dir_path) => {
                let home_dir_path_string = home_dir_path.display().to_string();
                let cd_path = Path::new(&home_dir_path_string);
                match env::set_current_dir(&cd_path){
                    Ok(_) => {},
                    Err(_) => println!("cd: can't get to home directory"),
                }
            },
            None => println!("cd: can't get to home directory"),
        }
    }else{
        let cd_path = Path::new(ab_path);
        let is_path = env::set_current_dir(&cd_path);
        match is_path{
            Ok(_) => {},
            Err(_) => println!("cd: {}: No such file or directory", ab_path),
        }
    }
}
