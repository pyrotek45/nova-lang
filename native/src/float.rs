use common::error::{NovaError, NovaResult};
use vm::memory_manager::VmData;
use vm::state;

pub fn int_to_float(state: &mut state::State) -> NovaResult<()> {
    let data = match state.memory.stack.pop() {
        Some(data) => data,
        None => {
            return Err(Box::new(NovaError::Runtime {
                msg: "Stack is empty".into(),
            }))
        }
    };
    let float = match data {
        VmData::Int(value) => value as f64,
        VmData::Float(value) => value,
        VmData::Bool(value) => {
            if value {
                1.0
            } else {
                0.0
            }
        }
        VmData::Char(value) => {
            if let Ok(parsed) = value.to_string().parse::<f64>() {
                parsed
            } else {
                state.memory.stack.push(VmData::None);
                return Ok(());
            }
        }
        VmData::Object(v) => {
            if let Some(obj) = state.memory.ref_from_heap(v) {
                if let Some(s) = obj.as_string() {
                    if let Ok(parsed) = s.parse::<f64>() {
                        parsed
                    } else {
                        state.memory.stack.push(VmData::None);
                        return Ok(());
                    }
                } else {
                    state.memory.stack.push(VmData::None);
                    return Ok(());
                }
            } else {
                state.memory.stack.push(VmData::None);
                return Ok(());
            }
        }
        _ => {
            state.memory.stack.push(VmData::None);
            return Ok(());
        }
    };
    state.memory.stack.push(VmData::Float(float));
    Ok(())
}
