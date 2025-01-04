use common::error::NovaError;
use vm::state::{self, Heap, VmData};

pub fn regex_match(state: &mut state::State) -> Result<(), NovaError> {
    let text = match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index).clone() {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };

    let pattern = match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };

    let re = match regex::Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(NovaError::Runtime {
                msg: format!("Invalid regex pattern: {e}").into(),
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
        Some(VmData::String(index)) => match state.get_ref(index).clone() {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };

    let pattern = match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };
    // need to continue to run the regex to capture all patterns in the text

    let re = match regex::Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(NovaError::Runtime {
                msg: format!("Invalid regex pattern: {e}").into(),
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
        let string_pos = state.allocate_string(capture.into());
        myarray.push(state.allocate_vmdata_to_heap(VmData::String(string_pos)));
    }

    let index = state.allocate_array(myarray);
    state.stack.push(VmData::List(index));
    state.gclock = false;
    Ok(())
}

// make a function that returns first capture from a regex match as a string and returns both index and string
pub fn regex_first(state: &mut state::State) -> Result<(), NovaError> {
    let text = match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index).clone() {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".to_string().into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".to_string().into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".to_string().into(),
            })
        }
    };

    let pattern = match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index) {
            Heap::String(str) => str,
            _ => {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".to_string().into(),
                })
            }
        },
        Some(_) => {
            return Err(NovaError::Runtime {
                msg: "Expected a string on the stack".to_string().into(),
            })
        }
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".to_string().into(),
            })
        }
    };
    // need to continue to run the regex to capture all patterns in the text

    let re = match regex::Regex::new(pattern) {
        Ok(re) => re,
        Err(e) => {
            return Err(NovaError::Runtime {
                msg: format!("Invalid regex pattern: {}", e).into(),
            })
        }
    };

    state.gclock = true;
    let captures = re.find(&text);

    // if no captures return none
    if captures.is_none() {
        state.stack.push(VmData::None);
        state.gclock = false;
        return Ok(());
    }
    // unwrap captures
    let captures = captures.unwrap();
    //dbg!(captures);
    // create a list of (int, int , string) to return
    let (start, end, str) = (
        captures.start(),
        captures.end(),
        captures.as_str().to_string(),
    );

    let string_pos = state.allocate_string(str.into());
    let start_pos = state.allocate_vmdata_to_heap(VmData::Int(start as i64));
    let end_pos = state.allocate_vmdata_to_heap(VmData::Int(end as i64));
    let string_pos = state.allocate_vmdata_to_heap(VmData::String(string_pos));
    let myarray = vec![start_pos, end_pos, string_pos];
    let index = state.allocate_array(myarray);
    state.stack.push(VmData::List(index));
    state.gclock = false;
    Ok(())
}
