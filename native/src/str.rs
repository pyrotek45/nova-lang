use common::error::NovaError;
use vm::state::{self, Heap, VmData};

pub fn strlen(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::String(index)) = state.stack.pop() {
        if let Heap::String(str) = state.deref(index) {
            state.stack.push(VmData::Int(str.len() as i64))
        }
    }
    Ok(())
}

pub fn str_to_chars(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::String(index)) = state.stack.pop() {
        if let Heap::String(str) = state.deref(index) {
            state.gclock = true;
            let mut myarray = vec![];
            for c in str.chars() {
                myarray.push(state.allocate_vmdata_to_heap(VmData::Char(c)))
            }
            let index = state.allocate_array(myarray);
            state.stack.push(VmData::List(index));
            state.gclock = false;
        }
    }
    Ok(())
}

pub fn chars_to_str(state: &mut state::State) -> Result<(), NovaError> {
    if let Some(VmData::List(index)) = state.stack.pop() {
        if let Heap::List(array) = state.deref(index) {
            state.gclock = true;
            let mut str = String::new();
            for item in array.iter() {
                let char = state.deref(*item);
                if let Heap::Char(c) = char {
                    str.push(c)
                }
            }
            let index = state.allocate_string(str);
            state.stack.push(VmData::String(index));
            state.gclock = false;
        }
    }
    Ok(())
}

pub fn to_string(state: &mut state::State) -> Result<(), NovaError> {
    let data = state.stack.pop().unwrap();
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
                sbuild += &format!("[");
                for (index, item) in array.iter().enumerate() {
                    if index > 0 {
                        sbuild += &format!(", ");
                    }
                    sbuild += &format!("{:?}", state.deref(*item));
                }
                sbuild += &format!("]");
            };
            sbuild
        }
        VmData::Struct(v) => format!("Struct pointer: {v}"),
        VmData::String(v) => {
            let out = if let Heap::String(str) = state.deref(v) {
                format!("{str}")
            } else {
                todo!()
            };
            out
        }
        VmData::None => "None".to_string(),
    };
    let index = state.allocate_string(string);
    state.stack.push(VmData::String(index));
    Ok(())
}
