//use common::error::{runtime_error, NovaError};
use common::error::NovaError;
use std::{fs, io};
use vm::state::{self, Heap, VmData};

pub fn read_line(state: &mut state::State) -> Result<(), NovaError> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| NovaError::Runtime {
            msg: format!("Error reading line: {}", e),
        })?;
    // removing newline token
    input.pop();
    let index = state.allocate_string(input);
    state.stack.push(VmData::String(index));
    Ok(())
}

pub fn read_file(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::String(index)) = state.stack.pop() {
        if let Heap::String(path) = state.deref(index) {
            match fs::read_to_string(path) {
                Ok(string) => {
                    let index = state.allocate_string(string);
                    state.stack.push(VmData::String(index));
                }
                Err(e) => {
                    return Err(NovaError::Runtime {
                        msg: format!("Error reading file: {}", e),
                    })
                }
            }
        }
    }
    Ok(())
}
