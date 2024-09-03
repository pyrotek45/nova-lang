use novacore::NovaCore;
use std::process::exit;

fn main() {
    if entry_command().is_none() {
        print_help();
        // TODO: add a repl
    }
}

fn entry_command() -> Option<()> {
    let mut args = std::env::args();
    args.next(); // file path
    let command = args.next()?;

    match command.as_str() {
        "run" => {
            let filepath = args.next()?;
            let novacore = compile_file_or_exit(&filepath);

            if let Err(e) = novacore.run() {
                e.show();
                exit(1);
            }
        }

        "dbg" => {
            let filepath = args.next()?;
            let novacore = compile_file_or_exit(&filepath);

            if let Err(e) = novacore.run_debug() {
                e.show();
                exit(1);
            }
        }

        "dis" => {
            let filepath = args.next()?;
            let novacore = compile_file_or_exit(&filepath);

            if let Err(e) = novacore.dis_file() {
                e.show();
                exit(1);
            }
        }

        "time" => {
            let filepath = args.next()?;

            let start = std::time::Instant::now();
            let novacore = compile_file_or_exit(&filepath);

            let execution_start = std::time::Instant::now();
            let execution_result = novacore.run_time();

            println!(
                "execution time: {}ms",
                execution_start.elapsed().as_millis()
            );

            println!("total time: {}ms", start.elapsed().as_millis());

            if let Err(e) = execution_result {
                e.show();
                exit(1);
            }
        }

        "check" => {
            let filepath = args.next()?;

            let start = std::time::Instant::now();
            let novacore = compile_file_or_exit(&filepath);

            if let Err(e) = novacore.check() {
                e.show();
                exit(1);
            }

            println!("OK | compile time: {}ms", start.elapsed().as_millis());
        }

        // TODO: add repl
        _ => print_help(),
    }

    Some(())
}

fn print_help() {
    println!("Nova 0.1.0: by pyrotek45");
    println!();
    println!("HELP MENU");
    println!("\trun   [file]  // runs the file using the nova vm");
    println!("\tdbg   [file]  // debug the file");
    println!("\ttime  [file]  // time the file");
    println!("\tcheck [file]  // check if the file compiles");
    println!("\tdis   [file]  // disassemble the file");
    println!("\thelp          // displays this menu");
}

fn compile_file_or_exit(file: &str) -> NovaCore {
    match novacore::NovaCore::new(file) {
        Ok(novacore) => novacore,
        Err(error) => {
            error.show();
            exit(1);
        }
    }
}
