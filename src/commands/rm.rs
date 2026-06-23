//! `rm [-r] [-f] file...` — remove files and (with -r) directories.

use std::fs;
use std::path::Path;

pub fn run(args: &[String]) -> Result<(), String> {
    let mut recursive = false;
    let mut force = false;
    let mut targets = Vec::new();

    for a in args {
        if a.starts_with('-') && a.len() > 1 {
            for ch in a[1..].chars() {
                match ch {
                    'r' | 'R' => recursive = true,
                    'f' => force = true,
                    other => return Err(format!("rm: invalid option -- '{}'", other)),
                }
            }
        } else {
            targets.push(a);
        }
    }

    if targets.is_empty() {
        if force {
            return Ok(());
        }
        return Err("rm: missing operand".to_string());
    }

    let mut first_error: Option<String> = None;
    for t in targets {
        if let Err(msg) = remove_one(Path::new(t), recursive, force) {
            first_error.get_or_insert(msg);
        }
    }

    match first_error {
        Some(msg) => Err(msg),
        None => Ok(()),
    }
}

fn remove_one(path: &Path, recursive: bool, force: bool) -> Result<(), String> {
    // symlink_metadata so we don't follow symlinks when deciding file vs dir.
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(e) => {
            if force {
                return Ok(());
            }
            return Err(format!("rm: cannot remove '{}': {}", path.display(), e));
        }
    };

    if meta.is_dir() {
        if !recursive {
            return Err(format!(
                "rm: cannot remove '{}': Is a directory",
                path.display()
            ));
        }
        fs::remove_dir_all(path)
            .map_err(|e| format!("rm: cannot remove '{}': {}", path.display(), e))
    } else {
        fs::remove_file(path)
            .map_err(|e| format!("rm: cannot remove '{}': {}", path.display(), e))
    }
}
