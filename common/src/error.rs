use crate::{fileposition::FilePosition, ttype::TType};
use colored::Colorize;
use std::{
    borrow::Cow,
    fmt,
    io::{self, BufRead},
    path::Path,
};

/// A boxed result alias used throughout the compiler pipeline.
/// `NovaError` is a large enum (>128 bytes due to `TType` fields),
/// so we box it in `Result` to keep return values pointer-sized on the
/// error path and satisfy clippy::result_large_err.
pub type NovaResult<T> = Result<T, Box<NovaError>>;

// ───────────────────────────────────────────────────────────────────
// Source-reading helper
// ───────────────────────────────────────────────────────────────────

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<std::fs::File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

/// Read a window of source lines around `center` (1-indexed).
/// Returns `Vec<(line_number, line_text)>`.
fn read_source_window(
    filepath: Option<&Path>,
    center: usize,
    radius: usize,
) -> Vec<(usize, String)> {
    let path = filepath.unwrap_or(Path::new(""));
    let Ok(lines) = read_lines(path) else {
        return Vec::new();
    };
    let start = center.saturating_sub(radius);
    let end = center + radius;
    let mut result = Vec::new();
    for (idx, line) in lines.enumerate() {
        let line_num = idx + 1;
        if line_num < start {
            continue;
        }
        if line_num > end {
            break;
        }
        if let Ok(text) = line {
            result.push((line_num, text));
        }
    }
    result
}

// ───────────────────────────────────────────────────────────────────
// Rendering helpers
// ───────────────────────────────────────────────────────────────────

/// The gutter separator string.
const GUTTER_SEP: &str = " │ ";
const GUTTER_SEP_BOLD: &str = " │ ";

/// Render a rich source snippet with a caret underline.
fn render_snippet(position: &FilePosition, label: &str) {
    let source = read_source_window(
        position.filepath.as_deref(),
        position.line,
        1, // show 1 line above and below
    );
    if source.is_empty() {
        return;
    }

    // Width of the widest line number
    let width = source
        .iter()
        .map(|(n, _)| n.to_string().len())
        .max()
        .unwrap_or(1);

    // blank gutter line
    println!(
        "  {}{}",
        format!("{:>width$}", "", width = width).dimmed(),
        GUTTER_SEP.dimmed()
    );

    for &(line_num, ref text) in &source {
        let is_error_line = line_num == position.line;
        let num_str = format!("{:>width$}", line_num, width = width);
        if is_error_line {
            println!(
                "  {}{}{}",
                num_str.bright_red().bold(),
                GUTTER_SEP_BOLD.bright_red(),
                text
            );
            // Underline
            let col = position.col.saturating_sub(1);
            let padding: String = text
                .chars()
                .take(col)
                .map(|c| if c == '\t' { '\t' } else { ' ' })
                .collect();
            // Determine underline length: at least 1 char, up to next whitespace or end
            let rest = &text[text
                .char_indices()
                .nth(col)
                .map(|(i, _)| i)
                .unwrap_or(text.len())..];
            let underline_len = rest
                .chars()
                .take_while(|c| !c.is_whitespace())
                .count()
                .max(1);
            let underline = "─".repeat(underline_len);
            println!(
                "  {}{}{}{}",
                format!("{:>width$}", "", width = width).dimmed(),
                GUTTER_SEP.dimmed(),
                padding,
                format!("╰{}─ {}", underline, label).bright_red().bold(),
            );
        } else {
            println!(
                "  {}{}{}",
                num_str.dimmed(),
                GUTTER_SEP.dimmed(),
                text.dimmed()
            );
        }
    }

    // closing gutter line
    println!(
        "  {}{}",
        format!("{:>width$}", "", width = width).dimmed(),
        GUTTER_SEP.dimmed()
    );
}

/// Render the "─── location ───" header bar.
fn render_location_header(kind: &str, position: &FilePosition) {
    let file = position
        .filepath
        .as_deref()
        .unwrap_or(Path::new("repl"))
        .display();
    let location = format!("{}:{}:{}", file, position.line, position.col);
    println!();
    println!(
        "{}{} {}",
        format!("  ── {} ", kind).bright_red().bold(),
        "──────────────────────────────".bright_red(),
        location.dimmed()
    );
}

/// Render a help/note line.
fn render_note(note: &str) {
    if !note.is_empty() {
        println!("  {} {}", "help:".bright_cyan().bold(), note.bright_cyan());
    }
}

/// Render extra annotations (secondary spans).
fn render_extra(extras: &[(String, FilePosition)]) {
    for (msg, pos) in extras {
        render_snippet(pos, msg);
    }
}

/// Trailing empty line for breathing room.
fn render_footer() {
    println!();
}

// ───────────────────────────────────────────────────────────────────
// The NovaError type
// ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum NovaError {
    /// Error opening / reading a source file.
    File { msg: Cow<'static, str> },

    /// Lexer encountered an invalid token or unterminated literal.
    Lexing {
        msg: Cow<'static, str>,
        note: Cow<'static, str>,
        position: FilePosition,
    },

    /// Parser encountered unexpected syntax.
    Parsing {
        msg: Cow<'static, str>,
        note: Cow<'static, str>,
        position: FilePosition,
        extra: Option<Vec<(String, FilePosition)>>,
    },

    /// Compiler back-end error (code generation).
    Compiler {
        msg: Cow<'static, str>,
        note: Cow<'static, str>,
    },

    /// VM runtime error (no source location available).
    Runtime { msg: Cow<'static, str> },

    /// VM runtime error with source location.
    RuntimeWithPos {
        msg: Cow<'static, str>,
        position: FilePosition,
    },

    /// Type error with expected/found context.
    TypeError {
        msg: Cow<'static, str>,
        expected: Cow<'static, str>,
        found: Cow<'static, str>,
        position: FilePosition,
    },

    /// Type mismatch between two TType values.
    TypeMismatch {
        expected: TType,
        found: TType,
        position: FilePosition,
    },

    /// Simple type error with a single message.
    SimpleTypeError {
        msg: Cow<'static, str>,
        position: FilePosition,
    },
}

// ───────────────────────────────────────────────────────────────────
// Display / Error trait implementations
// ───────────────────────────────────────────────────────────────────

impl fmt::Display for NovaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NovaError::File { msg } => write!(f, "File Error: {msg}"),
            NovaError::Lexing { msg, .. } => write!(f, "Lexing Error: {msg}"),
            NovaError::Parsing { msg, .. } => write!(f, "Error: {msg}"),
            NovaError::Compiler { msg, .. } => write!(f, "Compile Error: {msg}"),
            NovaError::Runtime { msg } => write!(f, "Runtime Error: {msg}"),
            NovaError::RuntimeWithPos { msg, .. } => write!(f, "Runtime Error: {msg}"),
            NovaError::TypeError { msg, .. } => write!(f, "Type Error: {msg}"),
            NovaError::TypeMismatch {
                expected, found, ..
            } => write!(f, "Type Mismatch: expected {expected}, found {found}"),
            NovaError::SimpleTypeError { msg, .. } => write!(f, "Type Error: {msg}"),
        }
    }
}

impl std::error::Error for NovaError {}

// ───────────────────────────────────────────────────────────────────
// Pretty-printing
// ───────────────────────────────────────────────────────────────────

impl NovaError {
    /// Display the error without source snippets (used in the REPL
    /// where there is no file to reference).
    pub fn show_without_position(&self) {
        match self {
            NovaError::File { msg } => {
                println!("\n  {} {}\n", "error:".bright_red().bold(), msg.bold());
            }

            NovaError::Lexing { msg, note, .. } => {
                println!("\n  {} {}", "error[Lexer]:".bright_red().bold(), msg.bold());
                render_note(note);
                render_footer();
            }

            NovaError::Parsing {
                msg, note, extra, ..
            } => {
                println!("\n  {} {}", "error:".bright_red().bold(), msg.bold());
                if let Some(extras) = extra {
                    for (extra_msg, _) in extras {
                        println!(
                            "  {} {}",
                            "note:".bright_yellow().bold(),
                            extra_msg.bright_yellow()
                        );
                    }
                }
                render_note(note);
                render_footer();
            }

            NovaError::Compiler { msg, note } => {
                println!(
                    "\n  {} {}",
                    "error[Compiler]:".bright_red().bold(),
                    msg.bold()
                );
                render_note(note);
                render_footer();
            }

            NovaError::Runtime { msg } => {
                println!(
                    "\n  {} {}\n",
                    "error[Runtime]:".bright_red().bold(),
                    msg.bold()
                );
            }

            NovaError::RuntimeWithPos { msg, .. } => {
                println!(
                    "\n  {} {}\n",
                    "error[Runtime]:".bright_red().bold(),
                    msg.bold()
                );
            }

            NovaError::TypeError {
                msg,
                expected,
                found,
                ..
            } => {
                println!("\n  {} {}", "error[Type]:".bright_red().bold(), msg.bold());
                println!(
                    "  {} {}    {} {}",
                    "expected:".bright_green().bold(),
                    expected.bright_green(),
                    "found:".bright_red().bold(),
                    found.bright_red()
                );
                render_footer();
            }

            NovaError::TypeMismatch {
                expected, found, ..
            } => {
                println!(
                    "\n  {} mismatched types",
                    "error[Type]:".bright_red().bold(),
                );
                println!(
                    "  {} {}    {} {}",
                    "expected:".bright_green().bold(),
                    format!("{expected}").bright_green(),
                    "found:".bright_red().bold(),
                    format!("{found}").bright_red()
                );
                render_footer();
            }

            NovaError::SimpleTypeError { msg, .. } => {
                println!(
                    "\n  {} {}\n",
                    "error[Type]:".bright_red().bold(),
                    msg.bold()
                );
            }
        }
    }

    /// Display the error with full source snippets and location info.
    pub fn show(&self) {
        match self {
            // ── File Error ──
            NovaError::File { msg } => {
                println!(
                    "\n  {} {}\n",
                    "error[File]:".bright_red().bold(),
                    msg.bold()
                );
            }

            // ── Lexing Error ──
            NovaError::Lexing {
                msg,
                note,
                position,
            } => {
                render_location_header("Lexing Error", position);
                render_snippet(position, msg);
                render_note(note);
                render_footer();
            }

            // ── Parsing Error ──
            NovaError::Parsing {
                msg,
                note,
                position,
                extra,
            } => {
                render_location_header("Error", position);
                render_snippet(position, msg);
                if let Some(extras) = extra {
                    render_extra(extras);
                }
                render_note(note);
                render_footer();
            }

            // ── Compiler Error ──
            NovaError::Compiler { msg, note } => {
                println!(
                    "\n  {} {}",
                    "error[Compiler]:".bright_red().bold(),
                    msg.bold()
                );
                render_note(note);
                render_footer();
            }

            // ── Runtime Error (no location) ──
            NovaError::Runtime { msg } => {
                println!(
                    "\n  {} {}\n",
                    "error[Runtime]:".bright_red().bold(),
                    msg.bold()
                );
            }

            // ── Runtime Error (with location) ──
            NovaError::RuntimeWithPos { msg, position } => {
                render_location_header("Runtime Error", position);
                render_snippet(position, msg);
                render_footer();
            }

            // ── Type Error ──
            NovaError::TypeError {
                msg,
                expected,
                found,
                position,
            } => {
                render_location_header("Type Error", position);
                render_snippet(
                    position,
                    &format!("expected `{}`, found `{}`", expected, found),
                );
                render_note(msg);
                render_footer();
            }

            // ── Type Mismatch ──
            NovaError::TypeMismatch {
                expected,
                found,
                position,
            } => {
                render_location_header("Type Mismatch", position);
                render_snippet(
                    position,
                    &format!("expected `{}`, found `{}`", expected, found),
                );
                render_footer();
            }

            // ── Simple Type Error ──
            NovaError::SimpleTypeError { msg, position } => {
                render_location_header("Type Error", position);
                render_snippet(position, msg);
                render_footer();
            }
        }
    }
}
