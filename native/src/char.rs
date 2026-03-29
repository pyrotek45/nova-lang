use common::error::{NovaError, NovaResult};
use vm::memory_manager::VmData;
use vm::state;

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

fn pop_char(state: &mut state::State) -> NovaResult<char> {
    match pop(state)? {
        VmData::Char(c) => Ok(c),
        _ => Err(runtime_err("Expected a Char on the stack")),
    }
}

/// chr(n: Int) -> Char
pub fn int_to_char(state: &mut state::State) -> NovaResult<()> {
    match pop(state)? {
        VmData::Int(ch) => state.memory.stack.push(VmData::Char((ch as u8) as char)),
        _ => return Err(runtime_err("Expected an Int on the stack")),
    }
    Ok(())
}

/// Char::isAlpha(c) -> Bool
pub fn char_is_alpha(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state.memory.stack.push(VmData::Bool(c.is_alphabetic()));
    Ok(())
}

/// Char::isDigit(c) -> Bool
pub fn char_is_digit(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state.memory.stack.push(VmData::Bool(c.is_ascii_digit()));
    Ok(())
}

/// Char::isWhitespace(c) -> Bool
pub fn char_is_whitespace(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state.memory.stack.push(VmData::Bool(c.is_whitespace()));
    Ok(())
}

/// Char::isAlphanumeric(c) -> Bool
pub fn char_is_alphanumeric(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state.memory.stack.push(VmData::Bool(c.is_alphanumeric()));
    Ok(())
}

/// Char::toUpper(c) -> Char
pub fn char_to_upper(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state
        .memory
        .stack
        .push(VmData::Char(c.to_ascii_uppercase()));
    Ok(())
}

/// Char::toLower(c) -> Char
pub fn char_to_lower(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state
        .memory
        .stack
        .push(VmData::Char(c.to_ascii_lowercase()));
    Ok(())
}

/// Char::isUpper(c) -> Bool
pub fn char_is_upper(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state.memory.stack.push(VmData::Bool(c.is_uppercase()));
    Ok(())
}

/// Char::isLower(c) -> Bool
pub fn char_is_lower(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state.memory.stack.push(VmData::Bool(c.is_lowercase()));
    Ok(())
}
