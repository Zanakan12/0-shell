//! `cd [dir]` — change the working directory. With no argument, go to $HOME.

use std::env;
use std::path::PathBuf;

pub fn run(args: &[String]) -> Result<(), String> {
    if args.len() > 1 {
        return Err("cd: too many arguments".to_string());
    }

    let target: PathBuf = match args.first() {
        None => home_dir().ok_or("cd: HOME not set")?,
        Some(p) if p == "~" => home_dir().ok_or("cd: HOME not set")?,
        Some(p) if p.starts_with("~/") => {
            let home = home_dir().ok_or("cd: HOME not set")?;
            home.join(&p[2..])
        }
        Some(p) => PathBuf::from(p),
    };

    env::set_current_dir(&target)
        .map_err(|e| format!("cd: {}: {}", target.display(), e))
}

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}
