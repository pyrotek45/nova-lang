use common::error::NovaError;
use vm::state::{self, Heap, VmData};

pub fn len(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::List(index)) = state.stack.pop() {
        if let Heap::List(array) = state.deref(index) {
            state.stack.push(VmData::Int(array.len() as i64))
        }
    }
    Ok(())
}

pub fn push(state: &mut state::State) -> Result<(), NovaError> {
    if let (Some(data), Some(VmData::List(index))) = (state.stack.pop(), state.stack.pop()) {
        if let Heap::List(mut array) = state.deref(index) {
            array.push(state.allocate_vmdata_to_heap(data));
            state.heap[index] = Heap::List(array);
        } else {
            panic!()
        }
    } else {
        panic!()
    }
    Ok(())
}

pub fn pop(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::List(index)) = state.stack.pop() {
        if let Heap::List(mut array) = state.deref(index) {
            if let Some(item) = array.pop() {
                state.stack.push(state.to_vmdata(item));
            } else {
                state.stack.push(VmData::None);
            }
            state.heap[index] = Heap::List(array);
        } else {
            panic!()
        }
    } else {
        panic!()
    }
    Ok(())
}
