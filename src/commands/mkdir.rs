//! `mkdir [-p] dir...` — create directories.

use std::fs;

pub fn run(args: &[String]) -> Result<(), String> {
    let mut parents = false;
    let mut dirs = Vec::new();
    for a in args {
        if a == "-p" {
            parents = true;
        } else if a.starts_with('-') && a.len() > 1 {
            return Err(format!("mkdir: invalid option -- '{}'", &a[1..]));
        } else {
            dirs.push(a);
        }
    }

    if dirs.is_empty() {
        return Err("mkdir: missing operand".to_string());
    }

    let mut first_error: Option<String> = None;
    for dir in dirs {
        let res = if parents {
            fs::create_dir_all(dir)
        } else {
            fs::create_dir(dir)
        };
        if let Err(e) = res {
            first_error.get_or_insert(format!("mkdir: cannot create directory '{}': {}", dir, e));
        }
    }

    match first_error {
        Some(msg) => Err(msg),
        None => Ok(()),
    }
}
