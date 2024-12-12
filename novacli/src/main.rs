use common::error::NovaError;
use novacore::NovaCore;
use reedline::{DefaultPromptSegment, DefaultValidator};
use std::{
    io::{self, Write},
    os::linux::raw::stat,
    process::exit,
};
use vm::state;

fn main() {
    if entry_command().is_none() {
        print_help();
        // TODO: add a repl
    }
}

fn entry_command() -> Option<()> {
    let mut args = std::env::args();
    args.next(); // Skip the file path
    let command = args.next()?;

    let handle_error = |result: Result<(), NovaError>| {
        if let Err(e) = result {
            e.show();
            exit(1);
        }
    };

    let execute_command = |filepath: String, action: fn(NovaCore) -> Result<(), NovaError>| {
        let novacore = compile_file_or_exit(&filepath);
        handle_error(action(novacore));
    };

    match command.as_str() {
        "run" => execute_command(args.next()?, NovaCore::run),
        "dbg" => execute_command(args.next()?, NovaCore::run_debug),
        "dis" => execute_command(args.next()?, NovaCore::dis_file),
        "time" => {
            let filepath = args.next()?;
            let novacore = compile_file_or_exit(&filepath);
            let start_time = std::time::Instant::now();
            let execution_result = novacore.run();
            println!("Execution time: {}ms", start_time.elapsed().as_millis());
            handle_error(execution_result);
        }
        "check" => {
            let filepath = args.next()?;
            let start_time = std::time::Instant::now();
            let novacore = compile_file_or_exit(&filepath);
            handle_error(novacore.check());
            println!("OK | Compile time: {}ms", start_time.elapsed().as_millis());
        }
        "repl" => {
            let mut novarepl = NovaCore::repl(); // Assuming NovaRepl is defined elsewhere
            loop {
                use reedline::{DefaultPrompt, Reedline, Signal};

                let validator = Box::new(DefaultValidator);

                let mut line_editor = Reedline::create().with_validator(validator);
                let mut prompt = DefaultPrompt::default();
                let mut states = vec![novarepl.clone()];
                prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                    "Session: {}  $",
                    states.len().to_string()
                ));

                loop {
                    let sig = line_editor.read_line(&prompt);
                    match sig {
                        Ok(Signal::Success(mut line)) => {
                            io::stdout().flush().unwrap();
                            //dbg!(line.clone());
                            match line.as_str() {
                                "exit" => {
                                    println!("Goodbye!");
                                    exit(0);
                                }
                                "clear" => {
                                    line_editor.clear_screen().unwrap();
                                    continue;
                                }
                                "new" => {
                                    states.clear();
                                    novarepl = NovaCore::repl();
                                    states.push(novarepl.clone());
                                    prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                                        "Session: {}  $",
                                        states.len().to_string()
                                    ));
                                    continue;
                                }
                                "help" => {
                                    print_help();
                                    continue;
                                }
                                pline => {
                                    if pline.starts_with("session") {
                                        let session =
                                            pline.split_whitespace().collect::<Vec<&str>>()[1]
                                                .parse::<usize>()
                                                .unwrap();
                                        if session < states.len() {
                                            novarepl = states[session].clone();
                                            states.truncate(session + 1);
                                            prompt.left_prompt = DefaultPromptSegment::Basic(
                                                format!("Session: {}  $", states.len().to_string()),
                                            );
                                        } else {
                                            println!("Session not found");
                                        }
                                        continue;
                                    }
                                }
                            }

                            if line.is_empty() {
                                continue;
                            }
                            line.push('\n');
                            // make a copy of the current repl and reload on error

                            let last_save = novarepl.clone();
                            match novarepl.run_line(&line) {
                                Ok(_) => {
                                    if !(line.contains("println") || line.contains("print")) {
                                        states.push(novarepl.clone());
                                    }
                                }
                                Err(e) => {
                                    e.show();
                                    novarepl = last_save
                                }
                            }
                            prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                                "Session: {}  $",
                                states.len().to_string()
                            ));
                        }
                        Ok(Signal::CtrlD) | Ok(Signal::CtrlC) => {
                            println!("Goodbye!");
                            exit(0);
                        }
                        x => {
                            println!("Event: {:?}", x);
                        }
                    }
                }
            }
        }
        _ => print_help(),
    }

    Some(())
}

fn print_help() {
    println!("Nova 0.1.0: by pyrotek45\n");
    println!("HELP MENU");
    println!("\trun   [file]  // runs the file using the nova vm");
    println!("\tdbg   [file]  // debug the file");
    println!("\ttime  [file]  // time the file");
    println!("\tcheck [file]  // check if the file compiles");
    println!("\tdis   [file]  // disassemble the file");
    println!("\thelp          // displays this menu");
    println!("\trepl          // starts the repl");
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

// repl code
