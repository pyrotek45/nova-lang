use colored::Colorize;

fn read_lines<P>(filename: P) -> std::io::Result<std::io::Lines<std::io::BufReader<std::fs::File>>>
where
    P: AsRef<std::path::Path>,
{
    let file = std::fs::File::open(filename)?;
    Ok(std::io::BufRead::lines(std::io::BufReader::new(file)))
}

fn print_line(line: usize, row: Option<usize>, file: &str, msg: &str) {
    if let Ok(lines) = read_lines(file) {
        // Consumes the iterator, returns an (Optional) String
        let mut linenumber = 1;
        for l in lines {
            if linenumber == line {
                let spaces = line.to_string().chars().count();
                if let Some(row) = row {
                    if let Ok(ip) = l {
                        for _ in 0..spaces {
                            print!(" ")
                        }
                        println!(" |");
                        println!("{} |  {} ", line, ip);

                        for _ in 0..spaces {
                            print!(" ")
                        }
                        print!(" |");

                        for _ in 0..=row {
                            print!(" ")
                        }
                        print!("{} {}", "^".red(), msg.bright_red());

                        println!();
                    }
                } else if let Ok(ip) = l {
                    for _ in 0..spaces {
                        print!(" ")
                    }
                    println!(" |");
                    println!("{} |  {} ", line, ip);

                    for _ in 0..spaces {
                        print!(" ")
                    }
                    print!(" |");

                    println!();
                }
            }
            linenumber += 1;
        }
    }
}

#[derive(Debug)]
pub enum ErrorType {
    File,
    Lexing,
    Parsing,
    Compiler,
    Runtime,
}

#[derive(Debug)]
pub struct NovaError {
    error: ErrorType,
    msg: String,
    note: String,
    line: usize,
    row: usize,
    filepath: String,
    extra: Option<Vec<(String, usize, usize)>>,
}

impl NovaError {
    #[inline(always)]
    pub fn show(&self) {
        match self.error {
            ErrorType::File => {
                println!("{}: {}", "File Error".red(), self.msg)
            }
            ErrorType::Lexing => {
                println!(
                    "{} in: {}:{}:{}",
                    "Lexing Error".bright_red(),
                    self.filepath,
                    self.line,
                    self.row
                );
                print_line(
                    self.line,
                    Some(self.row + 1),
                    &self.filepath,
                    &self.msg.red(),
                );
                println!("{}: {}", "Note".bright_yellow(), self.note.bright_yellow());
            }
            ErrorType::Parsing => {
                println!(
                    "{} in: {}:{}:{}",
                    "Parsing Error".bright_red(),
                    self.filepath,
                    self.line,
                    self.row
                );
                print_line(self.line, Some(self.row), &self.filepath, &self.msg.red());
                if let Some(extra) = &self.extra {
                    for (msg, line, row) in extra.iter() {
                        print_line(*line, Some(*row), &self.filepath, &msg.red());
                    }
                }
                println!("{}: {}", "Note".bright_yellow(), self.note.bright_yellow());
            }
            ErrorType::Runtime => {
                println!("Runtime Error: {}", self.msg.bright_red())
            }
            ErrorType::Compiler => {
                println!(
                    "{} in: {}:{}:{}",
                    "Compiling Error".bright_red(),
                    self.filepath,
                    self.line,
                    self.row
                );
                print_line(self.line, Some(self.row), &self.filepath, &self.msg.red());
                println!("{}: {}", "Note".bright_yellow(), self.note.bright_yellow());
            }
        }
    }
}

pub fn file_error(msg: String) -> NovaError {
    NovaError {
        error: ErrorType::File,
        msg,
        note: String::new(),
        line: 0,
        filepath: String::new(),
        row: 0,
        extra: None,
    }
}

pub fn lexer_error(
    msg: String,
    note: String,
    line: usize,
    row: usize,
    filepath: String,
) -> NovaError {
    NovaError {
        error: ErrorType::Lexing,
        msg,
        note,
        line,
        row,
        filepath,
        extra: None,
    }
}

pub fn parser_error(
    msg: String,
    note: String,
    line: usize,
    row: usize,
    filepath: String,
    extra: Option<Vec<(String, usize, usize)>>,
) -> NovaError {
    NovaError {
        error: ErrorType::Parsing,
        msg,
        note,
        line,
        row,
        filepath,
        extra,
    }
}

pub fn runtime_error(msg: String) -> NovaError {
    NovaError {
        error: ErrorType::Runtime,
        msg,
        note: String::new(),
        line: 0,
        filepath: String::new(),
        row: 0,
        extra: None,
    }
}

pub fn compiler_error(note: String, line: usize, filepath: String) -> NovaError {
    NovaError {
        error: ErrorType::Compiler,
        msg: String::new(),
        note,
        line: line + 1,
        filepath,
        row: 0,
        extra: None,
    }
}
