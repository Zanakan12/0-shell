//! `cp [-r] src... dest` — copy files, and (with -r) directories.

use std::fs;
use std::path::{Path, PathBuf};

pub fn run(args: &[String]) -> Result<(), String> {
    let mut recursive = false;
    let mut operands = Vec::new();
    for a in args {
        if a.starts_with('-') && a.len() > 1 {
            for ch in a[1..].chars() {
                match ch {
                    'r' | 'R' => recursive = true,
                    other => return Err(format!("cp: invalid option -- '{}'", other)),
                }
            }
        } else {
            operands.push(a.as_str());
        }
    }

    if operands.len() < 2 {
        return Err("cp: missing file operand".to_string());
    }

    let (sources, dest) = operands.split_at(operands.len() - 1);
    let dest = Path::new(dest[0]);
    let dest_is_dir = dest.is_dir();

    if sources.len() > 1 && !dest_is_dir {
        return Err(format!("cp: target '{}' is not a directory", dest.display()));
    }

    let mut first_error: Option<String> = None;
    for src in sources {
        let src = Path::new(src);
        let target = if dest_is_dir {
            match src.file_name() {
                Some(name) => dest.join(name),
                None => {
                    first_error.get_or_insert(format!("cp: invalid source '{}'", src.display()));
                    continue;
                }
            }
        } else {
            dest.to_path_buf()
        };

        if let Err(msg) = copy_one(src, &target, recursive) {
            first_error.get_or_insert(msg);
        }
    }

    match first_error {
        Some(msg) => Err(msg),
        None => Ok(()),
    }
}

fn copy_one(src: &Path, dest: &PathBuf, recursive: bool) -> Result<(), String> {
    let meta = fs::symlink_metadata(src)
        .map_err(|e| format!("cp: cannot stat '{}': {}", src.display(), e))?;

    if meta.is_dir() {
        if !recursive {
            return Err(format!("cp: -r not specified; omitting directory '{}'", src.display()));
        }
        copy_dir_recursive(src, dest)
    } else {
        fs::copy(src, dest)
            .map(|_| ())
            .map_err(|e| format!("cp: cannot copy '{}': {}", src.display(), e))
    }
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    fs::create_dir_all(dest)
        .map_err(|e| format!("cp: cannot create directory '{}': {}", dest.display(), e))?;

    let entries = fs::read_dir(src)
        .map_err(|e| format!("cp: cannot read directory '{}': {}", src.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("cp: {}", e))?;
        let from = entry.path();
        let to = dest.join(entry.file_name());
        let ft = entry
            .file_type()
            .map_err(|e| format!("cp: {}", e))?;
        if ft.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            fs::copy(&from, &to)
                .map(|_| ())
                .map_err(|e| format!("cp: cannot copy '{}': {}", from.display(), e))?;
        }
    }
    Ok(())
}
