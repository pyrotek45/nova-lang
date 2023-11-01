use common::error::NovaError;
use vm::state::{self, Heap, VmData};

pub fn len(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::List(index)) = state.stack.pop() {
        if let Heap::List(array) = state.deref(index) {
            state.stack.push(VmData::Int(array.len() as i64))
        }
    }
    Ok(())
}
