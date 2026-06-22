//! Built-in command implementations and the dispatcher.

mod cat;
mod cd;
mod cp;
mod echo;
mod help;
mod ls;
mod mkdir;
mod mv;
mod pwd;
mod rm;

/// What the shell loop should do after a command runs.
pub enum Flow {
    /// Continue the read-eval loop.
    Continue,
    /// Terminate the shell with the given status code.
    Exit(i32),
}

/// Execute a single parsed command (name + arguments).
///
/// `args` is guaranteed non-empty by the caller. Errors are printed to stderr
/// here so each command implementation can simply return `Result`.
pub fn dispatch(args: &[String]) -> Flow {
    let name = args[0].as_str();
    let rest = &args[1..];

    let result: Result<(), String> = match name {
        "exit" => return exit(rest),
        "echo" => echo::run(rest),
        "cd" => cd::run(rest),
        "pwd" => pwd::run(rest),
        "ls" => ls::run(rest),
        "cat" => cat::run(rest),
        "cp" => cp::run(rest),
        "rm" => rm::run(rest),
        "mv" => mv::run(rest),
        "mkdir" => mkdir::run(rest),
        "help" => help::run(rest),
        other => Err(format!("Command '{}' not found", other)),
    };

    if let Err(msg) = result {
        eprintln!("{}", msg);
    }
    Flow::Continue
}

/// `exit [code]` — leave the shell, optionally with a status code.
fn exit(args: &[String]) -> Flow {
    let code = args
        .first()
        .and_then(|s| s.parse::<i32>().ok())
        .unwrap_or(0);
    Flow::Exit(code)
}
