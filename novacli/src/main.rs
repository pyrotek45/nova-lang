use common::error::NovaError;
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
    args.next(); // Skip the file path
    let command = args.next()?;

    let handle_error = |result: Result<(), NovaError>| {
        if let Err(e) = result {
            e.show();
            exit(1);
        }
    };

    let execute_command = |filepath: &Path, action: fn(NovaCore) -> Result<(), NovaError>| {
        let novacore = compile_file_or_exit(filepath);
        handle_error(action(novacore));
    };

    match command.as_str() {
        "run" => execute_command(args.next().as_ref().map(Path::new)?, NovaCore::run),
        "dbg" => execute_command(args.next().as_ref().map(Path::new)?, NovaCore::run_debug),
        "dis" => execute_command(args.next().as_ref().map(Path::new)?, NovaCore::dis_file),
        "time" => {
            let filepath: PathBuf = args.next()?.into();
            let novacore = compile_file_or_exit(&filepath);
            let start_time = std::time::Instant::now();
            let execution_result = novacore.run();
            println!("Execution time: {}ms", start_time.elapsed().as_millis());
            handle_error(execution_result);
        }
        "check" => {
            let filepath: PathBuf = args.next()?.into();
            let start_time = std::time::Instant::now();
            let novacore = compile_file_or_exit(&filepath);
            handle_error(novacore.check());
            println!("OK | Compile time: {}ms", start_time.elapsed().as_millis());
        }
        "repl" => repl_session(),
        _ => print_help(),
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
                line_editor.sync_history().unwrap();
                io::stdout().flush().unwrap();
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
                        line_editor.clear_screen().unwrap();
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
                            novarepl = states.last().unwrap().clone();
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
                            let session = pline.split_whitespace().collect::<Vec<&str>>()[1]
                                .parse::<usize>()
                                .unwrap();
                            if session < states.len() {
                                novarepl = states[session].clone();
                                states.truncate(session + 1);
                                prompt.left_prompt = DefaultPromptSegment::Basic(format!(
                                    "Session: {}  $",
                                    states.len()
                                ));
                            } else {
                                println!("Session not found");
                            }
                            continue;
                        }
                        // save to file
                        if pline.starts_with("save") {
                            let file = pline.split_whitespace().collect::<Vec<&str>>()[1];
                            // save the current session to a file
                            // check if the file exists
                            if std::path::Path::new(file).exists() {
                                println!("File already exists, do you want to overwrite it? (y/n)");
                                let mut response = String::new();
                                io::stdin().read_line(&mut response).unwrap();
                                if response.trim() != "y" {
                                    continue;
                                }
                            }
                            let mut file = std::fs::File::create(file).unwrap();
                            file.write_all(novarepl.current_repl.as_bytes()).unwrap();
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
    println!("HELP MENU");
    println!("\trun   [file]  // runs the file using the nova vm");
    println!("\tdbg   [file]  // debug the file");
    println!("\ttime  [file]  // time the file");
    println!("\tcheck [file]  // check if the file compiles");
    println!("\tdis   [file]  // disassemble the file");
    println!("\thelp          // displays this menu");
    println!("\trepl          // starts the repl");
    // repl mode commands
    println!("\nREPL MODE COMMANDS");
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

fn compile_file_or_exit(file: &Path) -> NovaCore {
    match novacore::NovaCore::new(file) {
        Ok(novacore) => novacore,
        Err(error) => {
            error.show();
            exit(1);
        }
    }
}

// repl code
