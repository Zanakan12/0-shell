//! Minimal, dependency-free output coloring for `ls`.
//!
//! Colors are only emitted when stdout is an interactive terminal, so that
//! redirected/piped output stays clean and comparable to plain text. Terminal
//! detection is done without libc by inspecting /proc/self/fd/1 on Linux.

use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::sync::OnceLock;

const RESET: &str = "\x1b[0m";
const BLUE: &str = "\x1b[1;34m"; // directories
const CYAN: &str = "\x1b[1;36m"; // symlinks
const GREEN: &str = "\x1b[1;32m"; // executables
const YELLOW: &str = "\x1b[1;33m"; // devices / fifo / socket

static IS_TTY: OnceLock<bool> = OnceLock::new();

fn stdout_is_tty() -> bool {
    *IS_TTY.get_or_init(|| match fs::read_link("/proc/self/fd/1") {
        Ok(target) => {
            let p = target.to_string_lossy();
            p.contains("/dev/pts/") || p.starts_with("/dev/tty")
        }
        Err(_) => false,
    })
}

/// Wrap `name` in an ANSI color appropriate for its file type, or return it
/// unchanged when not writing to a terminal.
pub fn colorize(name: &str, meta: &std::fs::Metadata) -> String {
    if !stdout_is_tty() {
        return name.to_string();
    }

    let ft = meta.file_type();
    let code = if ft.is_dir() {
        BLUE
    } else if ft.is_symlink() {
        CYAN
    } else if ft.is_char_device() || ft.is_block_device() || ft.is_fifo() || ft.is_socket() {
        YELLOW
    } else if meta.is_file() && meta.permissions().mode() & 0o111 != 0 {
        GREEN
    } else {
        return name.to_string();
    };

    format!("{}{}{}", code, name, RESET)
}
