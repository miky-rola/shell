// use std::{io::{stdin, Stdin}, os::windows::process, process::Command};
#[allow(unused_imports)]
use std::io::{self, Write};
// use std::process;

// fn not_found(command: &str){
//     println!("{}: command not found", command);
// }

// fn tokenize(input: &str) -> Vec<&str>{
//     input.split(' ').collect()
// }

// fn handle_command(input: &str) -> bool {
//     let parts: Vec<&str> = input.split_whitespace().collect();

//     if parts.is_empty() {
//         return false; // No command provided
//     }

//     match parts[0] {
//         "exit" => {
//             if parts.len() == 2 && parts[1] == "0" {
//                 return true; // Exit the REPL
//             } else {
//                 println!("{}: command not found", input);
//             }
//         }
//         "echo" => {
//             if input.len() > 5 {
//                 println!("{}", &input[5..]); // Print everything after "echo "
//             } else {
//                 println!(); // Handle "echo" with no arguments
//             }
//         }
//         "type" => {
//             if parts.len() > 1 {
//                 if parts[1] == "exit" || parts[1] == "echo" || parts[1] == "type" {
//                     println!("{} is a shell builtin", parts[1]);
//                 } else {
//                     println!("{}: not found", parts[1]);
//                 }
//             } else {
//                 println!("type: usage: type [name]"); // Handle missing arguments
//             }
//         }
//         _ => {
//             println!("{}: command not found", input);
//         }
//     }

//     false
// }


// fn main(){
//     print!("$ ");
//     io::stdout().flush().unwrap();

//     loop {
//         // let stdin = io::stdin();
//         let mut input = String::new();
//         // stdin.read_line(&mut input).unwrap();
//         // let command = input.trim();
//         // let token = tokenize(command);
//         io::stdin().read_line( &mut input).unwrap();
//         let input = input.trim();

//         if handle_command(input){
//             break;
//         }
//         // match token[..]{
//         //     ["exit", code] => process::exit(code.parse::<i32>().unwrap()),
//         //     ["echo", ..] => println!("{}", token[1..].join(" ")),
//         //     _ => not_found(command),
//         // }
//         print!("$ ");
//         io::stdout().flush().unwrap();
//     }
// }


// 
fn main() {
    let stdin = io::stdin();
    let path_env = std::env::var("PATH").unwrap();
    let path_separator = if cfg!(windows) { ';' } else { ':' };
    let builtins = ["exit", "echo", "type"];

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        let argv = input.split_whitespace().collect::<Vec<&str>>();
        if argv.is_empty() {
            continue;
        }

        match argv[0] {
            "exit" => break,
            "echo" => {
                println!("{}", argv[1..].join(" "));
            }
            "type" => {
                if argv.len() != 2 {
                    println!("type: expected 1 argument, got {}", argv.len() - 1);
                    continue;
                }
                let cmd = argv[1];
                if builtins.contains(&cmd) {
                    println!("{} is a shell builtin", cmd);
                } else {
                    let mut paths = path_env.split(path_separator);
                    if let Some(path) =
                        paths.find(|path| std::fs::metadata(format!("{}/{}", path, cmd)).is_ok())
                    {
                        println!("{} is {}/{}", cmd, path, cmd);
                    } else {
                        println!("{} not found", cmd);
                    }
                }
            }
            _ => {
                println!("{}: command not found", input.trim())
            }
        }
    }
}

// fn main() {
//     // Uncomment this block to pass the first stage
//     // print!("$ ");
//     // io::stdout().flush().unwrap();

//     // Wait for user input
//     let stdin: io::Stdin = io::stdin();
//     let mut input: String = String::new();
//     // stdin.read_line(&mut input).unwrap();
//     // println!("{}: command not found", input.trim());
//     loop {
//         let mut input:String = String::new();
//         print!("$ ");
//         io::stdout().flush().unwrap();

//         stdin.read_line(&mut input).unwrap();
//         let input: &str = input.trim();
        
//         println!("{}: command not found", input.trim());
//         println!("Exit 0");
//         io::stdout().flush().unwrap();
//         break;

//     }
// }

// fn main() {
//     let Stdin = io::stdin();
//     let mut input = String::new();

//     loop {
//         input.clear();
//         print!("$ ");
//         io::stdout().flush().unwrap();

//         stdin().read_line( &mut input).unwrap();

//         match input.trim() {
//             "exit 0" => break,
//             &_ => {
//                 print!("{}: not found\n", input.trim())
//             }
//         }
//     }
// }