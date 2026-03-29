use std::io::{stdout, Write};
use std::time::Duration;

use common::error::{NovaError, NovaResult};
use crossterm::{
    cursor::{MoveTo, MoveToNextLine},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal,
};
use vm::memory_manager::VmData;
use vm::state;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn runtime_err(msg: impl Into<std::borrow::Cow<'static, str>>) -> Box<NovaError> {
    Box::new(NovaError::Runtime { msg: msg.into() })
}

fn pop(state: &mut state::State) -> NovaResult<VmData> {
    state
        .memory
        .stack
        .pop()
        .ok_or(runtime_err("Stack is empty"))
}

fn pop_int(state: &mut state::State) -> NovaResult<i64> {
    match pop(state)? {
        VmData::Int(v) => Ok(v),
        _ => Err(runtime_err("Expected an Int on the stack")),
    }
}

fn pop_string(state: &mut state::State) -> NovaResult<String> {
    match pop(state)? {
        VmData::Object(index) => {
            let s = state
                .memory
                .ref_from_heap(index)
                .and_then(|o| o.as_string())
                .ok_or(runtime_err("Expected a string object"))?;
            state.memory.dec(index);
            Ok(s)
        }
        _ => Err(runtime_err("Expected a String on the stack")),
    }
}

// ---------------------------------------------------------------------------
// Existing functions
// ---------------------------------------------------------------------------

/// terminal::rawmode(Bool) -> Void
pub fn rawmode(state: &mut state::State) -> NovaResult<()> {
    if let Some(VmData::Bool(b)) = state.memory.stack.pop() {
        if b {
            terminal::enable_raw_mode()
                .map_err(|e| runtime_err(format!("rawmode enable failed: {e}")))?;
        } else {
            terminal::disable_raw_mode()
                .map_err(|e| runtime_err(format!("rawmode disable failed: {e}")))?;
        }
    }
    execute!(stdout(), MoveToNextLine(0))
        .map_err(|e| runtime_err(format!("rawmode cursor move failed: {e}")))?;
    Ok(())
}

/// terminal::getch() -> Option(Char)
pub fn getch(state: &mut state::State) -> NovaResult<()> {
    let event = event::read().map_err(|e| runtime_err(format!("getch read failed: {e}")))?;
    if let Event::Key(KeyEvent {
        code: KeyCode::Char(character),
        modifiers: event::KeyModifiers::NONE,
        kind: _,
        state: _,
    }) = event
    {
        state.memory.stack.push(VmData::Char(character))
    } else {
        state.memory.stack.push(VmData::None);
    }
    Ok(())
}

/// terminal::rawread(Int) -> Option(Char)
pub fn rawread(state: &mut state::State) -> NovaResult<()> {
    if let Some(VmData::Int(time)) = state.memory.stack.pop() {
        let ms = if time < 0 { 0u64 } else { time as u64 };
        let ready = event::poll(Duration::from_millis(ms))
            .map_err(|e| runtime_err(format!("rawread poll failed: {e}")))?;
        if ready {
            let ev = event::read()
                .map_err(|e| runtime_err(format!("rawread read failed: {e}")))?;
            if let Event::Key(KeyEvent {
                code: KeyCode::Char(character),
                modifiers: event::KeyModifiers::NONE,
                kind: _,
                state: _,
            }) = ev
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

/// terminal::clearScreen() -> Void
pub fn clear_screen(_state: &mut state::State) -> NovaResult<()> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All))
        .map_err(|e| runtime_err(format!("clearScreen failed: {e}")))?;
    execute!(stdout(), MoveTo(0, 0))
        .map_err(|e| runtime_err(format!("clearScreen cursor move failed: {e}")))?;
    Ok(())
}

/// terminal::hideCursor() -> Void
pub fn hide_cursor(_state: &mut state::State) -> NovaResult<()> {
    execute!(stdout(), crossterm::cursor::Hide)
        .map_err(|e| runtime_err(format!("hideCursor failed: {e}")))?;
    Ok(())
}

/// terminal::showCursor() -> Void
pub fn show_cursor(_state: &mut state::State) -> NovaResult<()> {
    execute!(stdout(), crossterm::cursor::Show)
        .map_err(|e| runtime_err(format!("showCursor failed: {e}")))?;
    Ok(())
}

/// terminal::args() -> Option([String])
pub fn retrieve_command_line_args(state: &mut state::State) -> NovaResult<()> {
    let args: Vec<String> = std::env::args().skip(3).collect();
    if args.is_empty() {
        state.memory.stack.push(VmData::None);
    } else {
        // Build a list of string objects
        let mut list_data = vec![];
        for arg in args {
            let str_idx = state
                .memory
                .allocate(vm::memory_manager::Object::string(arg));
            list_data.push(VmData::Object(str_idx));
        }
        state.memory.push_list(list_data);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// New functions
// ---------------------------------------------------------------------------

/// terminal::moveTo(col: Int, row: Int) -> Void
/// Move the cursor to column `col`, row `row` (0-based).
pub fn move_to(state: &mut state::State) -> NovaResult<()> {
    let row = pop_int(state)? as u16;
    let col = pop_int(state)? as u16;
    execute!(stdout(), MoveTo(col, row)).map_err(|e| runtime_err(format!("moveTo failed: {e}")))?;
    Ok(())
}

/// terminal::getSize() -> (Int, Int)
/// Returns (width, height) of the terminal.
pub fn get_size(state: &mut state::State) -> NovaResult<()> {
    let (w, h) = terminal::size().map_err(|e| runtime_err(format!("getSize failed: {e}")))?;
    // Build a tuple object and push it on the stack
    let tuple =
        vm::memory_manager::Object::tuple(vec![VmData::Int(w as i64), VmData::Int(h as i64)]);
    let idx = state.memory.allocate(tuple);
    state.memory.stack.push(VmData::Object(idx));
    Ok(())
}

/// terminal::setForeground(r: Int, g: Int, b: Int) -> Void
/// Set the text foreground colour to an RGB value.
pub fn set_foreground(state: &mut state::State) -> NovaResult<()> {
    let b = pop_int(state)? as u8;
    let g = pop_int(state)? as u8;
    let r = pop_int(state)? as u8;
    execute!(stdout(), SetForegroundColor(Color::Rgb { r, g, b }))
        .map_err(|e| runtime_err(format!("setForeground failed: {e}")))?;
    Ok(())
}

/// terminal::setBackground(r: Int, g: Int, b: Int) -> Void
/// Set the text background colour to an RGB value.
pub fn set_background(state: &mut state::State) -> NovaResult<()> {
    let b_val = pop_int(state)? as u8;
    let g = pop_int(state)? as u8;
    let r = pop_int(state)? as u8;
    execute!(stdout(), SetBackgroundColor(Color::Rgb { r, g, b: b_val }))
        .map_err(|e| runtime_err(format!("setBackground failed: {e}")))?;
    Ok(())
}

/// terminal::resetColor() -> Void
/// Reset foreground and background colours to the terminal default.
pub fn reset_color(_state: &mut state::State) -> NovaResult<()> {
    execute!(stdout(), ResetColor).map_err(|e| runtime_err(format!("resetColor failed: {e}")))?;
    Ok(())
}

/// terminal::print(String) -> Void
/// Write a string directly to stdout without a trailing newline.
pub fn term_print(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    print!("{}", s);
    Ok(())
}

/// terminal::flush() -> Void
/// Flush stdout.
pub fn flush(_state: &mut state::State) -> NovaResult<()> {
    stdout()
        .flush()
        .map_err(|e| runtime_err(format!("flush failed: {e}")))?;
    Ok(())
}

/// terminal::enableMouse() -> Void
/// Enable mouse event capture.
pub fn enable_mouse(_state: &mut state::State) -> NovaResult<()> {
    execute!(stdout(), crossterm::event::EnableMouseCapture)
        .map_err(|e| runtime_err(format!("enableMouse failed: {e}")))?;
    Ok(())
}

/// terminal::disableMouse() -> Void
/// Disable mouse event capture.
pub fn disable_mouse(_state: &mut state::State) -> NovaResult<()> {
    execute!(stdout(), crossterm::event::DisableMouseCapture)
        .map_err(|e| runtime_err(format!("disableMouse failed: {e}")))?;
    Ok(())
}
