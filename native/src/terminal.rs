use std::{io::stdout, time::Duration};

use common::error::NovaError;
use crossterm::{
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEvent},
    execute, terminal,
};
use vm::state::{self, VmData};

pub fn rawmode(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::Bool(bool)) = state.stack.pop() {
        if bool {
            terminal::enable_raw_mode().expect("could not enable raw mode");
        } else {
            terminal::disable_raw_mode().expect("Could not disable raw mode")
        }
    }
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
        state.stack.push(VmData::Char(character))
    } else {
        state.stack.push(VmData::None);
    }
    Ok(())
}

pub fn rawread(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::Int(time)) = state.stack.pop() {
        if event::poll(Duration::from_millis(time as u64)).expect("Error") {
            if let Event::Key(KeyEvent {
                code: KeyCode::Char(character),
                modifiers: event::KeyModifiers::NONE,
                kind: _,
                state: _,
            }) = event::read().expect("Failed to read line")
            {
                state.stack.push(VmData::Char(character));
            } else {
                state.stack.push(VmData::None);
            }
        } else {
            state.stack.push(VmData::None);
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
    let args = std::env::args().skip(3);
    let mut myarray = vec![];
    state.gclock = true;
    let len = args.len();
    for arg in args {
        let string_pos = state.allocate_string(arg);
        myarray.push(state.allocate_vmdata_to_heap(VmData::String(string_pos)));
    }
    if len == 0 {
        state.stack.push(VmData::None);
    } else {
        let index = state.allocate_array(myarray);
        state.stack.push(VmData::List(index));
    }
    state.gclock = false;
    Ok(())
}
