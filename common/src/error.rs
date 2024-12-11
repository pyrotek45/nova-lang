use crate::{fileposition::FilePosition, ttype::TType};
use colored::Colorize;
use std::io::{self, BufRead};

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<std::fs::File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn print_line(position: &FilePosition, msg: &str) {
    if let Ok(lines) = read_lines(&position.filepath) {
        let line_number_width = position.line.to_string().chars().count();

        for (linenumber, line_content) in lines.enumerate() {
            let current_line = linenumber + 1;
            if current_line == position.line {
                if let Ok(line) = line_content {
                    // Print line number and line content with padding
                    print!("{:<width$} |\n", "", width = line_number_width);
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
        msg: String,
    },
    Lexing {
        msg: String,
        note: String,
        position: FilePosition,
    },
    Parsing {
        msg: String,
        note: String,
        position: FilePosition,
        extra: Option<Vec<(String, FilePosition)>>,
    },
    Compiler {
        msg: String,
        note: String,
    },
    Runtime {
        msg: String,
    },
    RuntimeWithPos {
        msg: String,
        position: FilePosition,
    },
    TypeError {
        msg: String,
        expected: String,
        found: String,
        position: FilePosition,
    },
    TypeMismatch {
        expected: TType,
        found: TType,
        position: FilePosition,
    },
    SimpleTypeError {
        msg: String,
        position: FilePosition,
    },
}

impl NovaError {
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
                    position.filepath,
                    position.line,
                    position.row
                );
                print_line(position, &msg);
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
                    position.filepath,
                    position.line,
                    position.row
                );
                print_line(position, &msg);
                if let Some(extra_notes) = extra {
                    for (extra_msg, extra_position) in extra_notes {
                        print_line(&extra_position, &extra_msg);
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
                    position.filepath,
                    position.line,
                    position.row
                );
                print_line(position, &msg);
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
                    position.filepath,
                    position.line,
                    position.row
                );
                print_line(
                    position,
                    &format!(
                        "Expected type: {}\nFound type: {}",
                        expected.to_string(),
                        found.to_string()
                    ),
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
                    position.filepath,
                    position.line,
                    position.row
                );
                print_line(
                    position,
                    &format!(
                        "Expected type: {}\nFound type: {}",
                        expected.to_string(),
                        found.to_string()
                    ),
                );
            }
            NovaError::SimpleTypeError { msg, position } => {
                println!(
                    "{} in {}:{}:{}",
                    "Type Error".bright_red(),
                    position.filepath,
                    position.line,
                    position.row
                );
                print_line(position, &msg);
            }
        }
    }
}
