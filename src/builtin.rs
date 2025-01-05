use std::fs::{self, File};
use std::io::{self, Write, BufReader, BufRead};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use chrono;
use filetime;
use which;

use crate::shell::Shell;

impl Shell {
    pub fn cd(&mut self, args: &[String]) -> io::Result<()> {
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

    pub fn echo(&mut self, args: &[String]) -> io::Result<()> {
        println!("{}", args.join(" "));
        Ok(())
    }

    pub fn pwd(&mut self, _args: &[String]) -> io::Result<()> {
        println!("{}", self.current_dir.display());
        Ok(())
    }

    pub fn clear(&mut self, _args: &[String]) -> io::Result<()> {
        match self.shell_type {
            crate::shell_type::ShellType::Windows => {
                Command::new("cmd").args(["/C", "cls"]).status()?;
            }
            _ => {
                print!("\x1B[2J\x1B[1;1H");
                io::stdout().flush()?;
            }
        }
        Ok(())
    }

    pub fn touch(&mut self, args: &[String]) -> io::Result<()> {
        if args.is_empty() {
            eprintln!("touch: missing file operand");
            return Ok(());
        }
    
        for file_name in args {
            let path = Path::new(file_name);
            if path.exists() {
                let now = filetime::FileTime::now();
                if let Err(e) = filetime::set_file_mtime(path, now) {
                    eprintln!("touch: failed to update times for '{}': {}", file_name, e);
                }
            } else {
                if let Err(e) = File::create(path) {
                    eprintln!("touch: failed to create '{}': {}", file_name, e);
                }
            }
        }
        Ok(())
    }

    pub fn cat(&mut self, args: &[String]) -> io::Result<()> {
        if args.is_empty() {
            eprintln!("cat: missing file operand");
            return Ok(());
        }
    
        for file_name in args {
            let path = Path::new(file_name);
            if path.exists() {
                let file = File::open(path)?;
                let reader = BufReader::new(file);
                for line in reader.lines() {
                    println!("{}", line?);
                }
            } else {
                eprintln!("cat: {}: No such file or directory", file_name);
            }
        }
        Ok(())
    }
    
    pub fn mkdir(&mut self, args: &[String]) -> io::Result<()> {
        if args.is_empty() {
            eprintln!("mkdir: missing operand");
            return Ok(());
        }
    
        for dir_name in args {
            let path = Path::new(dir_name);
            if let Err(e) = fs::create_dir(path) {
                eprintln!("mkdir: cannot create directory '{}': {}", dir_name, e);
            }
        }
        Ok(())
    }
    
    pub fn env(&mut self, _args: &[String]) -> io::Result<()> {
        for (key, value) in &self.env_vars {
            println!("{}={}", key, value);
        }
        Ok(())
    }

    pub fn which(&mut self, args: &[String]) -> io::Result<()> {
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

    pub fn ls(&mut self, args: &[String]) -> io::Result<()> {
        let path = args.get(0).map(|s| Path::new(s)).unwrap_or(&self.current_dir);
        
        match self.shell_type {
            crate::shell_type::ShellType::Windows => {
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

    pub fn type_cmd(&mut self, args: &[String]) -> io::Result<()> {
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

    pub fn history(&mut self, args: &[String]) -> io::Result<()> {
        let limit = args.get(0)
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(self.history.len());

        for (i, cmd) in self.history.iter().rev().take(limit).rev().enumerate() {
            println!("{:5} {}", i + 1, cmd);
        }
        Ok(())
    }

    pub fn source(&mut self, args: &[String]) -> io::Result<()> {
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
}
             