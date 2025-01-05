//use common::error::{runtime_error, NovaError};
use common::error::NovaError;
use std::{fs, io, rc::Rc};
use vm::state::{self, Heap, VmData};

pub fn read_line(state: &mut state::State) -> Result<(), NovaError> {
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| NovaError::Runtime {
            msg: format!("Error reading line: {e}").into(),
        })?;
    // removing newline token
    input.pop();
    let index = state.allocate_string(input.into());
    state.stack.push(VmData::String(index));
    Ok(())
}

pub fn read_file(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::String(index)) = state.stack.pop() {
        if let Heap::String(path) = state.get_ref(index) {
            match fs::read_to_string(path.to_string()) {
                Ok(string) => {
                    let index = state.allocate_string(string.into());
                    state.stack.push(VmData::String(index));
                }
                Err(e) => {
                    return Err(NovaError::Runtime {
                        msg: format!("Error reading file: {e}").into(),
                    })
                }
            }
        }
    }
    Ok(())
}

fn printf_with_array(format_string: &str, args: Vec<Rc<str>>) {
    let mut arg_iter = args.iter();
    let mut formatted = String::new();
    let mut segments = format_string.split("{}").peekable();

    while let Some(segment) = segments.next() {
        formatted.push_str(segment);
        if segments.peek().is_some() {
            if let Some(arg) = arg_iter.next() {
                formatted.push_str(arg);
            } else {
                formatted.push_str("{}");
            }
        }
    }

    print!("{}", formatted);
}

// printf function for the VM that takes an array of strings // and the format string as arguments
pub fn printf(state: &mut state::State) -> Result<(), NovaError> {
    let mut strings = vec![];
    let args = state.stack.pop();
    let str_index = state.stack.pop();

    if let (Some(VmData::List(args)), Some(VmData::String(str_index))) = (args, str_index) {
        if let Heap::String(format_string) = state.get_ref(str_index) {
            if let Heap::List(args) = state.get_ref(args) {
                // gather string arguments
                for arg in args.iter() {
                    if let Heap::StringAddress(string) = state.get_ref(*arg) {
                        if let Heap::String(string) = state.get_ref(*string) {
                            strings.push(string);
                        } else {
                            return Err(NovaError::Runtime {
                                msg: "Invalid arguments for printf".into(),
                            });
                        }
                    } else {
                        return Err(NovaError::Runtime {
                            msg: "Invalid arguments for printf".into(),
                        });
                    }
                }
                printf_with_array(format_string, strings.into_iter().cloned().collect());
            } else {
                return Err(NovaError::Runtime {
                    msg: "Invalid arguments for printf".into(),
                });
            }
        } else {
            return Err(NovaError::Runtime {
                msg: "Invalid arguments for printf".into(),
            });
        }
    } else {
        return Err(NovaError::Runtime {
            msg: "Invalid arguments for printf".into(),
        });
    }
    Ok(())
}
