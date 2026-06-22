//! 0-shell: a minimalist Unix-like shell implemented in pure Rust (std only).
//!
//! Reads a line, parses it into commands, and dispatches each to a built-in
//! implementation. No external binaries are spawned. Exits cleanly on `exit`
//! or Ctrl+D (EOF).

mod color;
mod commands;
mod lexer;
mod signal;
mod timefmt;
mod userdb;

use std::env;
use std::io::{self, Write};
use std::process;

use commands::Flow;

fn main() {
    signal::install();

    let stdin = io::stdin();
    let mut line = String::new();

    loop {
        print_prompt();

        line.clear();
        match stdin.read_line(&mut line) {
            // EOF (Ctrl+D): print a newline and exit cleanly.
            Ok(0) => {
                println!();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("0-shell: read error: {}", e);
                continue;
            }
        }

        let trimmed = line.trim_end_matches('\n');
        if trimmed.trim().is_empty() {
            continue;
        }

        let commands = match lexer::parse_line(trimmed) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        for cmd in &commands {
            if cmd.is_empty() {
                continue;
            }
            if let Flow::Exit(code) = commands::dispatch(cmd) {
                process::exit(code);
            }
        }
    }
}

/// Print the prompt, showing the current directory with $HOME abbreviated to
/// `~`, e.g. `~/projects/0-shell $ `.
fn print_prompt() {
    let cwd = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "?".to_string());

    let display = match env::var("HOME") {
        Ok(home) if !home.is_empty() && cwd.starts_with(&home) => {
            format!("~{}", &cwd[home.len()..])
        }
        _ => cwd,
    };

    print!("{} $ ", display);
    let _ = io::stdout().flush();
}
