use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::io::{self, Write, BufReader, BufRead};
use std::path::{Path, PathBuf, Component};
use std::process::{Command, Stdio};
use dirs;
use hostname;
use chrono;
use filetime;
use which;

use crate::shell_type::ShellType;

pub struct Shell {
    shell_type: ShellType,
    current_dir: PathBuf,
    env_vars: HashMap<String, String>,
    builtins: HashMap<String, fn(&mut Shell, &[String]) -> io::Result<()>>,
    home_dir: PathBuf,
    history: Vec<String>,
    history_file: PathBuf,
}