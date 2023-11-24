use std::io;

use common::error::NovaError;
use vm::state::{self, VmData};

pub fn read_line(state: &mut state::State) -> Result<(), NovaError> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {}
        Err(_) => {
            return Err(common::error::runtime_error(
                "Failed to readline".to_owned(),
            ))
        }
    }
    // removing newline token
    input.pop();
    //let index = state.allocate_string(input);
    //state.stack.push(VmData::String(index));
    Ok(())
}
