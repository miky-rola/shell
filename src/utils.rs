use std::io::{self, Write};
use crate::shell_type::ShellType;

pub fn detect_os() -> ShellType {
    if cfg!(windows) {
        ShellType::Windows
    } else if cfg!(target_os = "macos") {
        ShellType::MacOS
    } else {
        ShellType::Linux
    }
}

pub fn select_shell_type() -> io::Result<ShellType> {
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