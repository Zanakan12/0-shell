//! `mv src... dest` — move/rename files and directories.

use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    let operands: Vec<&str> = args
        .iter()
        .filter(|a| !(a.starts_with('-') && a.len() > 1))
        .map(|s| s.as_str())
        .collect();

    if operands.len() < 2 {
        return Err("mv: missing file operand".to_string());
    }

    let (sources, dest) = operands.split_at(operands.len() - 1);
    let dest = Path::new(dest[0]);
    let dest_is_dir = dest.is_dir();

    if sources.len() > 1 && !dest_is_dir {
        return Err(format!("mv: target '{}' is not a directory", dest.display()));
    }

    let mut first_error: Option<String> = None;
    for src in sources {
        let src = Path::new(src);
        let target = if dest_is_dir {
            match src.file_name() {
                Some(name) => dest.join(name),
                None => {
                    first_error.get_or_insert(format!("mv: invalid source '{}'", src.display()));
                    continue;
                }
            }
        } else {
            dest.to_path_buf()
        };

        if let Err(e) = fs::rename(src, &target) {
            first_error.get_or_insert(format!(
                "mv: cannot move '{}' to '{}': {}",
                src.display(),
                target.display(),
                e
            ));
        }
    }

    match first_error {
        Some(msg) => Err(msg),
        None => Ok(()),
    }
}
