use crate::{fileposition::FilePosition, ttype::TType};
use colored::Colorize;
use std::{
    borrow::Cow,
    io::{self, BufRead},
    path::Path,
};

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<std::fs::File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn print_line(position: &FilePosition, msg: &str) {
    if let Ok(lines) = read_lines(position.filepath.as_deref().unwrap_or(Path::new(""))) {
        let line_number_width = position.line.to_string().chars().count();

        for (linenumber, line_content) in lines.enumerate() {
            let current_line = linenumber + 1;
            if current_line == position.line {
                if let Ok(line) = line_content {
                    // Print line number and line content with padding
                    println!("{:<width$} |", "", width = line_number_width);
                    println!(
                        "{:width$} | {}",
                        current_line,
                        line,
                        width = line_number_width
                    );

                    // Print marker line with padding for alignment
                    print!("{:<width$} |", "", width = line_number_width);
                    if let Some(mut row) = position.row.checked_sub(1) {
                        row += 1;
                        println!("{: <row$}{} {}", "", "^".red(), msg.bright_red(), row = row);
                    } else {
                        println!(" {}", "^".red());
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum NovaError {
    File {
        msg: Cow<'static, str>,
    },
    Lexing {
        msg: Cow<'static, str>,
        note: Cow<'static, str>,
        position: FilePosition,
    },
    Parsing {
        msg: Cow<'static, str>,
        note: Cow<'static, str>,
        position: FilePosition,
        extra: Option<Vec<(String, FilePosition)>>,
    },
    Compiler {
        msg: Cow<'static, str>,
        note: Cow<'static, str>,
    },
    Runtime {
        msg: Cow<'static, str>,
    },
    RuntimeWithPos {
        msg: Cow<'static, str>,
        position: FilePosition,
    },
    TypeError {
        msg: Cow<'static, str>,
        expected: Cow<'static, str>,
        found: Cow<'static, str>,
        position: FilePosition,
    },
    TypeMismatch {
        expected: TType,
        found: TType,
        position: FilePosition,
    },
    SimpleTypeError {
        msg: Cow<'static, str>,
        position: FilePosition,
    },
}

impl NovaError {
    pub fn show_without_position(&self) {
        match &self {
            NovaError::File { msg } => {
                println!("{}: {}", "File Error".red(), msg);
            }
            NovaError::Lexing { msg, note, .. } => {
                println!("{}: {}", "Lexing Error".bright_red(), msg);
                println!("{}: {}", "Note".bright_yellow(), note.bright_yellow());
            }
            NovaError::Parsing {
                msg, note, extra, ..
            } => {
                println!("{}: {}", "Parsing Error".bright_red(), msg);
                if let Some(extra_notes) = extra {
                    for (extra_msg, _) in extra_notes {
                        println!("{}: {}", "Note".bright_yellow(), extra_msg.bright_yellow());
                    }
                }
                println!("{}: {}", "Note".bright_yellow(), note.bright_yellow());
            }
            NovaError::Runtime { msg } => {
                println!("Runtime Error: {}", msg.bright_red());
            }
            NovaError::Compiler { msg, note } => {
                println!("{}: {}", "Compiling Error".bright_red(), msg.bright_red());
                println!("{}: {}", "Note".bright_yellow(), note.bright_yellow());
            }
            NovaError::RuntimeWithPos { msg, .. } => {
                println!("{}: {}", "Runtime Error".bright_red(), msg.bright_red());
            }
            NovaError::TypeError {
                msg,
                expected,
                found,
                ..
            } => {
                println!("{}: {}", "Type Error".bright_red(), msg.bright_red());
                println!("Expected type: {expected}\nFound type: {found}",);
            }
            NovaError::TypeMismatch {
                expected, found, ..
            } => {
                println!(
                    "{}: {}",
                    "Type Mismatch".bright_red(),
                    "Type Mismatch".bright_red()
                );
                println!("Expected type: {expected}\nFound type: {found}",);
            }
            NovaError::SimpleTypeError { msg, .. } => {
                println!("{}: {}", "Type Error".bright_red(), msg.bright_red());
            }
        }
    }

    pub fn show(&self) {
        match &self {
            NovaError::File { msg } => {
                println!("{}: {}", "File Error".red(), msg);
            }
            NovaError::Lexing {
                msg,
                note,
                position,
            } => {
                println!(
                    "{} in {}:{}:{}",
                    "Lexing Error".bright_red(),
                    position
                        .filepath
                        .as_deref()
                        .unwrap_or(Path::new("repl"))
                        .display(),
                    position.line,
                    position.row
                );
                print_line(position, msg);
                println!("{}: {}", "Note".bright_yellow(), note.bright_yellow());
            }
            NovaError::Parsing {
                msg,
                note,
                position,
                extra,
            } => {
                println!(
                    "{} in {}:{}:{}",
                    "Parsing Error".bright_red(),
                    position
                        .filepath
                        .as_deref()
                        .unwrap_or(Path::new("repl"))
                        .display(),
                    position.line,
                    position.row
                );
                print_line(position, msg);
                if let Some(extra_notes) = extra {
                    for (extra_msg, extra_position) in extra_notes {
                        print_line(extra_position, extra_msg);
                    }
                }
                println!("{}: {}", "Note".bright_yellow(), note.bright_yellow());
            }
            NovaError::Runtime { msg } => {
                println!("Runtime Error: {}", msg.bright_red());
            }
            NovaError::Compiler { msg, note } => {
                println!("{}", "Compiling Error".bright_red(),);
                println!("{}\n{}", msg.bright_red(), note.bright_yellow());
            }
            NovaError::RuntimeWithPos { msg, position } => {
                println!(
                    "{} in {}:{}:{}",
                    "Runtime Error".bright_red(),
                    position
                        .filepath
                        .as_deref()
                        .unwrap_or(Path::new("repl"))
                        .display(),
                    position.line,
                    position.row
                );
                print_line(position, msg);
            }
            NovaError::TypeError {
                msg,
                expected,
                found,
                position,
            } => {
                println!(
                    "{} in {}:{}:{}",
                    "Type Error".bright_red(),
                    position
                        .filepath
                        .as_deref()
                        .unwrap_or(Path::new("repl"))
                        .display(),
                    position.line,
                    position.row
                );
                print_line(
                    position,
                    &format!("Expected type: {}\nFound type: {}", expected, found),
                );
                println!("{}: {}", "Note".bright_yellow(), msg.bright_yellow());
            }
            NovaError::TypeMismatch {
                expected,
                found,
                position,
            } => {
                println!(
                    "{} in {}:{}:{}",
                    "Type Mismatch".bright_red(),
                    position
                        .filepath
                        .as_deref()
                        .unwrap_or(Path::new("repl"))
                        .display(),
                    position.line,
                    position.row
                );
                print_line(
                    position,
                    &format!("Expected type: {}\nFound type: {}", expected, found),
                );
            }
            NovaError::SimpleTypeError { msg, position } => {
                println!(
                    "{} in {}:{}:{}",
                    "Type Error".bright_red(),
                    position
                        .filepath
                        .as_deref()
                        .unwrap_or(Path::new("repl"))
                        .display(),
                    position.line,
                    position.row
                );
                print_line(position, msg);
            }
        }
    }
}
