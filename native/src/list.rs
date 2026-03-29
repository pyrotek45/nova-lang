use common::error::{NovaError, NovaResult};
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

fn pop_int(state: &mut state::State) -> NovaResult<i64> {
    match pop(state)? {
        VmData::Int(v) => Ok(v),
        _ => Err(runtime_err("Expected an Int on the stack")),
    }
}

fn pop_list(state: &mut state::State) -> NovaResult<usize> {
    match pop(state)? {
        VmData::Object(index) => Ok(index),
        _ => Err(runtime_err("Expected a list on the stack")),
    }
}

/// List::len(list) -> Int
pub fn len(state: &mut state::State) -> NovaResult<()> {
    let index = pop_list(state)?;
    let length = state
        .memory
        .ref_from_heap(index)
        .and_then(|obj| match obj.object_type {
            ObjectType::List => Some(obj.data.len() as i64),
            _ => None,
        })
        .ok_or(runtime_err("Expected a list object"))?;
    state.memory.dec(index);
    state.memory.stack.push(VmData::Int(length));
    Ok(())
}

/// List::push(list, item)
pub fn push(state: &mut state::State) -> NovaResult<()> {
    let data = pop(state)?;
    let index = pop_list(state)?;
    state.memory.inc_value(data);
    if let Some(obj) = state.memory.ref_from_heap_mut(index) {
        if let ObjectType::List = obj.object_type {
            obj.data.push(data);
        } else {
            return Err(runtime_err("Expected a list object"));
        }
    } else {
        return Err(runtime_err("Invalid heap reference"));
    }
    state.memory.dec_value(data);
    state.memory.dec(index);
    Ok(())
}

/// List::pop(list) -> Option(T)
pub fn pop_item(state: &mut state::State) -> NovaResult<()> {
    let index = pop_list(state)?;
    let popped = {
        if let Some(obj) = state.memory.ref_from_heap_mut(index) {
            if let ObjectType::List = obj.object_type {
                obj.data.pop()
            } else {
                None
            }
        } else {
            None
        }
    };
    state.memory.dec(index);
    match popped {
        Some(value) => state.memory.stack.push(value),
        None => state.memory.stack.push(VmData::None),
    }
    Ok(())
}

/// List::remove(list, index)
pub fn remove(state: &mut state::State) -> NovaResult<()> {
    let idx = pop_int(state)? as usize;
    let list_index = pop_list(state)?;
    let removed = {
        if let Some(obj) = state.memory.ref_from_heap_mut(list_index) {
            if let ObjectType::List = obj.object_type {
                if idx < obj.data.len() {
                    Some(obj.data.remove(idx))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    };
    if let Some(val) = removed {
        state.memory.dec_value(val);
    }
    state.memory.dec(list_index);
    Ok(())
}

/// List::insert(list, index, item)
pub fn insert(state: &mut state::State) -> NovaResult<()> {
    let data = pop(state)?;
    let idx = pop_int(state)? as usize;
    let list_index = pop_list(state)?;
    state.memory.inc_value(data);
    if let Some(obj) = state.memory.ref_from_heap_mut(list_index) {
        if let ObjectType::List = obj.object_type {
            let idx = idx.min(obj.data.len());
            obj.data.insert(idx, data);
        } else {
            return Err(runtime_err("Expected a list object"));
        }
    } else {
        return Err(runtime_err("Invalid heap reference"));
    }
    state.memory.dec_value(data);
    state.memory.dec(list_index);
    Ok(())
}

/// List::swap(list, i, j)
pub fn swap(state: &mut state::State) -> NovaResult<()> {
    let j = pop_int(state)? as usize;
    let i = pop_int(state)? as usize;
    let list_index = pop_list(state)?;
    if let Some(obj) = state.memory.ref_from_heap_mut(list_index) {
        if let ObjectType::List = obj.object_type {
            if i < obj.data.len() && j < obj.data.len() {
                obj.data.swap(i, j);
            } else {
                return Err(runtime_err("List::swap: index out of bounds"));
            }
        }
    }
    state.memory.dec(list_index);
    Ok(())
}

/// List::clear(list)
pub fn clear(state: &mut state::State) -> NovaResult<()> {
    let list_index = pop_list(state)?;
    let items: Vec<VmData> = {
        if let Some(obj) = state.memory.ref_from_heap_mut(list_index) {
            if let ObjectType::List = obj.object_type {
                obj.data.drain(..).collect()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    };
    for item in items {
        state.memory.dec_value(item);
    }
    state.memory.dec(list_index);
    Ok(())
}

/// List::set(list, index, value)
pub fn set(state: &mut state::State) -> NovaResult<()> {
    let value = pop(state)?;
    let idx = pop_int(state)? as usize;
    let list_index = pop_list(state)?;
    state.memory.inc_value(value);
    if let Some(obj) = state.memory.ref_from_heap_mut(list_index) {
        if let ObjectType::List = obj.object_type {
            if idx < obj.data.len() {
                let old = obj.data[idx];
                obj.data[idx] = value;
                state.memory.dec_value(old);
            } else {
                return Err(runtime_err("List::set: index out of bounds"));
            }
        }
    }
    state.memory.dec_value(value);
    state.memory.dec(list_index);
    Ok(())
}
