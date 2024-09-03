use common::error::NovaError;
use vm::state::{self, VmData};

pub fn chr(state: &mut state::State) -> Result<(), NovaError> {
    match state.stack.pop() {
        Some(VmData::Int(ch)) => state.stack.push(VmData::Char((ch as u8) as char)),
        _ => {}
    }
    Ok(())
}
