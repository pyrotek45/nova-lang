use common::error::{NovaError, NovaResult};
use vm::memory_manager::{Object, ObjectType, VmData};
use vm::state;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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

fn pop_int(state: &mut state::State) -> NovaResult<i64> {
    match pop(state)? {
        VmData::Int(v) => Ok(v),
        _ => Err(runtime_err("Expected an Int on the stack")),
    }
}

fn pop_char(state: &mut state::State) -> NovaResult<char> {
    match pop(state)? {
        VmData::Char(c) => Ok(c),
        _ => Err(runtime_err("Expected a Char on the stack")),
    }
}

// ---------------------------------------------------------------------------
// Core conversion / inspection
// ---------------------------------------------------------------------------

/// String::len(s) -> Int
pub fn strlen(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.stack.push(VmData::Int(s.len() as i64));
    Ok(())
}

/// String::chars(s) -> [Char]
pub fn str_to_chars(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    let chars: Vec<VmData> = s.chars().map(VmData::Char).collect();
    state.memory.push_list(chars);
    Ok(())
}

/// List::string(chars: [Char]) -> String
pub fn chars_to_str(state: &mut state::State) -> NovaResult<()> {
    let data = pop(state)?;
    let index = match data {
        VmData::Object(i) => i,
        _ => return Err(runtime_err("Expected a list on the stack")),
    };
    let result = {
        let obj = state
            .memory
            .ref_from_heap(index)
            .ok_or(runtime_err("Invalid heap reference"))?;
        match &obj.object_type {
            ObjectType::List => {
                let mut s = String::new();
                for item in &obj.data {
                    match item {
                        VmData::Char(c) => s.push(*c),
                        _ => return Err(runtime_err("Expected a Char in the list")),
                    }
                }
                s
            }
            _ => return Err(runtime_err("Expected a list object")),
        }
    };
    state.memory.dec(index);
    state.memory.push_string(result);
    Ok(())
}

/// Cast::string(v) -> String
pub fn to_string(state: &mut state::State) -> NovaResult<()> {
    let data = pop(state)?;
    let string = match data {
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

/// Cast::int(v) -> Option(Int)
pub fn to_int(state: &mut state::State) -> NovaResult<()> {
    let data = pop(state)?;
    let int = match data {
        VmData::Int(v) => v,
        VmData::Float(v) => v as i64,
        VmData::Bool(v) => i64::from(v),
        VmData::Char(v) => match v.to_string().parse::<i64>() {
            Ok(n) => n,
            Err(_) => {
                state.memory.stack.push(VmData::None);
                return Ok(());
            }
        },
        VmData::Object(v) => {
            if let Some(obj) = state.memory.ref_from_heap(v) {
                if let Some(s) = obj.as_string() {
                    state.memory.dec(v);
                    match s.parse::<i64>() {
                        Ok(n) => n,
                        Err(_) => {
                            state.memory.stack.push(VmData::None);
                            return Ok(());
                        }
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

// ---------------------------------------------------------------------------
// String manipulation
// ---------------------------------------------------------------------------

/// String::contains(s, sub) -> Bool
pub fn str_contains(state: &mut state::State) -> NovaResult<()> {
    let sub = pop_string(state)?;
    let s = pop_string(state)?;
    state.memory.stack.push(VmData::Bool(s.contains(&sub)));
    Ok(())
}

/// String::startsWith(s, prefix) -> Bool
pub fn str_starts_with(state: &mut state::State) -> NovaResult<()> {
    let prefix = pop_string(state)?;
    let s = pop_string(state)?;
    state
        .memory
        .stack
        .push(VmData::Bool(s.starts_with(&prefix)));
    Ok(())
}

/// String::endsWith(s, suffix) -> Bool
pub fn str_ends_with(state: &mut state::State) -> NovaResult<()> {
    let suffix = pop_string(state)?;
    let s = pop_string(state)?;
    state.memory.stack.push(VmData::Bool(s.ends_with(&suffix)));
    Ok(())
}

/// String::trim(s) -> String
pub fn str_trim(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.push_string(s.trim().to_string());
    Ok(())
}

/// String::trimStart(s) -> String
pub fn str_trim_start(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.push_string(s.trim_start().to_string());
    Ok(())
}

/// String::trimEnd(s) -> String
pub fn str_trim_end(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.push_string(s.trim_end().to_string());
    Ok(())
}

/// String::toUpper(s) -> String
pub fn str_to_upper(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.push_string(s.to_uppercase());
    Ok(())
}

/// String::toLower(s) -> String
pub fn str_to_lower(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.push_string(s.to_lowercase());
    Ok(())
}

/// String::replace(s, from, to) -> String
pub fn str_replace(state: &mut state::State) -> NovaResult<()> {
    let to = pop_string(state)?;
    let from = pop_string(state)?;
    let s = pop_string(state)?;
    state.memory.push_string(s.replace(&from, &to));
    Ok(())
}

/// String::substring(s, start, end) -> String
pub fn str_substring(state: &mut state::State) -> NovaResult<()> {
    let end = pop_int(state)? as usize;
    let start = pop_int(state)? as usize;
    let s = pop_string(state)?;
    let chars: Vec<char> = s.chars().collect();
    let end = end.min(chars.len());
    let start = start.min(end);
    let sub: String = chars[start..end].iter().collect();
    state.memory.push_string(sub);
    Ok(())
}

/// String::indexOf(s, sub) -> Int  (-1 if not found)
pub fn str_index_of(state: &mut state::State) -> NovaResult<()> {
    let sub = pop_string(state)?;
    let s = pop_string(state)?;
    let idx = s.find(&sub).map(|i| i as i64).unwrap_or(-1);
    state.memory.stack.push(VmData::Int(idx));
    Ok(())
}

/// String::repeat(s, n) -> String
pub fn str_repeat(state: &mut state::State) -> NovaResult<()> {
    let n = pop_int(state)? as usize;
    let s = pop_string(state)?;
    state.memory.push_string(s.repeat(n));
    Ok(())
}

/// String::reverse(s) -> String
pub fn str_reverse(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.push_string(s.chars().rev().collect());
    Ok(())
}

/// String::isEmpty(s) -> Bool
pub fn str_is_empty(state: &mut state::State) -> NovaResult<()> {
    let s = pop_string(state)?;
    state.memory.stack.push(VmData::Bool(s.is_empty()));
    Ok(())
}

/// String::charAt(s, index) -> Option(Char)
pub fn str_char_at(state: &mut state::State) -> NovaResult<()> {
    let idx = pop_int(state)? as usize;
    let s = pop_string(state)?;
    match s.chars().nth(idx) {
        Some(c) => state.memory.stack.push(VmData::Char(c)),
        None => state.memory.stack.push(VmData::None),
    }
    Ok(())
}

/// String::split(s, delim) -> [String]
pub fn str_split(state: &mut state::State) -> NovaResult<()> {
    let delim = pop_string(state)?;
    let s = pop_string(state)?;
    let parts: Vec<VmData> = s
        .split(&delim)
        .map(|p| {
            let idx = state.memory.allocate(Object::string(p.to_string()));
            VmData::Object(idx)
        })
        .collect();
    state.memory.push_list(parts);
    Ok(())
}

/// String::join(parts: [String], delim) -> String
pub fn str_join(state: &mut state::State) -> NovaResult<()> {
    let delim = pop_string(state)?;
    let list_data = pop(state)?;
    let list_index = match list_data {
        VmData::Object(i) => i,
        _ => return Err(runtime_err("Expected a list on the stack")),
    };
    let result = {
        let obj = state
            .memory
            .ref_from_heap(list_index)
            .ok_or(runtime_err("Invalid heap reference"))?;
        let mut parts = Vec::new();
        for item in &obj.data {
            match item {
                VmData::Object(str_idx) => {
                    let s = state
                        .memory
                        .ref_from_heap(*str_idx)
                        .and_then(|o| o.as_string())
                        .ok_or(runtime_err("Expected a string in the list"))?;
                    parts.push(s);
                }
                _ => return Err(runtime_err("Expected string elements in the list")),
            }
        }
        parts.join(&delim)
    };
    state.memory.dec(list_index);
    state.memory.push_string(result);
    Ok(())
}

/// String::charToInt(c: Char) -> Int
pub fn char_to_int(state: &mut state::State) -> NovaResult<()> {
    let c = pop_char(state)?;
    state.memory.stack.push(VmData::Int(c as i64));
    Ok(())
}
