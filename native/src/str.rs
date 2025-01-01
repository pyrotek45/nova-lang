use common::error::NovaError;
use vm::state::{self, Heap, VmData};

pub fn strlen(state: &mut state::State) -> Result<(), NovaError> {
    match state.stack.pop() {
        Some(VmData::String(index)) => match state.deref(index) {
            Heap::String(str) => {
                state.stack.push(VmData::Int(str.len() as i64));
                Ok(())
            }
            _ => Err(NovaError::Runtime {
                msg: "Expected a string in the heap".into(),
            }),
        },
        Some(_) => Err(NovaError::Runtime {
            msg: "Expected a string on the stack".into(),
        }),
        None => Err(NovaError::Runtime {
            msg: "Stack is empty".into(),
        }),
    }
}

pub fn str_to_chars(state: &mut state::State) -> Result<(), NovaError> {
    match state.stack.pop() {
        Some(VmData::String(index)) => match state.deref(index) {
            Heap::String(str) => {
                state.gclock = true;
                let mut myarray = vec![];
                for c in str.chars() {
                    myarray.push(state.allocate_vmdata_to_heap(VmData::Char(c)));
                }
                let index = state.allocate_array(myarray);
                state.stack.push(VmData::List(index));
                state.gclock = false;
                Ok(())
            }
            _ => Err(NovaError::Runtime {
                msg: "Expected a string in the heap".into(),
            }),
        },
        Some(_) => Err(NovaError::Runtime {
            msg: "Expected a string on the stack".into(),
        }),
        None => Err(NovaError::Runtime {
            msg: "Stack is empty".into(),
        }),
    }
}

pub fn chars_to_str(state: &mut state::State) -> Result<(), NovaError> {
    let data = match state.stack.pop() {
        Some(data) => data,
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };

    let index = match data {
        VmData::List(index) => index,
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected a list on the stack".into(),
            })
        }
    };

    let array = match state.deref(index) {
        Heap::List(array) => array,
        _ => {
            return Err(NovaError::Runtime {
                msg: "Expected a list in the heap".into(),
            })
        }
    };

    state.gclock = true;
    let mut str = String::new();
    for item in array.iter() {
        let char = state.deref(*item);
        match char {
            Heap::Char(c) => str.push(c),
            _ => {
                state.gclock = false;
                return Err(NovaError::Runtime {
                    msg: "Expected a char in the list".into(),
                });
            }
        }
    }
    let index = state.allocate_string(str);
    state.stack.push(VmData::String(index));
    state.gclock = false;

    Ok(())
}

pub fn to_string(state: &mut state::State) -> Result<(), NovaError> {
    let data = match state.stack.pop() {
        Some(data) => data,
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };

    let string = match data {
        VmData::StackAddress(v) => format!("Stack pointer: {v}"),
        VmData::Function(v) => format!("function pointer: {v}"),
        VmData::Closure(v) => format!("closure pointer: {v}"),
        VmData::Int(v) => format!("{v}"),
        VmData::Float(v) => format!("{v}"),
        VmData::Bool(v) => format!("{v}"),
        VmData::Char(v) => format!("{v}"),
        VmData::List(v) => {
            let mut sbuild = String::new();
            if let Heap::List(array) = state.deref(v) {
                sbuild += "[";
                for (index, item) in array.iter().enumerate() {
                    if index > 0 {
                        sbuild += ", ";
                    }
                    sbuild += &format!("{:?}", state.deref(*item));
                }
                sbuild += "]";
            } else {
                return Err(NovaError::Runtime {
                    msg: "Expected a list in the heap".into(),
                });
            }
            sbuild
        }
        VmData::Struct(v) => format!("Struct pointer: {v}"),
        VmData::String(v) => {
            let Heap::String(s) = state.deref(v) else {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                });
            };
            s
        }
        VmData::None => "None".to_string(),
    };

    let index = state.allocate_string(string);
    state.stack.push(VmData::String(index));
    Ok(())
}

pub fn to_int(state: &mut state::State) -> Result<(), NovaError> {
    let data = match state.stack.pop() {
        Some(data) => data,
        None => {
            return Err(NovaError::Runtime {
                msg: "Stack is empty".into(),
            })
        }
    };

    let int = match data {
        VmData::Int(value) => value, // Already an integer, no conversion needed
        VmData::Float(value) => value as i64, // Convert float to integer
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
                state.stack.push(VmData::None);
                return Ok(());
            }
        }
        VmData::String(v) => {
            if let Heap::String(str) = state.deref(v) {
                if let Ok(parsed) = str.parse::<i64>() {
                    parsed
                } else {
                    state.stack.push(VmData::None);
                    return Ok(());
                }
            } else {
                state.stack.push(VmData::None);
                return Ok(());
            }
        }
        _ => {
            state.stack.push(VmData::None);
            return Ok(());
        }
    };

    state.stack.push(VmData::Int(int));
    Ok(())
}
