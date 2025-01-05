use std::fs::File;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::shell::Shell;
use crate::shell_type::ShellType;

impl Shell {
    pub fn parse_command(&self, input: &str) -> Vec<String> {
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

    pub fn check_redirection(&self, args: &[String]) -> (Vec<String>, Option<String>) {
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

    pub fn map_command(&self, command: &str, args: &[String]) -> (String, Vec<String>) {
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

    pub fn execute_piped_commands(&mut self, commands: &[Vec<String>]) -> io::Result<()> {
        let mut prev_stdout = None;

        for (i, command) in commands.iter().enumerate() {
            if command.is_empty() {
                continue;
            }

            let (command_name, args) = self.map_command(&command[0], &command[1..].to_vec());
            
            let is_last = i == commands.len() - 1;
            let mut cmd = Command::new(&command_name);
            cmd.args(&args);

            if let Some(prev_out) = prev_stdout {
                cmd.stdin(prev_out);
            }

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

    pub fn execute_external_command(&self, command: &str, args: &[String], output_file: Option<String>) -> io::Result<()> {
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