use common::error::NovaError;
use vm::state::{self, Heap, VmData};

pub fn regex_match(state: &mut state::State) -> Result<(), NovaError> {
    let text = match state.stack.pop() {
        Some(VmData::String(index)) => match state.deref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".to_string(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".to_string(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".to_string(),
            })
        }
    };

    let pattern = match state.stack.pop() {
        Some(VmData::String(index)) => match state.deref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".to_string(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".to_string(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".to_string(),
            })
        }
    };

    let re = match regex::Regex::new(&pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(NovaError::Runtime {
                msg: format!("Invalid regex pattern: {}", e),
            })
        }
    };

    let result = re.is_match(&text);
    state.stack.push(VmData::Bool(result));
    Ok(())
}

// make a function that returns captures from a regex match as a list of strings
pub fn regex_captures(state: &mut state::State) -> Result<(), NovaError> {
    let text = match state.stack.pop() {
        Some(VmData::String(index)) => match state.deref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".to_string(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".to_string(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".to_string(),
            })
        }
    };

    let pattern = match state.stack.pop() {
        Some(VmData::String(index)) => match state.deref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".to_string(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".to_string(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".to_string(),
            })
        }
    };
    // need to continue to run the regex to capture all patterns in the text

    let re = match regex::Regex::new(&pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(NovaError::Runtime {
                msg: format!("Invalid regex pattern: {}", e),
            })
        }
    };

    state.gclock = true;
    let mut myarray = vec![];
    let captures: Vec<String> = re
        .find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();
    //dbg!(&text, &pattern, &captures);
    for i in 0..captures.len() {
        let capture = match captures.get(i) {
            Some(capture) => capture.as_str(),
            None => "",
        };
        let string_pos = state.allocate_string(capture.to_string());
        myarray.push(state.allocate_vmdata_to_heap(VmData::String(string_pos)));
    }

    let index = state.allocate_array(myarray);
    state.stack.push(VmData::List(index));
    state.gclock = false;
    Ok(())
}
