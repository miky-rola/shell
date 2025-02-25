#[warn(unused_imports)]
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self, Write, BufReader, BufRead};
use std::path::{Path, PathBuf, Component};
use dirs;
use hostname;
use std::fs;
use glob::Pattern;
use crate::shell_type::ShellType;

pub struct Shell {
    pub shell_type: ShellType,
    pub current_dir: PathBuf,
    pub env_vars: HashMap<String, String>,
    pub builtins: HashMap<String, fn(&mut Shell, &[String]) -> io::Result<()>>,
    pub home_dir: PathBuf,
    pub history: Vec<String>,
    pub history_file: PathBuf,
}

impl Shell {
   
    pub fn new(shell_type: ShellType) -> io::Result<Shell> {
        println!("Initializing {} shell...", match shell_type {
            ShellType::Linux => "Linux",
            ShellType::MacOS => "MacOS",
            ShellType::Windows => "Windows",
        });

        let mut builtins = HashMap::new();
        builtins.insert("cd".to_string(), Shell::cd as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("echo".to_string(), Shell::echo as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("pwd".to_string(), Shell::pwd as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("type".to_string(), Shell::type_cmd as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("ls".to_string(), Shell::ls as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("clear".to_string(), Shell::clear as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("env".to_string(), Shell::env as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("which".to_string(), Shell::which as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("history".to_string(), Shell::history as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("source".to_string(), Shell::source as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("cat".to_string(), Shell::cat as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("mkdir".to_string(), Shell::mkdir as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("touch".to_string(), Shell::touch as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("grep".to_string(), Shell::grep as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("find".to_string(), Shell::find as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("head".to_string(), Shell::head as fn(&mut Shell, &[String]) -> io::Result<()>);
        builtins.insert("tail".to_string(), Shell::tail as fn(&mut Shell, &[String]) -> io::Result<()>);
        
        let current_dir = env::current_dir()?;
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let history_file = home_dir.join(match shell_type {
            ShellType::Windows => ".shell_history.txt",
            _ => ".shell_history",
        });

        let mut shell = Shell {
            shell_type,
            current_dir,
            env_vars: env::vars().collect(),
            builtins,
            home_dir,
            history: Vec::new(),
            history_file,
        };

        shell.load_history()?;
        Ok(shell)
    }

    
    /// The loop continues until 'exit' is entered or EOF is received
    pub fn run(&mut self) -> io::Result<()> {
        println!("Shell is running. Type 'exit' to quit.");
        
        loop {
            print!("{}", self.get_prompt());
            if let Err(e) = io::stdout().flush() {
                eprintln!("Error flushing stdout: {}", e);
                continue;
            }

            let mut input = String::new();
            match io::stdin().read_line(&mut input) {
                Ok(n) => {
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
            if input.is_empty() {
                continue;
            }

            if input == "exit" {
                println!("Exit command received, shutting down...");
                break;
            }

            // Process the command
            self.add_to_history(input);
            if let Err(e) = self.execute_command(input) {
                eprintln!("Error executing command: {}", e);
            }
        }
        Ok(())
    }


    /// - Shell-specific prompt character ($ for Linux, % for MacOS, > for Windows)
    pub fn get_prompt(&self) -> String {
        let display_path = self.format_display_path();
        let default_user = String::from("user");
        let username = self.env_vars.get("USER").unwrap_or(&default_user);
        let hostname = hostname::get().unwrap_or_default().to_string_lossy().to_string();

        match self.shell_type {
            ShellType::Linux => format!("{}@{}:{} $ ", username, hostname, display_path),
            ShellType::MacOS => format!("{}@{}:{} % ", username, hostname, display_path),
            ShellType::Windows => format!("{}> ", display_path),
        }
    }

    /// Formats the current working directory for display in the prompt
    fn format_display_path(&self) -> String {
        let path = self.current_dir.as_path();
        
        if self.shell_type == ShellType::Windows {
            path.display().to_string()
        } else {
            if path.starts_with(&self.home_dir) {
                let remainder = path.strip_prefix(&self.home_dir).unwrap_or(path);
                format!("~{}", remainder.display())
            } else {
                path.display().to_string()
            }
        }
    }

    /// Loads command history from the history file
    fn load_history(&mut self) -> io::Result<()> {
        if self.history_file.exists() {
            let file = File::open(&self.history_file)?;
            let reader = BufReader::new(file);
            for line in reader.lines() {
                if let Ok(command) = line {
                    self.history.push(command);
                }
            }
        }
        Ok(())
    }

    /// Saves the current command history to the history file
    /// 
    /// Writes each command in the history vector to the file
    fn save_history(&self) -> io::Result<()> {
        let mut file = File::create(&self.history_file)?;
        for command in &self.history {
            writeln!(file, "{}", command)?;
        }
        Ok(())
    }
 
    fn add_to_history(&mut self, command: &str) {
        if !command.trim().is_empty() {
            self.history.push(command.to_string());
            if let Err(e) = self.save_history() {
                eprintln!("Error saving history: {}", e);
            }
        }
    }

    /// Normalizes a path by resolving parent directory references (..)
    /// and removing redundant components
    /// Returns a cleaned up PathBuf
    pub fn normalize_path(&self, path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                Component::ParentDir => { components.pop(); }
                Component::Normal(name) => components.push(name.to_owned()),
                Component::RootDir => { components.clear(); components.push(std::ffi::OsString::from("/")); }
                _ => {}
            }
        }
        components.iter().collect()
    }

    pub fn find(&mut self, args: &[String]) -> io::Result<()> {
        let dir = args.get(0).map(Path::new).unwrap_or_else(|| Path::new("."));
        let pattern = args.get(1).map(|s| Pattern::new(s).ok()).flatten();

        Self::find_recursive(dir, pattern.as_ref())?;
        Ok(())
    }

    fn find_recursive(dir: &Path, pattern: Option<&Pattern>) -> io::Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(pattern) = pattern {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if pattern.matches(name) {
                        println!("{}", path.display());
                    }
                }
            } else {
                println!("{}", path.display());
            }

            if path.is_dir() {
                Self::find_recursive(&path, pattern)?;
            }
        }
        Ok(())
    }

    pub fn head(&mut self, args: &[String]) -> io::Result<()> {
        let (file_name, lines) = match args.len() {
            0 => {
                eprintln!("Usage: head [-n lines] <file>");
                return Ok(());
            }
            1 => (args[0].as_str(), 10),
            _ if args[0] == "-n" => {
                (args[2].as_str(), args[1].parse().unwrap_or(10))
            }
            _ => (args[0].as_str(), 10),
        };

        let file = File::open(file_name)?;
        let reader = BufReader::new(file);
        for (i, line) in reader.lines().enumerate() {
            if i >= lines {
                break;
            }
            println!("{}", line?);
        }
        Ok(())
    }

    pub fn grep(&mut self, args: &[String]) -> io::Result<()> {
        if args.len() < 2 {
            eprintln!("Usage: grep <pattern> <file>");
            return Ok(());
        }

        let pattern = &args[0];
        let file_name = &args[1];
        let file = File::open(file_name)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.contains(pattern) {
                println!("{}", line);
            }
        }
        Ok(())
    }

    pub fn tail(&mut self, args: &[String]) -> io::Result<()> {
        let (file_name, lines) = match args.len() {
            0 => {
                eprintln!("Usage: tail [-n lines] <file>");
                return Ok(());
            }
            1 => (args[0].as_str(), 10),
            _ if args[0] == "-n" => {
                (args[2].as_str(), args[1].parse().unwrap_or(10))
            }
            _ => (args[0].as_str(), 10),
        };

        let file = File::open(file_name)?;
        let reader = BufReader::new(file);
        let all_lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;
        let start = all_lines.len().saturating_sub(lines);
        
        for line in &all_lines[start..] {
            println!("{}", line);
        }
        Ok(())
    }
}