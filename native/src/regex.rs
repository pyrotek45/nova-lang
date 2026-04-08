use common::error::{NovaError, NovaResult};
use vm::memory_manager::{Object, VmData};
use vm::state;

/// Helper to pop a string Object from the stack and return the String value.
/// Decrements the reference count of the Object.
fn pop_string(state: &mut state::State) -> NovaResult<String> {
    match state.memory.stack.pop() {
        Some(VmData::Object(index)) => {
            let s = state
                .memory
                .ref_from_heap(index)
                .and_then(|obj| obj.as_string())
                .ok_or(Box::new(NovaError::Runtime {
                    msg: "Expected a string in the heap".into(),
                }))?;
            state.memory.dec(index);
            Ok(s)
        }
        Some(_) => Err(Box::new(NovaError::Runtime {
            msg: "Expected a string on the stack".into(),
        })),
        None => Err(Box::new(NovaError::Runtime {
            msg: "Stack is empty".into(),
        })),
    }
}

pub fn regex_match(state: &mut state::State) -> NovaResult<()> {
    let text = pop_string(state)?;
    let pattern = pop_string(state)?;

    let re = match regex::Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => {
            // Invalid regex pattern — return false instead of crashing
            state.memory.stack.push(VmData::Bool(false));
            return Ok(());
        }
    };

    let result = re.is_match(&text);
    state.memory.stack.push(VmData::Bool(result));
    Ok(())
}

pub fn regex_captures(state: &mut state::State) -> NovaResult<()> {
    let text = pop_string(state)?;
    let pattern = pop_string(state)?;

    let re = match regex::Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => {
            // Invalid regex pattern — return empty list instead of crashing
            state.memory.push_list(vec![]);
            return Ok(());
        }
    };

    let captures: Vec<String> = re
        .find_iter(&text)
        .map(|m| m.as_str().to_string())
        .collect();

    let mut list_data = vec![];
    for capture in captures {
        let str_idx = state.memory.allocate(Object::string(capture));
        list_data.push(VmData::Object(str_idx));
    }

    state.memory.push_list(list_data);
    Ok(())
}

pub fn regex_first(state: &mut state::State) -> NovaResult<()> {
    let text = pop_string(state)?;
    let pattern = pop_string(state)?;

    let re = match regex::Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => {
            // Invalid regex pattern — return None instead of crashing
            state.memory.stack.push(VmData::None);
            return Ok(());
        }
    };

    let Some(captures) = re.find(&text) else {
        state.memory.stack.push(VmData::None);
        return Ok(());
    };
    let start = captures.start() as i64;
    let end = captures.end() as i64;
    let matched_str = captures.as_str().to_string();

    // Create a tuple (start, end, string) as a list [Int, Int, Object(String)]
    let str_idx = state.memory.allocate(Object::string(matched_str));
    let list_data = vec![
        VmData::Int(start),
        VmData::Int(end),
        VmData::Object(str_idx),
    ];
    state.memory.push_list(list_data);
    Ok(())
}

/// Regex::split(pattern: String, text: String) -> [String]
/// Split `text` by all non-overlapping matches of `pattern`.
/// Returns the original text in a single-element list on invalid regex.
pub fn regex_split(state: &mut state::State) -> NovaResult<()> {
    let text = pop_string(state)?;
    let pattern = pop_string(state)?;

    let re = match regex::Regex::new(&pattern) {
        Ok(re) => re,
        Err(_) => {
            // Invalid regex pattern — return [text] unchanged
            let idx = state.memory.allocate(Object::string(text));
            state.memory.push_list(vec![VmData::Object(idx)]);
            return Ok(());
        }
    };

    let parts: Vec<VmData> = re
        .split(&text)
        .map(|p| {
            let idx = state.memory.allocate(Object::string(p.to_string()));
            VmData::Object(idx)
        })
        .collect();
    state.memory.push_list(parts);
    Ok(())
}
