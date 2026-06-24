//! Graceful Ctrl+C (SIGINT) handling (bonus feature).
//!
//! We install a no-op SIGINT handler via a direct FFI call to libc's `signal`
//! (the C library is already linked, so this needs no external crate). With a
//! handler installed instead of the default action, pressing Ctrl+C at the
//! prompt no longer terminates the shell — the read simply continues, leaving
//! the user back at a working prompt instead of crashing out.

const SIGINT: i32 = 2;

extern "C" {
    fn signal(signum: i32, handler: extern "C" fn(i32)) -> usize;
}

extern "C" fn ignore_sigint(_sig: i32) {
    // Intentionally empty: swallow the interrupt so the shell stays alive.
}

/// Install the SIGINT handler. Safe to call once at startup.
pub fn install() {
    // SAFETY: `ignore_sigint` is a valid `extern "C"` function pointer and we
    // pass a correct signal number; `signal` has no other preconditions.
    unsafe {
        signal(SIGINT, ignore_sigint);
    }
}
