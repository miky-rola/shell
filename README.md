# Rust Shell Implementation

A lightweight, cross-platform shell implementation written in Rust. This shell provides basic command-line functionality with built-in commands and support for external commands and command history.

## Features

- Cross-platform support (Linux, macOS, Windows)
- Built-in commands implementation
- Command history with persistence
- Environment variable support
- Platform-specific command prompts
- Directory navigation with path normalization
- File operations and text processing commands

### Built-in Commands

- `cd` - Change directory
- `pwd` - Print working directory
- `ls` - List directory contents
- `echo` - Display text
- `clear` - Clear screen
- `env` - Display environment variables
- `cat` - Concatenate and display file contents
- `grep` - Search text using patterns
- `find` - Search for files in directory hierarchy
- `head` - Output the first part of files
- `tail` - Output the last part of files

## Prerequisites

Make sure you have Rust and Cargo installed on your system. If not, you can install them from [rustup.rs](https://rustup.rs/).

## Installation

1. Clone the repository:
```bash
git clone https://github.com/miky-rola/shell/
```

2. Build the project:
```bash
cargo build --release
```

3. Run the shell:
```bash
cargo run --release
```

## Project Structure

```
src/
├── main.rs         # Entry point and shell initialization
├── shell.rs        # Core shell implementation
├── command_execution.rs       # Core shell implementation
└── shell_type.rs   # Shell type enumeration and related functionality
└── builtin.rs      # conatins the built in commands
└── utils.rs        # utils for shell
```

## Dependencies

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
anyhow = "1.0.68"                                
bytes = "1.3.0"                                 
thiserror = "1.0.38"                             
dirs = "5.0"
hostname = "0.3"
which = "4.4"
chrono = "0.4"
filetime = "0.2"
glob = "0.3"
```

## Usage

### Basic Usage

1. Start the shell:
```bash
cargo run --release
```

2. Use built-in commands:
```bash
$ pwd                    # Print working directory
$ cd ~/Documents        # Change directory
$ ls                    # List files
$ echo Hello, World!    # Print text
```

### Command History

- Use Up/Down arrow keys to navigate through command history
- Command history is automatically saved and persists between sessions
- History file is stored in your home directory as `.shell_history` (Unix) or `.shell_history.txt` (Windows)

## Example Session

```bash
user@hostname:~$ ls
Documents    Downloads    Pictures
user@hostname:~$ cd Documents
user@hostname:~/Documents$ pwd
/home/user/Documents
user@hostname:~/Documents$ echo "Hello from Rust Shell!"
Hello from Rust Shell!
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Known Limitations

- Limited shell scripting capabilities
- Basic command line editing features
- No pipeline operations support yet
- Limited wildcard expansion

## Future Improvements

- [ ] Add support for command pipelines (`|`)
- [ ] Implement input/output redirection
- [ ] Add shell scripting capabilities
- [ ] Implement job control
- [ ] Add more built-in commands
- [ ] Improve tab completion with context awareness
- [ ] Add alias support
- [ ] Implement environment variable expansion


## Acknowledgments

- The Rust community for excellent documentation and crates
- Various Unix shell implementations that inspired this project