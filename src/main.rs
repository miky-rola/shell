#[allow(unused_imports)]
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write, Read};
use std::path::{Path, PathBuf};
use std::process::Command;


#[derive(Debug)]
struct Shell {
    current_dir: PathBuf,
    env_vars: HashMap<String, String>,
    builtins: HashMap<String, fn(&mut Shell, &[String]) -> io::Result<()>>,
}


impl Shell {
    fn new() -> io::Result<Shell> {
        println!("Initializing shell...");
        let mut builtins = HashMap::new();
        builtins.insert("cd".to_string(), Shell::cd as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("echo".to_string(), Shell::echo as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("pwd".to_string(), Shell::pwd as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("type".to_string(), Shell::type_cmd as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("ls".to_string(), Shell::ls as fn(&mut Shell, &[String]) -> io::Result<()>);

        let current_dir = env::current_dir()?;
        println!("Starting in directory: {}", current_dir.display()); 

        Ok(Shell {
            current_dir: env::current_dir()?,
            env_vars: env::vars().collect(),
            builtins,
        })
    }


    fn run(&mut self) -> io::Result<()> {
        println!("Shell is running. Type 'exit' to quit."); 
        
        loop {
            print!("{} $ ", self.current_dir.display());
            if let Err(e) = io::stdout().flush() {
                eprintln!("Error flushing stdout: {}", e);
                continue;
            }

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(n) => {
                    println!("Read {} bytes from stdin", n); 
                    if n == 0 {
                        println!("Received EOF (Ctrl+D), exiting...");
                        break;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading from stdin: {}", e);
                    continue;
                }
            }


            let input = input.trim();
            println!("Received input: '{}'", input); 

            if input.is_empty() {
                continue;
            }

            if input == "exit" {
                println!("Exit command received, shutting down..."); 
                break;
            }

            if let Err(e) = self.execute_command(input) {
                eprintln!("Error executing command: {}", e);
            }
        }
        Ok(())
    }

    
    fn parse_command(&self, input: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current_token = String::new();
        let mut in_quotes = false;
        let mut escaped = false;

        for c in input.chars() {
            match (c, in_quotes, escaped) {
                ('\\', _, false) => escaped = true,
                ('"', _, true) => {
                    current_token.push('"');
                    escaped = false;
                }
                ('"', _, false) => {
                    in_quotes = !in_quotes;
                }
                (' ', false, false) => {
                    if !current_token.is_empty() {
                        tokens.push(current_token.clone());
                        current_token.clear();
                    }
                }
                (c, _, true) => {
                    current_token.push(c);
                    escaped = false;
                }
                (c, _, false) => current_token.push(c),
            }
        }

        if !current_token.is_empty() {
            tokens.push(current_token);
        }

        tokens
    }

    fn execute_command(&mut self, input: &str) -> io::Result<()> {
        let tokens = self.parse_command(input);
        if tokens.is_empty() {
            return Ok(());
        }

        let command = &tokens[0];
        let args = &tokens[1..];

        // Check for redirection
        let (args, output_file) = self.check_redirection(args);

        if let Some(builtin) = self.builtins.get(command) {
            builtin(self, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;
        } else {
            self.execute_external_command(command, &args, output_file)?;
        }

        Ok(())
    }

    fn check_redirection(&self, args: &[String]) -> (Vec<String>, Option<String>) {
        let mut new_args = Vec::new();
        let mut output_file = None;
        let mut i = 0;

        while i < args.len() {
            if args[i] == ">" && i + 1 < args.len() {
                output_file = Some(args[i + 1].clone());
                i += 2;
            } else {
                new_args.push(args[i].clone());
                i += 1;
            }
        }

        (new_args, output_file)
    }

    // Builtin Commands
    fn cd(&mut self, args: &[String]) -> io::Result<()> {
        let new_dir = args.get(0).map(|s| Path::new(s)).unwrap_or_else(|| Path::new("/"));
        env::set_current_dir(new_dir)?;
        self.current_dir = env::current_dir()?;
        Ok(())
    }

    fn echo(&mut self, args: &[String]) -> io::Result<()> {
        println!("{}", args.join(" "));
        Ok(())
    }

    fn pwd(&mut self, _args: &[String]) -> io::Result<()> {
        println!("{}", self.current_dir.display());
        Ok(())
    }

    fn ls(&mut self, args: &[String]) -> io::Result<()> {
        let path = args.get(0).map(|s| Path::new(s)).unwrap_or(&self.current_dir);
        
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = entry.file_name();
            print!("{}\t", name.to_string_lossy());
        }
        println!();
        Ok(())
    }

    fn type_cmd(&mut self, args: &[String]) -> io::Result<()> {
        if args.is_empty() {
            println!("type: missing command name");
            return Ok(());
        }

        let cmd = &args[0];
        if self.builtins.contains_key(cmd) {
            println!("{} is a shell builtin", cmd);
        } else {
            // Check if it's an external command in PATH
            if let Ok(path) = which::which(cmd) {
                println!("{} is {}", cmd, path.display());
            } else {
                println!("{} not found", cmd);
            }
        }
        Ok(())
    }

    fn execute_external_command(&self, command: &str, args: &[String], output_file: Option<String>) -> io::Result<()> {
        let output = Command::new(command)
            .args(args)
            .current_dir(&self.current_dir)
            .envs(&self.env_vars)
            .output()?;

        if let Some(file_path) = output_file {
            let mut file = File::create(file_path)?;
            file.write_all(&output.stdout)?;
        } else {
            io::stdout().write_all(&output.stdout)?;
            io::stderr().write_all(&output.stderr)?;
        }

        Ok(())
    }
}

fn main() -> io::Result<()> {
    println!("Starting shell application..."); 
    let mut shell = Shell::new()?;
    println!("Shell created successfully, entering main loop..."); 
    shell.run()
}