#[allow(unused_imports)]
use std::{
    process::exit,
    time::{Duration, Instant},
};

fn main() {
    if let (Some(command), Some(input), Some(filepath)) = (
        std::env::args().nth(1),
        std::env::args().nth(2),
        std::env::args().nth(3),
    ) {
        match (command.as_str(), input.as_str()) {
            ("run", "file") => {
                let _ = match novacore::NovaCore::new(&filepath) {
                    Ok(novacore) => match novacore.run() {
                        Ok(()) => {}
                        Err(error) => {
                            error.show();
                            exit(1)
                        }
                    },
                    Err(error) => {
                        error.show();
                        exit(1)
                    }
                };
            }
            ("dbg", "file") => {
                let _ = match novacore::NovaCore::new(&filepath) {
                    Ok(novacore) => match novacore.run_debug() {
                        Ok(()) => {}
                        Err(error) => {
                            error.show();
                            exit(1)
                        }
                    },
                    Err(error) => {
                        error.show();
                        exit(1)
                    }
                };
            }
            ("dis", "file") => {
                let _ = match novacore::NovaCore::new(&filepath) {
                    Ok(novacore) => match novacore.dis_file() {
                        Ok(()) => {}
                        Err(error) => {
                            error.show();
                            exit(1)
                        }
                    },
                    Err(error) => {
                        error.show();
                        exit(1)
                    }
                };
            }
            ("time", "file") => {
                let now = std::time::Instant::now();
                let _ = match novacore::NovaCore::new(&filepath) {
                    Ok(novacore) => match novacore.run() {
                        Ok(()) => {}
                        Err(error) => {
                            error.show();
                            exit(1)
                        }
                    },
                    Err(error) => {
                        error.show();
                        exit(1)
                    }
                };
                let exectime = now.elapsed();
                println!("Total time: {:#?}", exectime);
            }
            _ => {
                todo!()
            }
        }
    } else {
        if let Some(op) = std::env::args().nth(1) {
            match op.as_str() {
                "help" => {
                    println!("Nova 0.1: by pyrotek45");
                    println!();
                    println!("Help menu");
                    println!("\trun  [file] filepath\n\tdbg  [file] filepath\n\ttime [file] filepath\n\tdis  [file] filepath")
                }
                _ => {
                    // hopefully add a simple repl
                }
            }
        }
    }
}
