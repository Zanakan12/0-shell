//! `help` — document the built-in commands (bonus feature).

pub fn run(_args: &[String]) -> Result<(), String> {
    let text = "\
0-shell — built-in commands:

  echo [-n] [text...]     Print text. -n suppresses the trailing newline.
  cd [dir]                Change directory. No argument goes to $HOME.
  pwd                     Print the current working directory.
  ls [-l] [-a] [-F]       List directory contents.
                            -l long format, -a show hidden, -F classify.
  cat [file...]           Print files (or stdin) to standard output.
  cp [-r] src... dest     Copy files; -r for directories.
  rm [-r] [-f] file...    Remove files; -r for directories, -f to ignore errors.
  mv src... dest          Move or rename files and directories.
  mkdir [-p] dir...       Create directories; -p makes parents as needed.
  help                    Show this message.
  exit [code]             Exit the shell.

Features: command chaining with ';', $VAR expansion, quoting, Ctrl+D to exit.";
    println!("{}", text);
    Ok(())
}
