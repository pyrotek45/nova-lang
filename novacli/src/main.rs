use common::error::NovaResult;
use novacore::NovaCore;
use rand::Rng;
use reedline::{
    default_emacs_keybindings, ColumnarMenu, DefaultCompleter, DefaultPromptSegment,
    DefaultValidator, Emacs, FileBackedHistory, KeyCode, KeyModifiers, MenuBuilder, ReedlineEvent,
    ReedlineMenu,
};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::exit,
};
fn main() {
    if entry_command().is_none() {
        print_help();
        // TODO: add a repl
    }
}

fn entry_command() -> Option<()> {
    let mut args = std::env::args();
    args.next(); // Skip the binary path
    let command = args.next()?;

    let handle_error = |result: NovaResult<()>| {
        if let Err(e) = result {
            e.show();
            exit(1);
        }
    };

    let execute_command = |filepath: &Path, action: fn(NovaCore) -> NovaResult<()>| {
        let novacore = compile_file_or_exit(filepath);
        handle_error(action(novacore));
    };

    match command.as_str() {
        "run" => {
            let next_arg = match args.next() {
                Some(a) => a,
                None => {
                    // No arg — try main.nv
                    let main_nv = PathBuf::from("main.nv");
                    if main_nv.exists() {
                        eprintln!("(detected Nova project: running main.nv)");
                        execute_command(&main_nv, NovaCore::run);
                        return Some(());
                    } else {
                        eprintln!("Error: no file specified and no main.nv found in the current directory.");
                        eprintln!("  Usage: nova run <file.nv>");
                        eprintln!("  Or create a project: nova init myproject");
                        exit(1);
                    }
                }
            };
            if next_arg == "--git" {
                let github_path = args.next().unwrap_or_else(|| {
                    eprintln!("Error: --git requires a GitHub path like \"owner/repo/path/to/file.nv\"");
                    eprintln!("  Usage:    nova run --git owner/repo/path/file.nv [commit]");
                    eprintln!("  Example:  nova run --git pyrotek45/nova-lang/demo/fib.nv");
                    exit(1);
                });
                let commit = args.next(); // optional commit hash
                run_from_github(&github_path, commit.as_deref(), handle_error);
            } else {
                execute_command(Path::new(&next_arg), NovaCore::run);
            }
        }
        "dbg" => {
            let next_arg = args.next();
            if next_arg.as_deref() == Some("--git") {
                let (github_path, commit) = parse_git_args(&mut args, "dbg");
                let novacore = fetch_github_source(&github_path, commit.as_deref());
                handle_error(novacore.run_debug());
            } else {
                let file = resolve_file_from(next_arg, &mut args, "dbg")?;
                execute_command(&file, NovaCore::run_debug);
            }
        }
        "dis" => {
            let next_arg = args.next();
            if next_arg.as_deref() == Some("--git") {
                let (github_path, commit) = parse_git_args(&mut args, "dis");
                let novacore = fetch_github_source(&github_path, commit.as_deref());
                handle_error(novacore.dis_file());
            } else {
                let file = resolve_file_from(next_arg, &mut args, "dis")?;
                execute_command(&file, NovaCore::dis_file);
            }
        }
        "time" => {
            let next_arg = args.next();
            if next_arg.as_deref() == Some("--git") {
                let (github_path, commit) = parse_git_args(&mut args, "time");
                let novacore = fetch_github_source(&github_path, commit.as_deref());
                let start_time = std::time::Instant::now();
                let execution_result = novacore.run();
                println!("Execution time: {}ms", start_time.elapsed().as_millis());
                handle_error(execution_result);
            } else {
                let filepath = resolve_file_from(next_arg, &mut args, "time")?;
                let novacore = compile_file_or_exit(&filepath);
                let start_time = std::time::Instant::now();
                let execution_result = novacore.run();
                println!("Execution time: {}ms", start_time.elapsed().as_millis());
                handle_error(execution_result);
            }
        }
        "check" => {
            let next_arg = args.next();
            if next_arg.as_deref() == Some("--git") {
                let (github_path, commit) = parse_git_args(&mut args, "check");
                let novacore = fetch_github_source(&github_path, commit.as_deref());
                let start_time = std::time::Instant::now();
                handle_error(novacore.check());
                println!("OK | Compile time: {}ms", start_time.elapsed().as_millis());
            } else {
                let filepath = resolve_file_from(next_arg, &mut args, "check")?;
                let start_time = std::time::Instant::now();
                let novacore = compile_file_or_exit(&filepath);
                handle_error(novacore.check());
                println!("OK | Compile time: {}ms", start_time.elapsed().as_millis());
            }
        }
        "init" => {
            let name = args.next().unwrap_or_else(|| {
                eprintln!("Error: nova init requires a project name.");
                eprintln!("  Usage:    nova init <name> [--with owner/repo/folder]");
                eprintln!("  Example:  nova init myapp");
                eprintln!("  Example:  nova init mygame --with pyrotek45/nova-lang/std");
                exit(1);
            });
            let mut with_sources: Vec<String> = Vec::new();
            while let Some(flag) = args.next() {
                if flag == "--with" {
                    if let Some(src) = args.next() {
                        with_sources.push(src);
                    } else {
                        eprintln!("Error: --with requires a GitHub folder path (owner/repo/folder).");
                        exit(1);
                    }
                } else {
                    eprintln!("Warning: unknown flag '{}', ignoring.", flag);
                }
            }
            nova_init(&name, &with_sources);
        }
        "test" => {
            nova_test(&mut args);
        }
        "install" => {
            let name = args.next().unwrap_or_else(|| {
                eprintln!("Error: nova install requires a library name and a GitHub path.");
                eprintln!("  Usage:    nova install <name> <owner/repo/folder>");
                eprintln!("  Example:  nova install std pyrotek45/nova-lang/std");
                exit(1);
            });
            let repo_path = args.next().unwrap_or_else(|| {
                eprintln!("Error: nova install requires a GitHub path after the library name.");
                eprintln!("  Usage:    nova install <name> <owner/repo/folder>");
                eprintln!("  Example:  nova install std pyrotek45/nova-lang/std");
                exit(1);
            });
            nova_install(&name, &repo_path);
        }
        "remove" => {
            let name = args.next().unwrap_or_else(|| {
                eprintln!("Error: nova remove requires a library name.");
                eprintln!("  Usage:    nova remove <name>");
                eprintln!("  Example:  nova remove std");
                exit(1);
            });
            nova_remove(&name);
        }
        "repl" => repl_session(),
        "help" => print_help(),
        _ => {
            eprintln!("Error: unknown command '{}'.", command);
            eprintln!("  Run 'nova help' to see available commands.\n");
            print_help();
            exit(1);
        }
    }

    Some(())
}

fn repl_session() -> ! {
    let mut novarepl = NovaCore::repl();
    // print pretty welcome message in ascii art
    let banners = [
        r#"
     _______   ____________   _________   
     \      \  \_____  \   \ /   /  _  \  
     /   |   \  /   |   \   Y   /  /_\  \ 
    /    |    \/    |    \     /    |    \
    \____|__  /\_______  /\___/\____|__  /
            \/         \/              \/
    "#,
        r#"
     _        _______           _______ 
    ( (    /|(  ___  )|\     /|(  ___  )
    |  \  ( || (   ) || )   ( || (   ) |
    |   \ | || |   | || |   | || (___) |
    | (\ \) || |   | |( (   ) )|  ___  |
    | | \   || |   | | \ \_/ / | (   ) |
    | )  \  || (___) |  \   /  | )   ( |
    |/    )_)(_______)   \_/   |/     \|
    "#,
        r#"
        .-') _                    (`-.     ('-.     
        ( OO ) )                 _(OO  )_  ( OO ).-. 
    ,--./ ,--,'  .-'),-----. ,--(_/   ,. \ / . --. / 
    |   \ |  |\ ( OO'  .-.  '\   \   /(__/ | \-.  \  
    |    \|  | )/   |  | |  | \   \ /   /.-'-'  |  | 
    |  .     |/ \_) |  |\|  |  \   '   /, \| |_.'  | 
    |  |\    |    \ |  | |  |   \     /__) |  .-.  | 
    |  | \   |     `'  '-'  '    \   /     |  | |  | 
    `--'  `--'       `-----'      `-'      `--' `--' 
    "#,
        r#"
    ::::    :::  ::::::::  :::     :::     :::     
    :+:+:   :+: :+:    :+: :+:     :+:   :+: :+:   
    :+:+:+  +:+ +:+    +:+ +:+     +:+  +:+   +:+  
    +#+ +:+ +#+ +#+    +:+ +#+     +:+ +#++:++#++: 
    +#+  +#+#+# +#+    +#+  +#+   +#+  +#+     +#+ 
    #+#   #+#+# #+#    #+#   #+#+#+#   #+#     #+# 
    ###    ####  ########      ###     ###     ### 
    "#,
        r#"
    888b    |   ,88~-_   Y88b      /      e      
    |Y88b   |  d888   \   Y88b    /      d8b     
    | Y88b  | 88888    |   Y88b  /      /Y88b    
    |  Y88b | 88888    |    Y888/      /  Y88b   
    |   Y88b|  Y888   /      Y8/      /____Y88b  
    |    Y888   `88_-~        Y      /      Y88b
    "#,
        r#"                      
    @@@  @@@  @@@@@@  @@@  @@@  @@@@@@  
    @@!@!@@@ @@!  @@@ @@!  @@@ @@!  @@@ 
    @!@@!!@! @!@  !@! @!@  !@! @!@!@!@! 
    !!:  !!! !!:  !!!  !: .:!  !!:  !!! 
    ::    :   : :. :     ::     :   : :
    "#,
        r#"
    [...     [..    [....     [..         [..      [.       
    [. [..   [..  [..    [..   [..       [..      [. ..     
    [.. [..  [..[..        [..  [..     [..      [.  [..    
    [..  [.. [..[..        [..   [..   [..      [..   [..   
    [..   [. [..[..        [..    [.. [..      [...... [..  
    [..    [. ..  [..     [..      [....      [..       [.. 
    [..      [..    [....           [..      [..         [..
    "#,
        r#"                                                     
    888b      88    ,ad8888ba,   8b           d8   db         
    8888b     88   d8"'    `"8b  `8b         d8'  d88b        
    88 `8b    88  d8'        `8b  `8b       d8'  d8'`8b       
    88  `8b   88  88          88   `8b     d8'  d8'  `8b      
    88   `8b  88  88          88    `8b   d8'  d8YaaaaY8b     
    88    `8b 88  Y8,        ,8P     `8b d8'  d8""""""""8b    
    88     `8888   Y8a.    .a8P       `888'  d8'        `8b   
    88      `888    `"Y8888Y"'         `8'  d8'          `8b  
    "#,
        r#"
    `...     `..    `....     `..         `..      `.       
    `. `..   `..  `..    `..   `..       `..      `. ..     
    `.. `..  `..`..        `..  `..     `..      `.  `..    
    `..  `.. `..`..        `..   `..   `..      `..   `..   
    `..   `. `..`..        `..    `.. `..      `...... `..  
    `..    `. ..  `..     `..      `....      `..       `.. 
    `..      `..    `....           `..      `..         `..
    "#,
        r#"
    ===========================================
    =  =======  ====    ====  ====  =====  ====
    =   ======  ===  ==  ===  ====  ====    ===
    =    =====  ==  ====  ==  ====  ===  ==  ==
    =  ==  ===  ==  ====  ==  ====  ==  ====  =
    =  ===  ==  ==  ====  ==   ==   ==  ====  =
    =  ====  =  ==  ====  ===  ==  ===        =
    =  =====    ==  ====  ===  ==  ===  ====  =
    =  ======   ===  ==  =====    ====  ====  =
    =  =======  ====    =======  =====  ====  =
    ===========================================
    "#,
        r#"
    _           _    _  _  _  _    _           _       _          
   (_) _       (_) _(_)(_)(_)(_)_ (_)         (_)    _(_)_        
   (_)(_)_     (_)(_)          (_)(_)         (_)  _(_) (_)_      
   (_)  (_)_   (_)(_)          (_)(_)_       _(_)_(_)     (_)_    
   (_)    (_)_ (_)(_)          (_)  (_)     (_) (_) _  _  _ (_)   
   (_)      (_)(_)(_)          (_)   (_)   (_)  (_)(_)(_)(_)(_)   
   (_)         (_)(_)_  _  _  _(_)    (_)_(_)   (_)         (_)   
   (_)         (_)  (_)(_)(_)(_)        (_)     (_)         (_)
    "#,
    ];
    // print a random banner from the list

    println!(
        "{}",
        banners[rand::thread_rng().gen_range(0..banners.len())]
    );
    println!("Welcome to Nova 0.1.0: Made with Love <3 : Pyrotek45 ");
    println!("Type 'help' for a list of commands");
    // Assuming NovaRepl is defined elsewhere
    use reedline::{DefaultPrompt, Reedline, Signal};

    let validator = Box::new(DefaultValidator);
    let history = Box::new(
        FileBackedHistory::with_file(100, "history.txt".into())
            .expect("Error configuring history with file"),
    );

    let commands = vec![
        "exit".into(),
        "show".into(),
        "clear".into(),
        "new".into(),
        "help".into(),
        "session".into(),
        "save".into(),
        "keep".into(),
        "banner".into(),
        "back".into(),
        "ast".into(),
        // common functions
        "println".into(),
    ];
    let completer = Box::new(DefaultCompleter::new_with_wordlen(commands, 2));
    // Use the interactive menu to select options from the completer
    let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));
    // Set up the required keybindings
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".to_string()),
            ReedlineEvent::MenuNext,
        ]),
    );

    let edit_mode = Box::new(Emacs::new(keybindings));
    let mut line_editor = Reedline::create()
        .with_validator(validator)
        .with_history(history)
        .with_completer(completer)
        .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
        .with_edit_mode(edit_mode);

    let mut prompt = DefaultPrompt::default();
    let mut states = vec![novarepl.clone()];
    prompt.left_prompt = DefaultPromptSegment::Basic(format!("Session: {}  $", states.len()));
    prompt.right_prompt = DefaultPromptSegment::WorkingDirectory;
    loop {
        let sig = line_editor.read_line(&prompt);
        match sig {
            Ok(Signal::Success(mut line)) => {
                let _ = line_editor.sync_history();
                let _ = io::stdout().flush();
                //dbg!(line.clone());
                match line.as_str() {
                    "show" => {
                        // print current session
                        print!("{}", novarepl.current_repl);
                        continue;
                    }
                    "exit" => {
                        println!("Goodbye!");
                        exit(0);
                    }
                    "clear" => {
                        let _ = line_editor.clear_screen();
                        continue;
                    }
                    "new" => {
                        states.clear();
                        novarepl = NovaCore::repl();
                        states.push(novarepl.clone());
                        prompt.left_prompt =
                            DefaultPromptSegment::Basic(format!("Session: {}  $", states.len()));
                        continue;
                    }
                    "help" => {
                        print_help();
                        continue;
                    }
                    "banner" => {
                        println!(
                            "{}",
                            banners[rand::thread_rng().gen_range(0..banners.len())]
                        );
                        continue;
                    }
                    "back" => {
                        if states.len() > 1 {
                            states.pop();
                            if let Some(last) = states.last() {
                                novarepl = last.clone();
                            }
                            prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                                "Session: {}  $",
                                states.len()
                            ));
                        } else {
                            println!("No more sessions to go back to");
                        }
                        continue;
                    }
                    pline => {
                        if pline.starts_with("session") {
                            let parts: Vec<&str> = pline.split_whitespace().collect();
                            let session = parts.get(1).and_then(|s| s.parse::<usize>().ok());
                            match session {
                                Some(session) if session < states.len() => {
                                    novarepl = states[session].clone();
                                    states.truncate(session + 1);
                                    prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                                        "Session: {}  $",
                                        states.len()
                                    ));
                                }
                                _ => {
                                    println!("Session not found");
                                }
                            }
                            continue;
                        }
                        // save to file
                        if pline.starts_with("save") {
                            let parts: Vec<&str> = pline.split_whitespace().collect();
                            let Some(filename) = parts.get(1) else {
                                println!("Usage: save <filename>");
                                continue;
                            };
                            // check if the file exists
                            if std::path::Path::new(filename).exists() {
                                println!("File already exists, do you want to overwrite it? (y/n)");
                                let mut response = String::new();
                                if io::stdin().read_line(&mut response).is_err() {
                                    println!("Failed to read input");
                                    continue;
                                }
                                if response.trim() != "y" {
                                    continue;
                                }
                            }
                            match std::fs::File::create(filename) {
                                Ok(mut file) => {
                                    let _ = file.write_all(b"module repl\n\n");
                                    let _ = file.write_all(novarepl.current_repl.as_bytes());
                                }
                                Err(e) => {
                                    println!("Failed to create file: {}", e);
                                }
                            }
                            continue;
                        }

                        // store state even if println | print is used
                        if pline.starts_with("keep") {
                            // strip the store command
                            let mut line =
                                pline.split_whitespace().collect::<Vec<&str>>()[1..].join(" ");
                            // dbg!(line.clone());
                            if line.is_empty() {
                                continue;
                            }
                            line.push('\n');
                            // make a copy of the current repl and reload on error

                            let last_save = novarepl.clone();
                            match novarepl.run_line(&line, true) {
                                Ok(_) => {
                                    states.push(novarepl.clone());
                                    prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                                        "Session: {}  $",
                                        states.len()
                                    ));
                                }
                                Err(e) => {
                                    e.show_without_position();
                                    novarepl = last_save
                                }
                            }

                            continue;
                        }
                        // store state even if println | print is used
                        if pline.starts_with("ast") {
                            // strip the store command
                            let mut line =
                                pline.split_whitespace().collect::<Vec<&str>>()[1..].join(" ");

                            // dbg!(line.clone());
                            if line.is_empty() {
                                continue;
                            }

                            line.push('\n');
                            // make a copy of the current repl and reload on error

                            let last_save = novarepl.clone();
                            match novarepl.run_line(&line, true) {
                                Ok(_) => {
                                    println!("{:#?}", novarepl.parser.ast.clone());
                                    states.push(novarepl.clone());
                                    prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                                        "Session: {}  $",
                                        states.len()
                                    ));
                                }
                                Err(e) => {
                                    e.show_without_position();
                                    novarepl = last_save
                                }
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
                match novarepl.run_line(&line, false) {
                    Ok(_) => {
                        if !(line.contains("println") || line.contains("print")) {
                            states.push(novarepl.clone());
                        }
                        prompt.left_prompt =
                            DefaultPromptSegment::Basic(format!("Session: {}  $", states.len()));
                    }
                    Err(e) => {
                        e.show_without_position();
                        novarepl = last_save
                    }
                }
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

fn print_help() {
    println!("Nova 0.1.0: by pyrotek45\n");
    println!("COMMANDS");
    println!("\trun   [file]                    // run a file (or main.nv if inside a project)");
    println!("\trun   --git owner/repo/path     // run a file directly from GitHub");
    println!("\tcheck [file]                    // check if the file compiles");
    println!("\tcheck --git owner/repo/path     // type-check a file from GitHub");
    println!("\ttime  [file]                    // time a file's execution");
    println!("\ttime  --git owner/repo/path     // time a GitHub file's execution");
    println!("\tdis   [file]                    // disassemble a file");
    println!("\tdis   --git owner/repo/path     // disassemble a file from GitHub");
    println!("\tdbg   [file]                    // debug a file");
    println!("\tdbg   --git owner/repo/path     // debug a file from GitHub");
    println!("\ttest  [dir]                     // run all test_*.nv files in tests/ (or given dir)");
    println!("\tinit  <name> [--with repo/path] // create a new project (optionally fetch a folder)");
    println!("\tinstall <name> <repo/path>      // install a library into libs/<name>/");
    println!("\tremove  <name>                  // remove a library from libs/<name>/");
    println!("\trepl                            // start the interactive repl");
    println!("\thelp                            // display this menu");
    println!("\nPROJECT STRUCTURE");
    println!("\tA Nova project is any folder with a main.nv file.");
    println!("\tIf [file] is omitted, Nova looks for main.nv in the current directory.");
    println!("\tPut shared modules in libs/ and import with: import libs.mylib");
    println!("\tUse super to go up a directory:               import super.libs.mylib");
    println!("\tPut tests in tests/ and run with:             nova test");
    println!("\nGITHUB IMPORTS");
    println!("\tIn source:  import @ \"pyrotek45/nova-lang/std/core.nv\"");
    println!("\tWith lock:  import @ \"pyrotek45/nova-lang/std/core.nv\" ! \"abc1234\"");
    // repl mode commands
    println!("\nREPL COMMANDS");
    println!("\tshow           // show the current session");
    println!("\texit           // exit the repl");
    println!("\tclear          // clear the screen");
    println!("\tnew            // start a new session");
    println!("\thelp           // display this menu");
    println!("\tsession [num]  // switch to a session");
    println!("\tsave [file]    // save the current session to a file");
    println!("\tkeep [code]    // keep the current session");
    println!("\tbanner         // print a random banner");
    println!("\tast [code]     // print the ast of the code");
    println!("\tback           // go back to the previous session");
}

fn nova_init(name: &str, with_sources: &[String]) {
    use std::fs;

    let project_dir = PathBuf::from(name);
    if project_dir.exists() {
        eprintln!("Error: directory '{}' already exists.", name);
        exit(1);
    }

    // Create project directory structure
    fs::create_dir_all(project_dir.join("libs")).unwrap_or_else(|e| {
        eprintln!("Error: could not create project directory: {}", e);
        exit(1);
    });
    fs::create_dir_all(project_dir.join("tests")).unwrap_or_else(|e| {
        eprintln!("Error: could not create tests directory: {}", e);
        exit(1);
    });

    // Write main.nv
    let has_libs = !with_sources.is_empty();
    let main_content = if has_libs {
        format!(
            r#"module main

// Nova project: {}
// Run with:  nova run  (from inside the project folder)
// Or:        nova run {}/main.nv

// Import standard libraries from libs/
import libs.core

fn main() {{
    println("Hello from {}!")
}}

main()
"#,
            name, name, name
        )
    } else {
        format!(
            r#"module main

// Nova project: {}
// Run with:  nova run  (from inside the project folder)
// Or:        nova run {}/main.nv

fn main() {{
    println("Hello from {}!")
}}

main()
"#,
            name, name, name
        )
    };

    fs::write(project_dir.join("main.nv"), main_content).unwrap_or_else(|e| {
        eprintln!("Error: could not write main.nv: {}", e);
        exit(1);
    });

    // Write a starter test
    let test_content = r#"module test_example

assert(1 + 1 == 2, "basic addition")
assert(true, "truth")

println("PASS: test_example")
"#
    .to_string();

    fs::write(project_dir.join("tests/test_example.nv"), test_content).unwrap_or_else(|e| {
        eprintln!("Error: could not write tests/test_example.nv: {}", e);
        exit(1);
    });

    println!("Created {}/main.nv", name);
    println!("Created {}/libs/", name);
    println!("Created {}/tests/test_example.nv", name);

    // Fetch --with sources into libs/
    for source in with_sources {
        fetch_github_folder_to_libs(source, &project_dir.join("libs"));
    }

    println!("\nProject '{}' is ready!", name);
    println!("  cd {}", name);
    println!("  nova run");
    println!("  nova test");
}

fn nova_install(name: &str, github_path: &str) {
    use std::fs;

    let libs_dir = PathBuf::from("libs");
    if !libs_dir.exists() {
        // Create libs/ if it doesn't exist yet (user might be in a basic project)
        fs::create_dir_all(&libs_dir).unwrap_or_else(|e| {
            eprintln!("Error: could not create libs/ directory: {}", e);
            exit(1);
        });
    }

    let target_dir = libs_dir.join(name);
    if target_dir.exists() {
        eprintln!(
            "Library '{}' is already installed at libs/{}.",
            name, name
        );
        eprintln!("  To update it, remove it first:  nova remove {}", name);
        exit(1);
    }

    fs::create_dir_all(&target_dir).unwrap_or_else(|e| {
        eprintln!("Error: could not create libs/{}: {}", name, e);
        exit(1);
    });

    let fetched = fetch_github_folder_to_libs(github_path, &target_dir);

    if fetched == 0 {
        // Clean up the empty directory so the user isn't left in a broken state
        let _ = fs::remove_dir_all(&target_dir);
        eprintln!(
            "\nError: no files were fetched. Library '{}' was not installed.",
            name
        );
        eprintln!("  Check that the repository and folder path are correct.");
        exit(1);
    }

    println!(
        "\nInstalled '{}' into libs/{}.",
        name, name
    );
    println!("  Import with:  import libs.{}.modulename", name);
}

fn nova_remove(name: &str) {
    use std::fs;

    let target_dir = PathBuf::from("libs").join(name);
    if !target_dir.exists() {
        eprintln!("Error: library '{}' is not installed (libs/{} does not exist).", name, name);
        exit(1);
    }

    fs::remove_dir_all(&target_dir).unwrap_or_else(|e| {
        eprintln!("Error: could not remove libs/{}: {}", name, e);
        exit(1);
    });

    println!("Removed '{}' from libs/.", name);
}

/// Fetch all .nv files from a GitHub folder and save them into libs/.
/// Uses the GitHub Contents API to list the directory, then fetches each file.
/// Returns the number of files fetched, or 0 on failure.
fn fetch_github_folder_to_libs(github_path: &str, libs_dir: &Path) -> usize {
    let parts: Vec<&str> = github_path.splitn(3, '/').collect();
    if parts.len() < 3 {
        eprintln!(
            "Warning: invalid --with path \"{}\". Expected owner/repo/folder. Skipping.",
            github_path
        );
        return 0;
    }
    let owner = parts[0];
    let repo = parts[1];
    let folder_path = parts[2];

    // Use GitHub Contents API to list the directory
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/contents/{}?ref=main",
        owner, repo, folder_path
    );

    eprintln!("Fetching folder listing from {}...", api_url);
    let body = match ureq::get(&api_url)
        .set("User-Agent", "nova-lang-cli")
        .call()
    {
        Ok(resp) => match resp.into_string() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Warning: could not read API response: {}", e);
                return 0;
            }
        },
        Err(e) => {
            eprintln!(
                "Warning: could not fetch folder listing: {}\n  \
                 Check that the repository is public and the folder path is correct.\n  \
                 Path: {}/{}/{}",
                e, owner, repo, folder_path
            );
            return 0;
        }
    };

    // Simple JSON parsing: extract "download_url" and "name" fields for files
    // The API returns an array of objects. We look for "type":"file" entries
    // and extract their download_url.
    let mut fetched = 0;
    let mut i = 0;
    let bytes = body.as_bytes();
    while i < bytes.len() {
        // Find each "download_url" field
        if let Some(pos) = body[i..].find("\"download_url\":") {
            let start = i + pos + "\"download_url\":".len();
            // Skip whitespace and opening quote
            let rest = &body[start..];
            if let Some(quote_start) = rest.find('"') {
                let url_start = start + quote_start + 1;
                if let Some(quote_end) = body[url_start..].find('"') {
                    let url = &body[url_start..url_start + quote_end];
                    if url != "null" && url.ends_with(".nv") {
                        // Extract filename from URL
                        let filename = url.rsplit('/').next().unwrap_or("unknown.nv");
                        let dest = libs_dir.join(filename);

                        // Fetch the actual file
                        match ureq::get(url).call() {
                            Ok(resp) => match resp.into_string() {
                                Ok(content) => {
                                    std::fs::write(&dest, content).unwrap_or_else(|e| {
                                        eprintln!("  Error writing {}: {}", dest.display(), e);
                                    });
                                    println!("  Fetched {} -> libs/{}", filename, filename);
                                    fetched += 1;
                                }
                                Err(e) => {
                                    eprintln!("  Warning: could not read {}: {}", filename, e);
                                }
                            },
                            Err(e) => {
                                eprintln!("  Warning: could not fetch {}: {}", filename, e);
                            }
                        }
                    }
                    i = url_start + quote_end + 1;
                } else {
                    break;
                }
            } else {
                break;
            }
        } else {
            break;
        }
    }

    if fetched == 0 {
        eprintln!(
            "  Warning: no .nv files found in {}/{}/{}",
            owner, repo, folder_path
        );
    } else {
        println!("  Fetched {} files from {}", fetched, github_path);
    }
    fetched
}

/// Run all test_*.nv files in a tests/ directory and report results.
fn nova_test(args: &mut std::env::Args) {
    let test_dir = if let Some(dir) = args.next() {
        PathBuf::from(dir)
    } else {
        PathBuf::from("tests")
    };

    if !test_dir.exists() || !test_dir.is_dir() {
        eprintln!("Error: test directory '{}' not found.", test_dir.display());
        eprintln!("  Create a tests/ folder and add test_*.nv files.");
        eprintln!("  Or specify a directory: nova test path/to/tests");
        exit(1);
    }

    // Collect test_*.nv files
    let mut test_files: Vec<PathBuf> = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&test_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("test_") && name.ends_with(".nv") {
                    test_files.push(path);
                }
            }
        }
    }
    test_files.sort();

    if test_files.is_empty() {
        eprintln!("No test files found in {}.", test_dir.display());
        eprintln!("  Test files must be named test_*.nv");
        exit(1);
    }

    println!("========================================");
    println!("  Nova Test Runner");
    println!("========================================");
    println!("Running {} test files from {}/\n", test_files.len(), test_dir.display());

    let mut pass = 0;
    let mut fail = 0;
    let mut failures: Vec<String> = Vec::new();

    for test_file in &test_files {
        let test_name = test_file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Compile and run
        let result = novacore::NovaCore::new(test_file);
        match result {
            Ok(novacore) => match novacore.run() {
                Ok(()) => {
                    println!("  \u{2713} {}", test_name);
                    pass += 1;
                }
                Err(e) => {
                    println!("  \u{2717} {} (runtime error)", test_name);
                    e.show();
                    fail += 1;
                    failures.push(test_name.to_string());
                }
            },
            Err(e) => {
                println!("  \u{2717} {} (compile error)", test_name);
                e.show();
                fail += 1;
                failures.push(test_name.to_string());
            }
        }
    }

    println!("\n========================================");
    println!("  {} passed, {} failed", pass, fail);
    println!("========================================");

    if !failures.is_empty() {
        println!("\nFailed tests:");
        for f in &failures {
            println!("  - {}", f);
        }
        exit(1);
    } else {
        println!("\nAll tests passed! \u{2713}");
    }
}

fn compile_file_or_exit(file: &Path) -> NovaCore {
    match novacore::NovaCore::new(file) {
        Ok(novacore) => novacore,
        Err(error) => {
            error.show();
            exit(1);
        }
    }
}

/// Resolve a file path when the first arg has already been consumed.
/// `first` is the already-read arg (or None if there was none).
fn resolve_file_from(
    first: Option<String>,
    _args: &mut std::env::Args,
    cmd: &str,
) -> Option<PathBuf> {
    if let Some(arg) = first {
        Some(PathBuf::from(arg))
    } else {
        let main_nv = PathBuf::from("main.nv");
        if main_nv.exists() {
            eprintln!("(detected Nova project: running main.nv)");
            Some(main_nv)
        } else {
            eprintln!("Error: no file specified and no main.nv found in the current directory.");
            eprintln!("  Usage: nova {} <file.nv>", cmd);
            eprintln!("  Or create a project: nova init myproject");
            exit(1);
        }
    }
}

/// Parse the --git arguments: github_path (required), commit (optional).
fn parse_git_args(args: &mut std::env::Args, cmd: &str) -> (String, Option<String>) {
    let github_path = args.next().unwrap_or_else(|| {
        eprintln!("Error: --git requires a GitHub path like \"owner/repo/path/to/file.nv\"");
        eprintln!("  Usage:    nova {} --git owner/repo/path/file.nv [commit]", cmd);
        eprintln!(
            "  Example:  nova {} --git pyrotek45/nova-lang/demo/fib.nv",
            cmd
        );
        exit(1);
    });
    let commit = args.next();
    (github_path, commit)
}

fn run_from_github(
    github_path: &str,
    commit: Option<&str>,
    handle_error: impl Fn(common::error::NovaResult<()>),
) {
    let novacore = fetch_github_source(github_path, commit);
    handle_error(novacore.run());
}

/// Fetch a `.nv` file from GitHub and compile it, returning a ready-to-use NovaCore.
fn fetch_github_source(github_path: &str, commit: Option<&str>) -> NovaCore {
    let parts: Vec<&str> = github_path.splitn(3, '/').collect();
    if parts.len() < 3 {
        eprintln!(
            "Error: invalid GitHub path \"{}\"\n  \
             Expected format: owner/repo/path/to/file.nv\n  \
             Example: nova run --git pyrotek45/nova-lang/demo/fib.nv",
            github_path
        );
        exit(1);
    }
    let owner = parts[0];
    let repo = parts[1];
    let file_path = parts[2];
    let branch = commit.unwrap_or("main");

    let url = format!(
        "https://raw.githubusercontent.com/{}/{}/{}/{}",
        owner, repo, branch, file_path
    );

    eprintln!("Fetching {}...", url);
    let source = match ureq::get(&url).call() {
        Ok(resp) => match resp.into_string() {
            Ok(body) => body,
            Err(e) => {
                eprintln!("Error: could not read response body: {}", e);
                exit(1);
            }
        },
        Err(e) => {
            eprintln!("Error: could not fetch file from GitHub: {}", e);
            if commit.is_some() {
                eprintln!("  Check that commit \"{}\" exists and the file path is correct.", branch);
            } else {
                eprintln!("  Check that the repository is public and the file path is correct.");
                eprintln!("  Tip: specify a commit hash as a third argument to lock the version.");
            }
            exit(1);
        }
    };

    let virtual_path = PathBuf::from(format!("github://{}/{}/{}", owner, repo, file_path));
    NovaCore::from_source(&source, &virtual_path)
}

// repl code
