#[warn(unused_imports)]
use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write, BufReader, BufRead};
use std::path::{Path, PathBuf, Component};
use std::process::{Command, Stdio};
use dirs;
use hostname;


#[derive(Debug, Clone, PartialEq)]
enum ShellType {
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug)]
struct Shell {
    shell_type: ShellType,
    current_dir: PathBuf,
    env_vars: HashMap<String, String>,
    builtins: HashMap<String, fn(&mut Shell, &[String]) -> io::Result<()>>,
    home_dir: PathBuf,
    history: Vec<String>,
    history_file: PathBuf,
}

impl Shell {
    fn new(shell_type: ShellType) -> io::Result<Shell> {
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

    fn get_prompt(&self) -> String {
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

    fn run(&mut self) -> io::Result<()> {
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

            self.add_to_history(input);

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
                ('|', false, false) => {
                    if !current_token.is_empty() {
                        tokens.push(current_token);
                        current_token = String::new();
                    }
                    tokens.push("|".to_string());
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

        // Split commands by pipe
        let mut commands: Vec<Vec<String>> = Vec::new();
        let mut current_command = Vec::new();

        for token in tokens {
            if token == "|" {
                if !current_command.is_empty() {
                    commands.push(current_command);
                    current_command = Vec::new();
                }
            } else {
                current_command.push(token);
            }
        }
        if !current_command.is_empty() {
            commands.push(current_command);
        }

        // Execute piped commands
        if commands.len() > 1 {
            self.execute_piped_commands(&commands)
        } else {
            let tokens = &commands[0];
            if tokens.is_empty() {
                return Ok(());
            }

            let command = &tokens[0];
            let args = &tokens[1..];

            let (args, output_file) = self.check_redirection(args);

            // First, check if it's a builtin command using the original command name
            if let Some(builtin) = self.builtins.get(command) {
                return builtin(self, &args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
            }

            // If not a builtin, map the command for the current OS
            let (mapped_command, mapped_args) = self.map_command(command, &args);
            
            // After mapping, check again if it's now a builtin
            if let Some(builtin) = self.builtins.get(&mapped_command) {
                return builtin(self, &mapped_args.iter().map(|s| s.to_string()).collect::<Vec<_>>());
            }

            // Finally, execute as external command
            self.execute_external_command(&mapped_command, &mapped_args, output_file)
        }
    }

    fn execute_piped_commands(&mut self, commands: &[Vec<String>]) -> io::Result<()> {
        let mut prev_stdout = None;

        for (i, command) in commands.iter().enumerate() {
            if command.is_empty() {
                continue;
            }

            let (command_name, args) = self.map_command(&command[0], &command[1..].to_vec());
            
            let is_last = i == commands.len() - 1;
            let mut cmd = Command::new(&command_name);
            cmd.args(&args);

            // Set up stdin from previous command
            if let Some(prev_out) = prev_stdout {
                cmd.stdin(prev_out);
            }

            // Set up stdout pipe for next command
            if !is_last {
                cmd.stdout(Stdio::piped());
            }

            let output = if is_last {
                cmd.status()?;
                None
            } else {
                let child = cmd.spawn()?;
                Some(child.stdout.unwrap())
            };

            prev_stdout = output;
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

    fn normalize_path(&self, path: &Path) -> PathBuf {
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

    fn map_command(&self, command: &str, args: &[String]) -> (String, Vec<String>) {
        match self.shell_type {
            ShellType::Windows => match command {
                "ls" => ("cmd".to_string(), vec!["/C".to_string(), "dir".to_string(), "/W".to_string()]),
                "clear" => ("cmd".to_string(), vec!["/C".to_string(), "cls".to_string()]),
                "rm" => ("del".to_string(), args.to_vec()),
                "cp" => ("copy".to_string(), args.to_vec()),
                "mv" => ("move".to_string(), args.to_vec()),
                "cat" => ("type".to_string(), args.to_vec()),
                "grep" => ("findstr".to_string(), args.to_vec()),
                "touch" => {
                    let mut new_args = vec!["NUL".to_string(), ">".to_string()];
                    new_args.extend(args.iter().cloned());
                    ("echo".to_string(), new_args)
                },
                "chmod" => ("icacls".to_string(), args.to_vec()),
                "ps" => ("tasklist".to_string(), vec![]),
                "kill" => ("taskkill".to_string(), vec!["/PID".to_string()].into_iter().chain(args.iter().cloned()).collect()),
                _ => (command.to_string(), args.to_vec()),
            },
            ShellType::Linux | ShellType::MacOS => match command {
                "dir" => ("ls".to_string(), vec![]),
                "cls" => ("clear".to_string(), vec![]),
                "copy" => ("cp".to_string(), args.to_vec()),
                "move" => ("mv".to_string(), args.to_vec()),
                "del" => ("rm".to_string(), args.to_vec()),
                "type" => ("cat".to_string(), args.to_vec()),
                "findstr" => ("grep".to_string(), args.to_vec()),
                "tasklist" => ("ps".to_string(), vec!["-e".to_string()]),
                "taskkill" => {
                    let mut new_args = vec!["-9".to_string()];
                    new_args.extend(args.iter().cloned());
                    ("kill".to_string(), new_args)
                },
                _ => (command.to_string(), args.to_vec()),
            },
        }
    }

    // Builtin Commands
    fn cd(&mut self, args: &[String]) -> io::Result<()> {
        let new_dir = match args.get(0) {
            Some(path) if path == "~" || path == "$HOME" => self.home_dir.clone(),
            Some(path) => {
                let path_buf = PathBuf::from(path);
                if path_buf.is_absolute() {
                    path_buf
                } else {
                    self.current_dir.join(path_buf)
                }
            }
            None => self.home_dir.clone(),
        };

        let normalized_path = self.normalize_path(&new_dir);
        env::set_current_dir(&normalized_path)?;
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

    fn clear(&mut self, _args: &[String]) -> io::Result<()> {
        match self.shell_type {
            ShellType::Windows => {
                Command::new("cmd").args(["/C", "cls"]).status()?;
            }
            _ => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush()?;
            }
        }
        Ok(())
    }

    fn env(&mut self, _args: &[String]) -> io::Result<()> {
        for (key, value) in &self.env_vars {
            println!("{}={}", key, value);
        }
        Ok(())
    }

    fn which(&mut self, args: &[String]) -> io::Result<()> {
        if args.is_empty() {
            println!("which: missing command name");
            return Ok(());
        }

        let cmd = &args[0];
        if self.builtins.contains_key(cmd) {
            println!("{} is a shell builtin", cmd);
        } else {
            match which::which(cmd) {
                Ok(path) => println!("{}", path.display()),
                Err(_) => println!("{}: command not found", cmd),
            }
        }
        Ok(())
    }

    fn ls(&mut self, args: &[String]) -> io::Result<()> {
        let path = args.get(0).map(|s| Path::new(s)).unwrap_or(&self.current_dir);
        
        match self.shell_type {
            ShellType::Windows => {
                Command::new("cmd")
                    .args(["/C", "dir", "/W"])
                    .current_dir(path)
                    .status()?;
            }
            _ => {
                println!("{} {} {:>8} {:>19} {}", 
                    "Type",
                    "Perms",
                    "Size",
                    "Modified",
                    "Name"
                );
                println!("{} {} {:>8} {:>19} {}", 
                    "----",
                    "-----",
                    "----",
                    "-------------------",
                    "----"
                );
    
                for entry in fs::read_dir(path)? {
                    let entry = entry?;
                    let metadata = entry.metadata()?;
                    let file_type = if metadata.is_dir() { "d" } else { "-" };
                    
                    #[cfg(unix)]
                    let permissions = format!("{:o}", metadata.permissions().mode() & 0o777);
                    #[cfg(not(unix))]
                    let permissions = "N/A".to_string();
                    
                    let size = metadata.len();
                    
                    // Format the last modified time
                    let modified = metadata.modified()?;
                    let datetime: chrono::DateTime<chrono::Local> = modified.into();
                    let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
                    
                    let name = entry.file_name();
                    println!("{:4} {:5} {:8} {} {}", 
                        file_type, 
                        permissions, 
                        size, 
                        formatted_time,
                        name.to_string_lossy()
                    );
                }
            }
        }
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
            match which::which(cmd) {
                Ok(path) => println!("{} is {}", cmd, path.display()),
                Err(_) => println!("{}: not found", cmd),
            }
        }
        Ok(())
    }

    fn history(&mut self, args: &[String]) -> io::Result<()> {
        let limit = args.get(0)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(self.history.len());

        for (i, cmd) in self.history.iter().rev().take(limit).rev().enumerate() {
            println!("{:5} {}", i + 1, cmd);
        }
        Ok(())
    }

    fn source(&mut self, args: &[String]) -> io::Result<()> {
        if args.is_empty() {
            println!("source: missing file operand");
            return Ok(());
        }

        let path = PathBuf::from(&args[0]);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if !line.trim().is_empty() && !line.trim().starts_with('#') {
                self.execute_command(&line)?;
            }
        }
        Ok(())
    }

    fn execute_external_command(&self, command: &str, args: &[String], output_file: Option<String>) -> io::Result<()> {
        let mut cmd = Command::new(command);
        cmd.args(args)
            .current_dir(&self.current_dir)
            .envs(&self.env_vars);

        if let Some(file_path) = output_file {
            let output = cmd.output()?;
            let mut file = File::create(file_path)?;
            file.write_all(&output.stdout)?;
        } else {
            cmd.status()?;
        }

        Ok(())
    }
}

fn detect_os() -> ShellType {
    if cfg!(windows) {
        ShellType::Windows
    } else if cfg!(target_os = "macos") {
        ShellType::MacOS
    } else {
        ShellType::Linux
    }
}

fn select_shell_type() -> io::Result<ShellType> {
    let default_os = detect_os();
    println!("Select shell type (default: {:?}):", default_os);
    println!("1. Linux");
    println!("2. MacOS");
    println!("3. Windows");
    println!("Press Enter for default");
    
    loop {
        print!("Enter your choice (1-3): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "" => return Ok(default_os),
            "1" => return Ok(ShellType::Linux),
            "2" => return Ok(ShellType::MacOS),
            "3" => return Ok(ShellType::Windows),
            _ => println!("Invalid choice, please try again."),
        }
    }
}

fn main() -> io::Result<()> {
    println!("Starting shell application...");
    let shell_type = select_shell_type()?;
    let mut shell = Shell::new(shell_type)?;
    println!("Shell created successfully, entering main loop...");
    shell.run()
}