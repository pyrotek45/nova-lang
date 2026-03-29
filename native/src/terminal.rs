use std::{io::stdout, time::Duration};

use common::error::NovaError;
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    event::{self, Event, KeyCode, KeyEvent},
    execute, terminal,
};
use vm::memory_manager::VmData;
use vm::state;

pub fn rawmode(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::Bool(bool)) = state.memory.stack.pop() {
        if bool {
            terminal::enable_raw_mode().expect("could not enable raw mode");
        } else {
            terminal::disable_raw_mode().expect("Could not disable raw mode")
        }
    }
    execute!(stdout(), MoveToNextLine(0)).unwrap();
    Ok(())
}

pub fn getch(state: &mut state::State) -> Result<(), NovaError> {
    if let Event::Key(KeyEvent {
        code: KeyCode::Char(character),
        modifiers: event::KeyModifiers::NONE,
        kind: _,
        state: _,
    }) = event::read().expect("Failed to read line")
    {
        state.memory.stack.push(VmData::Char(character))
    } else {
        state.memory.stack.push(VmData::None);
    }
    Ok(())
}

pub fn rawread(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::Int(time)) = state.memory.stack.pop() {
        if event::poll(Duration::from_millis(time as u64)).expect("Error") {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char(character),
                modifiers: event::KeyModifiers::NONE,
                kind: _,
                state: _,
            }) = event::read().expect("Failed to read line")
            {
                state.memory.stack.push(VmData::Char(character));
            } else {
                state.memory.stack.push(VmData::None);
            }
        } else {
            state.memory.stack.push(VmData::None);
        }
    }
    Ok(())
}

pub fn clear_screen(_state: &mut state::State) -> Result<(), NovaError> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All)).unwrap();
    execute!(stdout(), MoveTo(0, 0)).unwrap();
    Ok(())
}

pub fn hide_cursor(_state: &mut state::State) -> Result<(), NovaError> {
    execute!(stdout(), crossterm::cursor::Hide).unwrap();
    Ok(())
}

pub fn show_cursor(_state: &mut state::State) -> Result<(), NovaError> {
    execute!(stdout(), crossterm::cursor::Show).unwrap();
    Ok(())
}

pub fn retrieve_command_line_args(state: &mut state::State) -> Result<(), NovaError> {
    let args: Vec<String> = std::env::args().skip(3).collect();
    if args.is_empty() {
        state.memory.stack.push(VmData::None);
    } else {
        // Build a list of string objects
        let mut list_data = vec![];
        for arg in args {
            let str_idx = state.memory.allocate(vm::memory_manager::Object::string(arg));
            list_data.push(VmData::Object(str_idx));
        }
        state.memory.push_list(list_data);
    }
    Ok(())
}
