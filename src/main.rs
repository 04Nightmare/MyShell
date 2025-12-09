#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop{
        print!("$ ");
        io::stdout().flush().unwrap();
    
        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        if command.trim() == "exit" {
            break;
        }else if command.trim().starts_with("echo") {
            println!("{}",command.split_off(5).trim());
        }else{
            println!("{}: command not found", command.trim());
        }
    }
}





  // let mut argument = String::new();
        // io::stdin().read_line(&mut argument).unwrap();
        // match command.trim() {
        //     "exit" => break,
        //     "echo" => println!("{}",argument.trim()),
        //     _ => println!("{}: command not found", command.trim()),
        // }