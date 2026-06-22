//! Tokenizer for the shell input line.
//!
//! Responsibilities:
//! - Split a line into separate commands on unquoted `;` (command chaining).
//! - Split each command into arguments on unquoted whitespace.
//! - Honor single quotes ('...'), double quotes ("..."), and backslash escapes.
//! - Expand environment variables ($VAR / ${VAR}) outside of single quotes.

use std::env;

/// A lexing error (currently only unterminated quotes).
#[derive(Debug)]
pub enum LexError {
    UnterminatedQuote(char),
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LexError::UnterminatedQuote(q) => {
                write!(f, "syntax error: unterminated quote {}", q)
            }
        }
    }
}

/// Parse a full input line into a list of commands, each a list of arguments.
/// Empty commands (e.g. from a trailing `;`) are dropped.
pub fn parse_line(line: &str) -> Result<Vec<Vec<String>>, LexError> {
    let mut commands = Vec::new();
    let mut current: Vec<String> = Vec::new();

    // Current argument being built and whether anything was written to it
    // (so that an empty quoted argument like "" is preserved).
    let mut arg = String::new();
    let mut arg_has_content = false;

    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            // Unquoted whitespace ends the current argument.
            ' ' | '\t' => {
                if arg_has_content {
                    current.push(std::mem::take(&mut arg));
                    arg_has_content = false;
                }
            }
            // Command separator.
            ';' => {
                if arg_has_content {
                    current.push(std::mem::take(&mut arg));
                    arg_has_content = false;
                }
                if !current.is_empty() {
                    commands.push(std::mem::take(&mut current));
                }
            }
            // Single quotes: literal, no expansion or escapes inside.
            '\'' => {
                arg_has_content = true;
                loop {
                    match chars.next() {
                        Some('\'') => break,
                        Some(ch) => arg.push(ch),
                        None => return Err(LexError::UnterminatedQuote('\'')),
                    }
                }
            }
            // Double quotes: escapes for \" \\ \$ and env expansion are honored.
            '"' => {
                arg_has_content = true;
                loop {
                    match chars.next() {
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some(n @ ('"' | '\\' | '$')) => arg.push(n),
                            Some(other) => {
                                arg.push('\\');
                                arg.push(other);
                            }
                            None => return Err(LexError::UnterminatedQuote('"')),
                        },
                        Some('$') => expand_var(&mut chars, &mut arg),
                        Some(ch) => arg.push(ch),
                        None => return Err(LexError::UnterminatedQuote('"')),
                    }
                }
            }
            // Backslash escape outside quotes: next char is literal.
            '\\' => {
                arg_has_content = true;
                if let Some(n) = chars.next() {
                    arg.push(n);
                }
            }
            // Environment variable expansion outside quotes.
            '$' => {
                arg_has_content = true;
                expand_var(&mut chars, &mut arg);
            }
            // Ordinary character.
            _ => {
                arg_has_content = true;
                arg.push(c);
            }
        }
    }

    if arg_has_content {
        current.push(arg);
    }
    if !current.is_empty() {
        commands.push(current);
    }

    Ok(commands)
}

/// Expand a variable reference starting just after a `$`. Supports `$NAME` and
/// `${NAME}`. An unknown variable expands to the empty string. A lone `$`
/// (not followed by a name) is kept literally.
fn expand_var<I>(chars: &mut std::iter::Peekable<I>, out: &mut String)
where
    I: Iterator<Item = char>,
{
    // Braced form: ${NAME}
    if chars.peek() == Some(&'{') {
        chars.next();
        let mut name = String::new();
        for ch in chars.by_ref() {
            if ch == '}' {
                break;
            }
            name.push(ch);
        }
        push_var(&name, out);
        return;
    }

    // Bare form: $NAME (letters, digits, underscore).
    let mut name = String::new();
    while let Some(&ch) = chars.peek() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            name.push(ch);
            chars.next();
        } else {
            break;
        }
    }

    if name.is_empty() {
        // A literal `$` with nothing after it.
        out.push('$');
    } else {
        push_var(&name, out);
    }
}

fn push_var(name: &str, out: &mut String) {
    if let Ok(val) = env::var(name) {
        out.push_str(&val);
    }
}
