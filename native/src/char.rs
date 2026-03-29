use common::error::NovaError;
use vm::memory_manager::VmData;
use vm::state;

pub fn int_to_char(state: &mut state::State) -> Result<(), NovaError> {
    match state.memory.stack.pop() {
        Some(VmData::Int(ch)) => state.memory.stack.push(VmData::Char((ch as u8) as char)),
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected an integer on the stack".into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    }
    Ok(())
}
