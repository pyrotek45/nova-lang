use common::error::NovaError;
use vm::memory_manager::VmData;
use vm::state;

/// assert(condition: Bool) - halts execution with error if condition is false
pub fn assert_true(state: &mut state::State) -> Result<(), NovaError> {
    match state.memory.stack.pop() {
        Some(VmData::Bool(true)) => Ok(()),
        Some(VmData::Bool(false)) => Err(NovaError::Runtime {
            msg: "Assertion failed".into(),
        }),
        _ => Err(NovaError::Runtime {
            msg: "assert: expected a Bool argument".into(),
        }),
    }
}

/// assert_msg(condition: Bool, message: String) - halts with custom message if false
pub fn assert_msg(state: &mut state::State) -> Result<(), NovaError> {
    let msg_val = state.memory.stack.pop();
    let cond_val = state.memory.stack.pop();

    let message = match msg_val {
        Some(VmData::Object(index)) => {
            let obj = state
                .memory
                .ref_from_heap(index)
                .ok_or(NovaError::Runtime {
                    msg: "assert_msg: invalid heap reference for message".into(),
                })?;
            let s = obj.as_string().unwrap_or_else(|| "???".to_string());
            state.memory.dec(index);
            s
        }
        _ => "???".to_string(),
    };

    match cond_val {
        Some(VmData::Bool(true)) => Ok(()),
        Some(VmData::Bool(false)) => Err(NovaError::Runtime {
            msg: format!("Assertion failed: {}", message).into(),
        }),
        _ => Err(NovaError::Runtime {
            msg: "assert_msg: expected a Bool as first argument".into(),
        }),
    }
}
