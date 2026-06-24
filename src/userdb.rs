//! Resolve numeric uids/gids to names by parsing /etc/passwd and /etc/group.
//!
//! This avoids spawning external programs (`id`, `getent`) while still showing
//! human-readable owners in `ls -l`. If a lookup fails for any reason we fall
//! back to the numeric id, exactly as coreutils does.

use std::fs;

/// Look up a user name for `uid`, falling back to the numeric id.
pub fn user_name(uid: u32) -> String {
    lookup("/etc/passwd", uid).unwrap_or_else(|| uid.to_string())
}

/// Look up a group name for `gid`, falling back to the numeric id.
pub fn group_name(gid: u32) -> String {
    lookup("/etc/group", gid).unwrap_or_else(|| gid.to_string())
}

/// Parse a colon-separated database (passwd/group) for `name:_:id:...` and
/// return the name matching `id`. Both files share this layout for the first
/// three fields.
fn lookup(path: &str, id: u32) -> Option<String> {
    let contents = fs::read_to_string(path).ok()?;
    for line in contents.lines() {
        let mut fields = line.split(':');
        let name = fields.next()?;
        let _passwd = fields.next()?;
        let entry_id: u32 = fields.next()?.parse().ok()?;
        if entry_id == id {
            return Some(name.to_string());
        }
    }
    None
}
