use vm::state::Heap;
use common::error::NovaError;
use vm::state::{self, VmData};

pub fn int_to_float(state: &mut state::State) -> Result<(), NovaError> {
    let data = match state.stack.pop() {
        Some(data) => data,
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".to_string(),
            })
        }
    };
    let float = match data {
        VmData::Int(value) => value as f64, // Convert integer to float
        VmData::Float(value) => value,      // Already a float, no conversion needed
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
                state.stack.push(VmData::None);
                return Ok(());
            }
        }
        VmData::String(v) => {
            if let Heap::String(str) = state.deref(v) {
                if let Ok(parsed) = str.parse::<f64>() {
                    parsed
                } else {
                    state.stack.push(VmData::None);
                    return Ok(());
                }
            } else {
                state.stack.push(VmData::None);
                return Ok(());
            }
        }
        _ => {
            state.stack.push(VmData::None);
            return Ok(());
        }
    };
    state.stack.push(VmData::Float(float));
    Ok(())
}