mod shell;
mod shell_type;
mod utils;
mod builtin;
mod command_execution;

use shell::Shell;
// use shell_type::ShellType;
use utils::select_shell_type;
use std::io;

fn main() -> io::Result<()> {
    println!("Starting shell application...");
    let shell_type = select_shell_type()?;
    let mut shell = Shell::new(shell_type)?;
    println!("Shell created successfully, entering main loop...");
    shell.run()
}
