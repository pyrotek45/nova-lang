use std::rc::Rc;

use common::error::NovaError;
use vm::state::{self, Heap, VmData};

pub fn strlen(state: &mut state::State) -> Result<(), NovaError> {
    match state.stack.pop() {
        Some(VmData::String(index)) => match state.get_ref(index) {
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
        Some(VmData::String(index)) => match state.get_ref(index).clone() {
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

    let array = match state.get_ref(index).clone() {
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
        let char = state.get_ref(*item);
        match char {
            Heap::Char(c) => str.push(*c),
            _ => {
                state.gclock = false;
                return Err(NovaError::Runtime {
                    msg: "Expected a char in the list".into(),
                });
            }
        }
    }
    let index = state.allocate_string(str.into());
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

    let string: Rc<str> = match data {
        VmData::StackAddress(v) => format!("Stack pointer: {v}").into(),
        VmData::Function(v) => format!("function pointer: {v}").into(),
        VmData::Closure(v) => format!("closure pointer: {v}").into(),
        VmData::Int(v) => format!("{v}").into(),
        VmData::Float(v) => format!("{v}").into(),
        VmData::Bool(v) => format!("{v}").into(),
        VmData::Char(v) => format!("{v}").into(),
        VmData::List(v) => {
            let mut sbuild = String::new();
            if let Heap::List(array) = state.get_ref(v) {
                sbuild += "[";
                for (index, item) in array.iter().enumerate() {
                    if index > 0 {
                        sbuild += ", ";
                    }
                    sbuild += &format!("{:?}", state.get_ref(*item));
                }
                sbuild += "]";
            } else {
                return Err(NovaError::Runtime {
                    msg: "Expected a list in the heap".into(),
                });
            }
            sbuild.into()
        }
        VmData::Struct(v) => format!("Struct pointer: {v}").into(),
        VmData::String(v) => {
            let Heap::String(s) = state.get_ref(v) else {
                return Err(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                });
            };
            s.clone()
        }
        VmData::None => "None".into(),
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
            if let Heap::String(str) = state.get_ref(v) {
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
