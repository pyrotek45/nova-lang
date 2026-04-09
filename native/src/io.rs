use common::error::{NovaError, NovaResult};
use std::{fs, io};
use vm::memory_manager::{ObjectType, VmData};
use vm::state;

fn runtime_err(msg: impl Into<std::borrow::Cow<'static, str>>) -> Box<NovaError> {
    Box::new(NovaError::Runtime { msg: msg.into() })
}

fn pop(state: &mut state::State) -> NovaResult<VmData> {
    state
        .memory
        .stack
        .pop()
        .ok_or(runtime_err("Stack is empty"))
}

fn pop_string(state: &mut state::State) -> NovaResult<String> {
    match pop(state)? {
        VmData::Object(index) => {
            let s = state
                .memory
                .ref_from_heap(index)
                .and_then(|o| o.as_string())
                .ok_or(runtime_err("Expected a string object"))?;
            state.memory.dec(index);
            Ok(s)
        }
        _ => Err(runtime_err("Expected a string on the stack")),
    }
}

/// readln() -> String
pub fn read_line(state: &mut state::State) -> NovaResult<()> {
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|e| {
        Box::new(NovaError::Runtime {
            msg: format!("Error reading line: {e}").into(),
        })
    })?;
    // remove trailing newline
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }
    state.memory.push_string(input);
    Ok(())
}

/// readFile(path: String) -> Option(String)
pub fn read_file(state: &mut state::State) -> NovaResult<()> {
    let path = pop_string(state)?;
    match fs::read_to_string(&path) {
        Ok(string) => {
            state.memory.push_string(string);
            Ok(())
        }
        Err(_) => {
            state.memory.stack.push(VmData::None);
            Ok(())
        }
    }
}

/// writeFile(path: String, content: String) -> Bool
pub fn write_file(state: &mut state::State) -> NovaResult<()> {
    let content = pop_string(state)?;
    let path = pop_string(state)?;
    match fs::write(&path, &content) {
        Ok(()) => {
            state.memory.stack.push(VmData::Bool(true));
            Ok(())
        }
        Err(_) => {
            state.memory.stack.push(VmData::Bool(false));
            Ok(())
        }
    }
}

/// fileExists(path: String) -> Bool
pub fn file_exists(state: &mut state::State) -> NovaResult<()> {
    let path = pop_string(state)?;
    state
        .memory
        .stack
        .push(VmData::Bool(std::path::Path::new(&path).exists()));
    Ok(())
}

/// appendFile(path: String, content: String) -> Bool
pub fn append_file(state: &mut state::State) -> NovaResult<()> {
    use std::io::Write;
    let content = pop_string(state)?;
    let path = pop_string(state)?;
    let result = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .and_then(|mut file| file.write_all(content.as_bytes()));
    match result {
        Ok(()) => {
            state.memory.stack.push(VmData::Bool(true));
        }
        Err(_) => {
            state.memory.stack.push(VmData::Bool(false));
        }
    }
    Ok(())
}

/// tempDir() -> String
/// Returns the OS temporary directory (e.g. "/tmp" on Unix, "C:\Users\...\AppData\Local\Temp" on Windows).
/// The returned path always ends with a path separator so callers can append a filename directly.
pub fn temp_dir(state: &mut state::State) -> NovaResult<()> {
    let mut tmp = std::env::temp_dir()
        .to_string_lossy()
        .to_string();
    // Ensure it ends with a separator so `tempDir() + "file.txt"` just works.
    if !tmp.ends_with(std::path::MAIN_SEPARATOR) {
        tmp.push(std::path::MAIN_SEPARATOR);
    }
    state.memory.push_string(tmp);
    Ok(())
}

// ---------------------------------------------------------------------------
// printf / format helpers
// ---------------------------------------------------------------------------

fn printf_with_array(format_string: &str, args: Vec<String>) {
    print!("{}", format_with_array(format_string, args));
}

fn format_with_array(format_string: &str, args: Vec<String>) -> String {
    let mut arg_iter = args.iter();
    let mut formatted = String::new();
    let chars: Vec<char> = format_string.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        if chars[i] == '{' {
            if i + 1 < len && chars[i + 1] == '{' {
                // Escaped opening brace: {{ → {
                formatted.push('{');
                i += 2;
            } else if i + 1 < len && chars[i + 1] == '}' {
                // Placeholder: {} → substitute next arg
                if let Some(arg) = arg_iter.next() {
                    formatted.push_str(arg);
                } else {
                    formatted.push_str("{}");
                }
                i += 2;
            } else {
                formatted.push('{');
                i += 1;
            }
        } else if chars[i] == '}' {
            if i + 1 < len && chars[i + 1] == '}' {
                // Escaped closing brace: }} → }
                formatted.push('}');
                i += 2;
            } else {
                formatted.push('}');
                i += 1;
            }
        } else {
            formatted.push(chars[i]);
            i += 1;
        }
    }
    formatted
}

fn extract_string_list(state: &state::State, list_index: usize) -> NovaResult<Vec<String>> {
    let list_obj = state
        .memory
        .ref_from_heap(list_index)
        .ok_or(runtime_err("Invalid heap reference for list"))?;
    match &list_obj.object_type {
        ObjectType::List => {
            let mut strings = vec![];
            for item in &list_obj.data {
                match item {
                    VmData::Object(str_idx) => {
                        let str_obj = state
                            .memory
                            .ref_from_heap(*str_idx)
                            .ok_or(runtime_err("Invalid arguments for printf"))?;
                        let s = str_obj
                            .as_string()
                            .ok_or(runtime_err("Invalid arguments for printf"))?;
                        strings.push(s);
                    }
                    _ => return Err(runtime_err("Invalid arguments for printf")),
                }
            }
            Ok(strings)
        }
        _ => Err(runtime_err("Invalid arguments for printf")),
    }
}

/// printf(fmt: String, args: [String])
pub fn printf(state: &mut state::State) -> NovaResult<()> {
    let args = pop(state)?;
    let str_val = pop(state)?;
    if let (VmData::Object(args_idx), VmData::Object(str_idx)) = (args, str_val) {
        let format_string = {
            let obj = state
                .memory
                .ref_from_heap(str_idx)
                .ok_or(runtime_err("Invalid arguments for printf"))?;
            obj.as_string()
                .ok_or(runtime_err("Invalid arguments for printf"))?
        };
        let strings = extract_string_list(state, args_idx)?;
        printf_with_array(&format_string, strings);
        state.memory.dec(args_idx);
        state.memory.dec(str_idx);
    } else {
        return Err(runtime_err("Invalid arguments for printf"));
    }
    Ok(())
}

/// format(fmt: String, args: [String]) -> String
pub fn format(state: &mut state::State) -> NovaResult<()> {
    let args = pop(state)?;
    let str_val = pop(state)?;
    if let (VmData::Object(args_idx), VmData::Object(str_idx)) = (args, str_val) {
        let format_string = {
            let obj = state
                .memory
                .ref_from_heap(str_idx)
                .ok_or(runtime_err("Invalid arguments for format"))?;
            obj.as_string()
                .ok_or(runtime_err("Invalid arguments for format"))?
        };
        let strings = extract_string_list(state, args_idx)?;
        let result = format_with_array(&format_string, strings);
        state.memory.dec(args_idx);
        state.memory.dec(str_idx);
        state.memory.push_string(result);
    } else {
        return Err(runtime_err("Invalid arguments for format"));
    }
    Ok(())
}
