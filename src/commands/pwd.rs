//! `pwd` — print the current working directory.

use std::env;

pub fn run(_args: &[String]) -> Result<(), String> {
    let dir = env::current_dir().map_err(|e| format!("pwd: {}", e))?;
    println!("{}", dir.display());
    Ok(())
}
