//! `cat [file...]` — concatenate files to stdout. With no files, copy stdin.

use std::fs::File;
use std::io::{self, Read, Write};

pub fn run(args: &[String]) -> Result<(), String> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    if args.is_empty() {
        // Read from stdin until EOF.
        let mut buf = Vec::new();
        io::stdin()
            .read_to_end(&mut buf)
            .map_err(|e| format!("cat: {}", e))?;
        out.write_all(&buf).map_err(|e| format!("cat: {}", e))?;
        return Ok(());
    }

    let mut first_error: Option<String> = None;
    for path in args {
        match File::open(path) {
            Ok(mut f) => {
                let mut buf = Vec::new();
                if let Err(e) = f.read_to_end(&mut buf) {
                    first_error.get_or_insert(format!("cat: {}: {}", path, e));
                    continue;
                }
                if out.write_all(&buf).is_err() {
                    return Err("cat: write error".to_string());
                }
            }
            Err(e) => {
                first_error.get_or_insert(format!("cat: {}: {}", path, e));
            }
        }
    }

    match first_error {
        Some(msg) => Err(msg),
        None => Ok(()),
    }
}
