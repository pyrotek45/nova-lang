use common::error::NovaError;
use vm::state::{self, VmData};

pub fn int_to_char(state: &mut state::State) -> Result<(), NovaError> {
    match state.stack.pop() {
        Some(VmData::Int(ch)) => state.stack.push(VmData::Char((ch as u8) as char)),
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
