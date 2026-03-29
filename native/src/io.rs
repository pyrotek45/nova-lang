use common::error::{NovaError, NovaResult};
use std::{fs, io};
use vm::memory_manager::{ObjectType, VmData};
use vm::state;

pub fn read_line(state: &mut state::State) -> NovaResult<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        Box::new(NovaError::Runtime {
            msg: format!("Error reading line: {e}").into(),
        })
    })?;
    // removing newline token
    input.pop();
    state.memory.push_string(input);
    Ok(())
}

pub fn read_file(state: &mut state::State) -> NovaResult<()> {
    if let Some(VmData::Object(index)) = state.memory.stack.pop() {
        let path = {
            let obj = state
                .memory
                .ref_from_heap(index)
                .ok_or(Box::new(NovaError::Runtime {
                    msg: "Invalid heap reference".into(),
                }))?;
            obj.as_string().ok_or(Box::new(NovaError::Runtime {
                msg: "Expected a string path".into(),
            }))?
        };
        state.memory.dec(index);
        match fs::read_to_string(&path) {
            Ok(string) => {
                state.memory.push_string(string);
            }
            Err(e) => {
                return Err(Box::new(NovaError::Runtime {
                    msg: format!("Error reading file: {e}").into(),
                }))
            }
        }
    }
    Ok(())
}

fn printf_with_array(format_string: &str, args: Vec<String>) {
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

fn format_with_array(format_string: &str, args: Vec<String>) -> String {
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

    formatted
}

/// Helper to extract an array of strings from a list-of-strings Object.
/// In the refvm model, a list of strings is Object(List) whose data contains VmData::Object pointers
/// each pointing to an Object(String).
fn extract_string_list(state: &state::State, list_index: usize) -> NovaResult<Vec<String>> {
    let list_obj = state
        .memory
        .ref_from_heap(list_index)
        .ok_or(Box::new(NovaError::Runtime {
            msg: "Invalid heap reference for list".into(),
        }))?;
    match &list_obj.object_type {
        ObjectType::List => {
            let mut strings = vec![];
            for item in &list_obj.data {
                match item {
                    VmData::Object(str_idx) => {
                        let str_obj = state.memory.ref_from_heap(*str_idx).ok_or(Box::new(
                            NovaError::Runtime {
                                msg: "Invalid arguments for printf".into(),
                            },
                        ))?;
                        let s = str_obj.as_string().ok_or(Box::new(NovaError::Runtime {
                            msg: "Invalid arguments for printf".into(),
                        }))?;
                        strings.push(s);
                    }
                    _ => {
                        return Err(Box::new(NovaError::Runtime {
                            msg: "Invalid arguments for printf".into(),
                        }))
                    }
                }
            }
            Ok(strings)
        }
        _ => Err(Box::new(NovaError::Runtime {
            msg: "Invalid arguments for printf".into(),
        })),
    }
}

pub fn printf(state: &mut state::State) -> NovaResult<()> {
    let args = state.memory.stack.pop();
    let str_val = state.memory.stack.pop();

    if let (Some(VmData::Object(args_idx)), Some(VmData::Object(str_idx))) = (args, str_val) {
        let format_string = {
            let obj = state
                .memory
                .ref_from_heap(str_idx)
                .ok_or(Box::new(NovaError::Runtime {
                    msg: "Invalid arguments for printf".into(),
                }))?;
            obj.as_string().ok_or(Box::new(NovaError::Runtime {
                msg: "Invalid arguments for printf".into(),
            }))?
        };
        let strings = extract_string_list(state, args_idx)?;
        printf_with_array(&format_string, strings);
        state.memory.dec(args_idx);
        state.memory.dec(str_idx);
    } else {
        return Err(Box::new(NovaError::Runtime {
            msg: "Invalid arguments for printf".into(),
        }));
    }
    Ok(())
}

pub fn format(state: &mut state::State) -> NovaResult<()> {
    let args = state.memory.stack.pop();
    let str_val = state.memory.stack.pop();

    if let (Some(VmData::Object(args_idx)), Some(VmData::Object(str_idx))) = (args, str_val) {
        let format_string = {
            let obj = state
                .memory
                .ref_from_heap(str_idx)
                .ok_or(Box::new(NovaError::Runtime {
                    msg: "Invalid arguments for format".into(),
                }))?;
            obj.as_string().ok_or(Box::new(NovaError::Runtime {
                msg: "Invalid arguments for format".into(),
            }))?
        };
        let strings = extract_string_list(state, args_idx)?;
        let result = format_with_array(&format_string, strings);
        state.memory.dec(args_idx);
        state.memory.dec(str_idx);
        state.memory.push_string(result);
    } else {
        return Err(Box::new(NovaError::Runtime {
            msg: "Invalid arguments for format".into(),
        }));
    }
    Ok(())
}
