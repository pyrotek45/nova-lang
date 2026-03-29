use std::{thread, time};

use common::error::NovaError;
use vm::memory_manager::VmData;
use vm::state;

pub fn sleep(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::Int(time)) = state.memory.stack.pop() {
        let delay = time::Duration::from_millis(time as u64);
        thread::sleep(delay);
    }
    Ok(())
}
