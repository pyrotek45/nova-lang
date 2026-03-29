use common::error::{NovaError, NovaResult};
use rand::Rng;
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

fn pop_float(state: &mut state::State) -> NovaResult<f64> {
    match pop(state)? {
        VmData::Float(v) => Ok(v),
        _ => Err(runtime_err("Expected a Float on the stack")),
    }
}

/// random(low, high) -> Int  (inclusive range)
pub fn random_int(state: &mut state::State) -> NovaResult<()> {
    let high = pop_int(state)?;
    let low = pop_int(state)?;
    let mut rng = rand::thread_rng();
    state
        .memory
        .stack
        .push(VmData::Int(rng.gen_range(low..=high)));
    Ok(())
}

/// randomFloat(low, high) -> Float  (half-open range [low, high))
pub fn random_float(state: &mut state::State) -> NovaResult<()> {
    let high = pop_float(state)?;
    let low = pop_float(state)?;
    let mut rng = rand::thread_rng();
    state
        .memory
        .stack
        .push(VmData::Float(rng.gen_range(low..high)));
    Ok(())
}

/// randomBool() -> Bool
pub fn random_bool(state: &mut state::State) -> NovaResult<()> {
    let mut rng = rand::thread_rng();
    state.memory.stack.push(VmData::Bool(rng.gen_bool(0.5)));
    Ok(())
}
