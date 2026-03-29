use common::error::NovaResult;
use vm::memory_manager::{ObjectType, VmData};
use vm::state;

pub fn len(state: &mut state::State) -> NovaResult<()> {
    if let Some(VmData::Object(index)) = state.memory.stack.pop() {
        if let Some(obj) = state.memory.ref_from_heap(index) {
            if let ObjectType::List = obj.object_type {
                let len = obj.data.len() as i64;
                state.memory.dec(index);
                state.memory.stack.push(VmData::Int(len));
            }
        }
    }
    Ok(())
}

pub fn push(state: &mut state::State) -> NovaResult<()> {
    if let (Some(data), Some(VmData::Object(index))) =
        (state.memory.stack.pop(), state.memory.stack.pop())
    {
        // inc data if it's an Object since we're adding a new reference
        state.memory.inc_value(data);
        if let Some(obj) = state.memory.ref_from_heap_mut(index) {
            if let ObjectType::List = obj.object_type {
                obj.data.push(data);
            }
        }
        // dec the popped data and list ref
        state.memory.dec_value(data);
        state.memory.dec(index);
    } else {
        panic!("List::push: expected data and list on stack")
    }
    Ok(())
}

pub fn pop(state: &mut state::State) -> NovaResult<()> {
    if let Some(VmData::Object(index)) = state.memory.stack.pop() {
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
            Some(value) => {
                // The value was owned by the list, now transfer ownership to the stack
                state.memory.stack.push(value);
            }
            None => {
                state.memory.stack.push(VmData::None);
            }
        }
    } else {
        panic!("List::pop: expected list on stack")
    }
    Ok(())
}

pub fn remove(state: &mut state::State) -> NovaResult<()> {
    if let (Some(VmData::Int(idx)), Some(VmData::Object(list_index))) =
        (state.memory.stack.pop(), state.memory.stack.pop())
    {
        let removed = {
            if let Some(obj) = state.memory.ref_from_heap_mut(list_index) {
                if let ObjectType::List = obj.object_type {
                    Some(obj.data.remove(idx as usize))
                } else {
                    None
                }
            } else {
                None
            }
        };
        // dec the removed element if it was an Object
        if let Some(removed_val) = removed {
            state.memory.dec_value(removed_val);
        }
        state.memory.dec(list_index);
    } else {
        panic!("List::remove: expected int and list on stack")
    }
    Ok(())
}
