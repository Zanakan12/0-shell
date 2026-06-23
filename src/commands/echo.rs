//! `echo [-n] [args...]` — print arguments separated by spaces.

pub fn run(args: &[String]) -> Result<(), String> {
    // Support the common `-n` flag (suppress trailing newline). Flags may be
    // combined and must appear before the first operand, matching coreutils.
    let mut newline = true;
    let mut idx = 0;
    while idx < args.len() {
        let a = &args[idx];
        if a.len() >= 2 && a.starts_with('-') && a[1..].chars().all(|c| c == 'n') {
            newline = false;
            idx += 1;
        } else {
            break;
        }
    }

    let text = args[idx..].join(" ");
    if newline {
        println!("{}", text);
    } else {
        print!("{}", text);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
    Ok(())
}
