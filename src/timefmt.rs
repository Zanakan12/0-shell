//! Local-time formatting for `ls -l`, matching coreutils' default style.
//!
//! Converting a Unix timestamp to *local* civil time requires the system
//! timezone, which std does not expose. We call libc's `localtime_r` directly
//! via FFI (the C library is already linked, so no crate is needed).

use std::os::raw::{c_int, c_long};

/// Mirror of glibc's `struct tm` on Linux. Only the leading integer fields are
/// read; the trailing `tm_gmtoff`/`tm_zone` are present for correct layout.
#[repr(C)]
struct Tm {
    tm_sec: c_int,
    tm_min: c_int,
    tm_hour: c_int,
    tm_mday: c_int,
    tm_mon: c_int,
    tm_year: c_int,
    tm_wday: c_int,
    tm_yday: c_int,
    tm_isdst: c_int,
    tm_gmtoff: c_long,
    tm_zone: *const u8,
}

extern "C" {
    fn localtime_r(timep: *const i64, result: *mut Tm) -> *mut Tm;
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Format a modification time as `Mon DD HH:MM`, or `Mon DD  YYYY` for files
/// more than ~6 months from now — the same heuristic GNU `ls` uses.
pub fn format_mtime(secs: i64) -> String {
    let mut tm: Tm = unsafe { std::mem::zeroed() };
    // SAFETY: both pointers are valid for the duration of the call.
    let ok = unsafe { !localtime_r(&secs, &mut tm).is_null() };
    if !ok {
        return secs.to_string();
    }

    let month = tm.tm_mon.clamp(0, 11) as usize;
    let mon = MONTHS[month];
    let day = tm.tm_mday;
    let year = tm.tm_year as i64 + 1900;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(secs);
    let six_months = 60 * 60 * 24 * 365 / 2;

    if (now - secs).abs() > six_months {
        format!("{} {:>2}  {}", mon, day, year)
    } else {
        format!("{} {:>2} {:02}:{:02}", mon, day, tm.tm_hour, tm.tm_min)
    }
}
