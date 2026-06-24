//! `ls [-l] [-a] [-F]` — list directory contents.
//!
//! Supported flags:
//!   -l  long format (permissions, links, owner, group, size, mtime, name)
//!   -a  include entries starting with '.'
//!   -F  append an indicator (/ * @ etc.) to entries

use std::fs::{self, Metadata};
use std::os::unix::fs::{FileTypeExt, MetadataExt, PermissionsExt};
use std::path::Path;

use crate::userdb;

struct Flags {
    long: bool,
    all: bool,
    classify: bool,
}

pub fn run(args: &[String]) -> Result<(), String> {
    let mut flags = Flags {
        long: false,
        all: false,
        classify: false,
    };
    let mut paths: Vec<&str> = Vec::new();

    for a in args {
        if a.starts_with('-') && a.len() > 1 {
            for ch in a[1..].chars() {
                match ch {
                    'l' => flags.long = true,
                    'a' => flags.all = true,
                    'F' => flags.classify = true,
                    other => return Err(format!("ls: invalid option -- '{}'", other)),
                }
            }
        } else {
            paths.push(a);
        }
    }

    if paths.is_empty() {
        paths.push(".");
    }

    let multiple = paths.len() > 1;
    let mut first_error: Option<String> = None;
    let mut first_output = true;

    for path in paths {
        if multiple {
            if !first_output {
                println!();
            }
            println!("{}:", path);
        }
        first_output = false;

        if let Err(msg) = list_path(Path::new(path), &flags) {
            first_error.get_or_insert(msg);
        }
    }

    match first_error {
        Some(msg) => Err(msg),
        None => Ok(()),
    }
}

fn list_path(path: &Path, flags: &Flags) -> Result<(), String> {
    let meta = fs::symlink_metadata(path)
        .map_err(|e| format!("ls: cannot access '{}': {}", path.display(), e))?;

    // A non-directory operand is listed as itself.
    if !meta.is_dir() {
        let name = path.to_string_lossy().to_string();
        let entry = Entry {
            display: name,
            file_name: path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            meta,
        };
        print_entries(&[entry], flags);
        return Ok(());
    }

    let mut entries: Vec<Entry> = Vec::new();

    if flags.all {
        // Include synthetic "." and ".." like coreutils does for -a.
        if let Ok(m) = fs::symlink_metadata(path) {
            entries.push(Entry {
                display: ".".to_string(),
                file_name: ".".to_string(),
                meta: m,
            });
        }
        if let Ok(m) = fs::symlink_metadata(path.join("..")) {
            entries.push(Entry {
                display: "..".to_string(),
                file_name: "..".to_string(),
                meta: m,
            });
        }
    }

    let read = fs::read_dir(path)
        .map_err(|e| format!("ls: cannot open directory '{}': {}", path.display(), e))?;

    for entry in read {
        let entry = entry.map_err(|e| format!("ls: {}", e))?;
        let file_name = entry.file_name().to_string_lossy().to_string();
        if !flags.all && file_name.starts_with('.') {
            continue;
        }
        let meta = entry
            .path()
            .symlink_metadata()
            .map_err(|e| format!("ls: {}", e))?;
        entries.push(Entry {
            display: file_name.clone(),
            file_name,
            meta,
        });
    }

    // Sort by name, ignoring a leading dot (coreutils-like ordering).
    entries.sort_by(|a, b| sort_key(&a.file_name).cmp(&sort_key(&b.file_name)));

    print_entries(&entries, flags);
    Ok(())
}

struct Entry {
    display: String,
    file_name: String,
    meta: Metadata,
}

fn sort_key(name: &str) -> String {
    name.trim_start_matches('.').to_lowercase()
}

fn print_entries(entries: &[Entry], flags: &Flags) {
    if flags.long {
        // total = sum of allocated blocks, in 1K units (st_blocks is 512B).
        let total: u64 = entries.iter().map(|e| e.meta.blocks()).sum::<u64>() / 2;
        println!("total {}", total);

        // Compute column widths for alignment.
        let mut links_w = 0;
        let mut owner_w = 0;
        let mut group_w = 0;
        let mut size_w = 0;
        let mut rows = Vec::new();

        for e in entries {
            let perms = perm_string(&e.meta);
            let nlink = e.meta.nlink().to_string();
            let owner = userdb::user_name(e.meta.uid());
            let group = userdb::group_name(e.meta.gid());
            let size = size_field(&e.meta);
            let time = crate::timefmt::format_mtime(e.meta.mtime());
            let name = decorated_name(e, flags);

            links_w = links_w.max(nlink.len());
            owner_w = owner_w.max(owner.len());
            group_w = group_w.max(group.len());
            size_w = size_w.max(size.len());

            rows.push((perms, nlink, owner, group, size, time, name));
        }

        for (perms, nlink, owner, group, size, time, name) in rows {
            println!(
                "{} {:>links$} {:<owner$} {:<group$} {:>size$} {} {}",
                perms,
                nlink,
                owner,
                group,
                size,
                time,
                name,
                links = links_w,
                owner = owner_w,
                group = group_w,
                size = size_w,
            );
        }
    } else {
        for e in entries {
            println!("{}", decorated_name(e, flags));
        }
    }
}

/// Build the name as shown, with optional -F classifier and color.
fn decorated_name(e: &Entry, flags: &Flags) -> String {
    let ft = e.meta.file_type();
    let suffix = if flags.classify {
        if ft.is_dir() {
            "/"
        } else if ft.is_symlink() {
            "@"
        } else if ft.is_socket() {
            "="
        } else if ft.is_fifo() {
            "|"
        } else if is_executable(&e.meta) {
            "*"
        } else {
            ""
        }
    } else {
        ""
    };

    let colored = crate::color::colorize(&e.display, &e.meta);
    format!("{}{}", colored, suffix)
}

/// Render the size column. For device files, show major,minor instead.
fn size_field(meta: &Metadata) -> String {
    let ft = meta.file_type();
    if ft.is_char_device() || ft.is_block_device() {
        let rdev = meta.rdev();
        let major = (rdev >> 8) & 0xfff;
        let minor = (rdev & 0xff) | ((rdev >> 12) & 0xfff00);
        format!("{}, {}", major, minor)
    } else {
        meta.size().to_string()
    }
}

fn is_executable(meta: &Metadata) -> bool {
    meta.is_file() && (meta.permissions().mode() & 0o111 != 0)
}

/// Build the 10-character permission string, e.g. "drwxr-xr-x".
fn perm_string(meta: &Metadata) -> String {
    let mode = meta.permissions().mode();
    let ft = meta.file_type();

    let type_char = if ft.is_dir() {
        'd'
    } else if ft.is_symlink() {
        'l'
    } else if ft.is_char_device() {
        'c'
    } else if ft.is_block_device() {
        'b'
    } else if ft.is_fifo() {
        'p'
    } else if ft.is_socket() {
        's'
    } else {
        '-'
    };

    let mut s = String::with_capacity(10);
    s.push(type_char);

    // Owner / group / other rwx triples, honoring setuid/setgid/sticky bits.
    let rwx = |shift: u32, special: u32, special_lc: char, special_uc: char| -> String {
        let bits = (mode >> shift) & 0o7;
        let mut t = String::with_capacity(3);
        t.push(if bits & 0o4 != 0 { 'r' } else { '-' });
        t.push(if bits & 0o2 != 0 { 'w' } else { '-' });
        let exec = bits & 0o1 != 0;
        let has_special = mode & special != 0;
        t.push(match (has_special, exec) {
            (true, true) => special_lc,
            (true, false) => special_uc,
            (false, true) => 'x',
            (false, false) => '-',
        });
        t
    };

    s.push_str(&rwx(6, 0o4000, 's', 'S')); // owner, setuid
    s.push_str(&rwx(3, 0o2000, 's', 'S')); // group, setgid
    s.push_str(&rwx(0, 0o1000, 't', 'T')); // other, sticky
    s
}
