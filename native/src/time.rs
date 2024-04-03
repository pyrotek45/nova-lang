use std::{thread, time};

use common::error::NovaError;
use vm::state::{self, VmData};

pub fn sleep(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::Int(time)) = state.stack.pop() {
        let delay = time::Duration::from_millis(time as u64);
        thread::sleep(delay);
    } else {
        return Err(common::error::runtime_error("Failed to sleep".to_owned()));
    }
    Ok(())
}
