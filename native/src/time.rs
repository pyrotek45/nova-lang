use std::{thread, time};

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

fn pop_int(state: &mut state::State) -> NovaResult<i64> {
    match pop(state)? {
        VmData::Int(v) => Ok(v),
        _ => Err(runtime_err("Expected an Int on the stack")),
    }
}

/// sleep(ms: Int)
pub fn sleep(state: &mut state::State) -> NovaResult<()> {
    let ms = pop_int(state)?;
    let delay = time::Duration::from_millis(ms as u64);
    thread::sleep(delay);
    Ok(())
}

/// now() -> Int  (milliseconds since UNIX epoch)
pub fn now_ms(state: &mut state::State) -> NovaResult<()> {
    let ms = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64;
    state.memory.stack.push(VmData::Int(ms));
    Ok(())
}

/// nowSec() -> Float  (seconds since UNIX epoch, fractional)
pub fn now_sec(state: &mut state::State) -> NovaResult<()> {
    let secs = time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();
    state.memory.stack.push(VmData::Float(secs));
    Ok(())
}
