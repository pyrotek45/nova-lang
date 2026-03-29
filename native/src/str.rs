use common::error::{NovaError, NovaResult};
use vm::memory_manager::{ObjectType, VmData};
use vm::state;

pub fn strlen(state: &mut state::State) -> NovaResult<()> {
    match state.memory.stack.pop() {
        Some(VmData::Object(index)) => {
            if let Some(obj) = state.memory.ref_from_heap(index) {
                if let Some(s) = obj.as_string() {
                    state.memory.stack.push(VmData::Int(s.len() as i64));
                    // dec ref since we popped without going through memory.pop()
                    state.memory.dec(index);
                    Ok(())
                } else {
                    state.memory.dec(index);
                    Err(Box::new(NovaError::Runtime {
                        msg: "Expected a string object".into(),
                    }))
                }
            } else {
                Err(Box::new(NovaError::Runtime {
                    msg: "Invalid heap reference".into(),
                }))
            }
        }
        Some(_) => Err(Box::new(NovaError::Runtime {
            msg: "Expected a string on the stack".into(),
        })),
        None => Err(Box::new(NovaError::Runtime {
            msg: "Stack is empty".into(),
        })),
    }
}

pub fn str_to_chars(state: &mut state::State) -> NovaResult<()> {
    match state.memory.stack.pop() {
        Some(VmData::Object(index)) => {
            if let Some(obj) = state.memory.ref_from_heap(index) {
                if let Some(s) = obj.as_string() {
                    let chars: Vec<VmData> = s.chars().map(VmData::Char).collect();
                    state.memory.dec(index);
                    state.memory.push_list(chars);
                    Ok(())
                } else {
                    state.memory.dec(index);
                    Err(Box::new(NovaError::Runtime {
                        msg: "Expected a string object".into(),
                    }))
                }
            } else {
                Err(Box::new(NovaError::Runtime {
                    msg: "Invalid heap reference".into(),
                }))
            }
        }
        Some(_) => Err(Box::new(NovaError::Runtime {
            msg: "Expected a string on the stack".into(),
        })),
        None => Err(Box::new(NovaError::Runtime {
            msg: "Stack is empty".into(),
        })),
    }
}

pub fn chars_to_str(state: &mut state::State) -> NovaResult<()> {
    let data = match state.memory.stack.pop() {
        Some(data) => data,
        None => {
            return Err(Box::new(NovaError::Runtime {
                msg: "Stack is empty".into(),
            }))
        }
    };

    let index = match data {
        VmData::Object(index) => index,
        _ => {
            return Err(Box::new(NovaError::Runtime {
                msg: "Expected a list on the stack".into(),
            }))
        }
    };

    let chars_string = {
        let obj = state
            .memory
            .ref_from_heap(index)
            .ok_or(Box::new(NovaError::Runtime {
                msg: "Invalid heap reference".into(),
            }))?;
        match &obj.object_type {
            ObjectType::List => {
                let mut s = String::new();
                for item in &obj.data {
                    match item {
                        VmData::Char(c) => s.push(*c),
                        _ => {
                            return Err(Box::new(NovaError::Runtime {
                                msg: "Expected a char in the list".into(),
                            }))
                        }
                    }
                }
                s
            }
            _ => {
                return Err(Box::new(NovaError::Runtime {
                    msg: "Expected a list object".into(),
                }))
            }
        }
    };

    state.memory.dec(index);
    state.memory.push_string(chars_string);
    Ok(())
}

pub fn to_string(state: &mut state::State) -> NovaResult<()> {
    let data = match state.memory.stack.pop() {
        Some(data) => data,
        None => {
            return Err(Box::new(NovaError::Runtime {
                msg: "Stack is empty".into(),
            }))
        }
    };

    let string: String = match data {
        VmData::StackAddress(v) => format!("Stack pointer: {v}"),
        VmData::Function(v) => format!("function pointer: {v}"),
        VmData::Int(v) => format!("{v}"),
        VmData::Float(v) => format!("{v}"),
        VmData::Bool(v) => format!("{v}"),
        VmData::Char(v) => format!("{v}"),
        VmData::Object(v) => {
            let s = state.memory.print_heap_object(v, 0);
            state.memory.dec(v);
            s
        }
        VmData::None => "None".to_string(),
    };

    state.memory.push_string(string);
    Ok(())
}

pub fn to_int(state: &mut state::State) -> NovaResult<()> {
    let data = match state.memory.stack.pop() {
        Some(data) => data,
        None => {
            return Err(Box::new(NovaError::Runtime {
                msg: "Stack is empty".into(),
            }))
        }
    };

    let int = match data {
        VmData::Int(value) => value,
        VmData::Float(value) => value as i64,
        VmData::Bool(value) => {
            if value {
                1
            } else {
                0
            }
        }
        VmData::Char(value) => {
            if let Ok(parsed) = value.to_string().parse::<i64>() {
                parsed
            } else {
                state.memory.stack.push(VmData::None);
                return Ok(());
            }
        }
        VmData::Object(v) => {
            if let Some(obj) = state.memory.ref_from_heap(v) {
                if let Some(s) = obj.as_string() {
                    state.memory.dec(v);
                    if let Ok(parsed) = s.parse::<i64>() {
                        parsed
                    } else {
                        state.memory.stack.push(VmData::None);
                        return Ok(());
                    }
                } else {
                    state.memory.dec(v);
                    state.memory.stack.push(VmData::None);
                    return Ok(());
                }
            } else {
                state.memory.stack.push(VmData::None);
                return Ok(());
            }
        }
        _ => {
            state.memory.stack.push(VmData::None);
            return Ok(());
        }
    };

    state.memory.stack.push(VmData::Int(int));
    Ok(())
}
