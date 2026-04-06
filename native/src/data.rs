use std::collections::HashMap;

use common::error::{NovaError, NovaResult};
use serde_json::{json, Map, Value};
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

// ---------------------------------------------------------------------------
// Serialize: VmData → serde_json::Value
// ---------------------------------------------------------------------------

/// Recursively convert a VmData value into a JSON Value with embedded type tags.
///
/// Every value is represented as a JSON object with a `"_type"` field so that
/// `deserialize_value` can reconstruct the exact same heap structure later.
fn serialize_value(
    memory: &vm::memory_manager::MemoryManager,
    value: &VmData,
    depth: usize,
) -> Result<Value, String> {
    if depth > 128 {
        return Err("Data::save: nesting depth exceeded (possible cycle)".into());
    }

    match value {
        VmData::Int(v) => Ok(json!({ "_type": "Int", "value": *v })),
        VmData::Float(v) => {
            if v.is_nan() {
                Ok(json!({ "_type": "Float", "value": "NaN" }))
            } else if v.is_infinite() {
                if *v > 0.0 {
                    Ok(json!({ "_type": "Float", "value": "Inf" }))
                } else {
                    Ok(json!({ "_type": "Float", "value": "-Inf" }))
                }
            } else {
                Ok(json!({ "_type": "Float", "value": *v }))
            }
        }
        VmData::Bool(v) => Ok(json!({ "_type": "Bool", "value": *v })),
        VmData::Char(v) => Ok(json!({ "_type": "Char", "value": v.to_string() })),
        VmData::None => Ok(json!({ "_type": "None" })),
        VmData::Function(_) => Err("Data::save: cannot serialize Function values".into()),
        VmData::StackAddress(_) => {
            Err("Data::save: cannot serialize StackAddress values".into())
        }
        VmData::Object(idx) => {
            let obj = memory
                .ref_from_heap(*idx)
                .ok_or_else(|| "Data::save: invalid heap reference".to_string())?;

            match &obj.object_type {
                ObjectType::String => {
                    let s: String = obj
                        .data
                        .iter()
                        .filter_map(|v| if let VmData::Char(c) = v { Some(*c) } else { None })
                        .collect();
                    Ok(json!({ "_type": "String", "value": s }))
                }
                ObjectType::List => {
                    let elements: Result<Vec<Value>, String> = obj
                        .data
                        .iter()
                        .map(|v| serialize_value(memory, v, depth + 1))
                        .collect();
                    Ok(json!({ "_type": "List", "elements": elements? }))
                }
                ObjectType::Tuple => {
                    let elements: Result<Vec<Value>, String> = obj
                        .data
                        .iter()
                        .map(|v| serialize_value(memory, v, depth + 1))
                        .collect();
                    Ok(json!({ "_type": "Tuple", "elements": elements? }))
                }
                ObjectType::Struct(name) => {
                    // Serialize fields in sorted order for deterministic output.
                    // Skip the runtime "type" field – it is added automatically by the VM.
                    // Include the original index so deserialization restores exact field order.
                    let mut fields = Map::new();
                    let mut sorted_keys: Vec<_> = obj.table.iter()
                        .filter(|(k, _)| k.as_str() != "type")
                        .collect();
                    sorted_keys.sort_by_key(|(k, _)| (*k).clone());
                    for (key, &idx) in &sorted_keys {
                        if idx < obj.data.len() {
                            let val = serialize_value(memory, &obj.data[idx], depth + 1)?;
                            fields.insert((*key).clone(), json!({ "index": idx, "value": val }));
                        }
                    }
                    Ok(json!({
                        "_type": "Struct",
                        "name": name.as_str(),
                        "fields": Value::Object(fields)
                    }))
                }
                ObjectType::Enum { name, tag } => {
                    let data: Result<Vec<Value>, String> = obj
                        .data
                        .iter()
                        .map(|v| serialize_value(memory, v, depth + 1))
                        .collect();
                    Ok(json!({
                        "_type": "Enum",
                        "name": name.as_str(),
                        "tag": *tag,
                        "data": data?
                    }))
                }
                ObjectType::Closure(_) => {
                    Err("Data::save: cannot serialize Closure values".into())
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Deserialize: serde_json::Value → VmData  (allocates on the VM heap)
// ---------------------------------------------------------------------------

/// Recursively reconstruct a VmData value from a JSON Value.
///
/// Heap objects (strings, lists, structs, etc.) are allocated via
/// `memory.allocate()` so the GC tracks them properly.
fn deserialize_value(
    memory: &mut vm::memory_manager::MemoryManager,
    val: &Value,
    depth: usize,
) -> Result<VmData, String> {
    if depth > 128 {
        return Err("Data::load: nesting depth exceeded".into());
    }

    let obj = val.as_object().ok_or("Data::load: expected JSON object")?;
    let type_tag = obj
        .get("_type")
        .and_then(|v| v.as_str())
        .ok_or("Data::load: missing _type field")?;

    match type_tag {
        "Int" => {
            let v = obj
                .get("value")
                .and_then(|v| v.as_i64())
                .ok_or("Data::load: Int missing 'value'")?;
            Ok(VmData::Int(v))
        }
        "Float" => {
            let raw = obj.get("value").ok_or("Data::load: Float missing 'value'")?;
            let v = if let Some(s) = raw.as_str() {
                match s {
                    "NaN" => f64::NAN,
                    "Inf" => f64::INFINITY,
                    "-Inf" => f64::NEG_INFINITY,
                    _ => return Err(format!("Data::load: unknown Float string '{}'", s)),
                }
            } else {
                raw.as_f64()
                    .ok_or("Data::load: Float 'value' is not a number")?
            };
            Ok(VmData::Float(v))
        }
        "Bool" => {
            let v = obj
                .get("value")
                .and_then(|v| v.as_bool())
                .ok_or("Data::load: Bool missing 'value'")?;
            Ok(VmData::Bool(v))
        }
        "Char" => {
            let s = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or("Data::load: Char missing 'value'")?;
            let c = s
                .chars()
                .next()
                .ok_or("Data::load: Char 'value' is empty")?;
            Ok(VmData::Char(c))
        }
        "None" => Ok(VmData::None),
        "String" => {
            let s = obj
                .get("value")
                .and_then(|v| v.as_str())
                .ok_or("Data::load: String missing 'value'")?;
            let string_obj = Object::string(s.to_string());
            let idx = memory.allocate(string_obj);
            Ok(VmData::Object(idx))
        }
        "List" => {
            let elements = obj
                .get("elements")
                .and_then(|v| v.as_array())
                .ok_or("Data::load: List missing 'elements'")?;
            // Inhibit GC while building the list so intermediate objects aren't collected
            memory.gc_inhibit();
            let mut data = Vec::with_capacity(elements.len());
            for elem in elements {
                match deserialize_value(memory, elem, depth + 1) {
                    Ok(v) => data.push(v),
                    Err(e) => {
                        memory.gc_release();
                        return Err(e);
                    }
                }
            }
            let list_obj = Object::new(ObjectType::List, data);
            let idx = memory.allocate(list_obj);
            memory.gc_release();
            Ok(VmData::Object(idx))
        }
        "Tuple" => {
            let elements = obj
                .get("elements")
                .and_then(|v| v.as_array())
                .ok_or("Data::load: Tuple missing 'elements'")?;
            memory.gc_inhibit();
            let mut data = Vec::with_capacity(elements.len());
            for elem in elements {
                match deserialize_value(memory, elem, depth + 1) {
                    Ok(v) => data.push(v),
                    Err(e) => {
                        memory.gc_release();
                        return Err(e);
                    }
                }
            }
            let tuple_obj = Object::tuple(data);
            let idx = memory.allocate(tuple_obj);
            memory.gc_release();
            Ok(VmData::Object(idx))
        }
        "Struct" => {
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Data::load: Struct missing 'name'")?;
            let fields = obj
                .get("fields")
                .and_then(|v| v.as_object())
                .ok_or("Data::load: Struct missing 'fields'")?;

            memory.gc_inhibit();

            // Reconstruct the struct preserving original field indices.
            // Each field is stored as { "index": N, "value": {...} }.
            let field_count = fields.len();
            let mut data: Vec<VmData> = vec![VmData::None; field_count];
            let mut table: HashMap<String, usize> = HashMap::new();
            for (key, wrapper) in fields {
                let wrapper_obj = wrapper.as_object()
                    .ok_or("Data::load: Struct field entry must be an object")?;
                let idx = wrapper_obj.get("index")
                    .and_then(|v| v.as_u64())
                    .ok_or("Data::load: Struct field missing 'index'")? as usize;
                let val_json = wrapper_obj.get("value")
                    .ok_or("Data::load: Struct field missing 'value'")?;
                match deserialize_value(memory, val_json, depth + 1) {
                    Ok(v) => {
                        if idx < field_count {
                            data[idx] = v;
                        }
                        table.insert(key.clone(), idx);
                    }
                    Err(e) => {
                        memory.gc_release();
                        return Err(e);
                    }
                }
            }

            let mut struct_obj = Object::new(ObjectType::Struct(name.to_string()), data);
            struct_obj.table = table;

            // Add the runtime "type" field that the VM attaches to every struct
            let type_str_obj = Object::string(name.to_string());
            let type_str_idx = memory.allocate(type_str_obj);
            let type_field_index = struct_obj.data.len();
            struct_obj.data.push(VmData::Object(type_str_idx));
            struct_obj.table.insert("type".to_string(), type_field_index);

            let idx = memory.allocate(struct_obj);
            memory.gc_release();
            Ok(VmData::Object(idx))
        }
        "Enum" => {
            let name = obj
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Data::load: Enum missing 'name'")?;
            let tag = obj
                .get("tag")
                .and_then(|v| v.as_i64())
                .ok_or("Data::load: Enum missing 'tag'")?;
            let data_arr = obj
                .get("data")
                .and_then(|v| v.as_array())
                .ok_or("Data::load: Enum missing 'data'")?;

            memory.gc_inhibit();
            let mut data = Vec::with_capacity(data_arr.len());
            for elem in data_arr {
                match deserialize_value(memory, elem, depth + 1) {
                    Ok(v) => data.push(v),
                    Err(e) => {
                        memory.gc_release();
                        return Err(e);
                    }
                }
            }
            let enum_obj = Object::enum_object(name.to_string(), tag, data);
            let idx = memory.allocate(enum_obj);
            memory.gc_release();
            Ok(VmData::Object(idx))
        }
        other => Err(format!("Data::load: unknown type tag '{}'", other)),
    }
}

// ---------------------------------------------------------------------------
// Public native functions
// ---------------------------------------------------------------------------

/// `Data::save(path: String, value: T) -> Bool`
///
/// Serializes any Nova value to a JSON file.  Returns true on success.
/// The JSON embeds type information (`_type` tags) so it can be loaded back
/// with full type fidelity.
///
/// Closures and Functions cannot be serialized — the function returns a
/// runtime error if the value graph contains one.
pub fn save(state: &mut state::State) -> NovaResult<()> {
    let value = pop(state)?;
    let path = pop_string(state)?;

    match serialize_value(&state.memory, &value, 0) {
        Ok(json_value) => {
            // Pretty-print for human readability
            let json_str = serde_json::to_string_pretty(&json_value).map_err(|e| {
                runtime_err(format!("Data::save: JSON serialization failed: {}", e))
            })?;

            std::fs::write(&path, json_str).map_err(|e| {
                runtime_err(format!("Data::save: failed to write '{}': {}", path, e))
            })?;

            // Dec the value if it's a heap object (we popped it)
            state.memory.dec_value(value);
            state.memory.stack.push(VmData::Bool(true));
            Ok(())
        }
        Err(msg) => {
            state.memory.dec_value(value);
            Err(runtime_err(msg))
        }
    }
}

/// `Data::load(path: String) -> Option(T)`
///
/// Loads a Nova value from a JSON file previously created by `Data::save`.
/// Returns the deserialized value (which acts as `Some(T)`) on success,
/// or `VmData::None` if the file doesn't exist, is malformed, or the
/// type information doesn't match.
///
/// The caller uses `@[T: SomeType]` to tell the compiler what type to
/// expect.  At runtime, the deserializer reconstructs whatever the JSON
/// describes; the type annotation ensures compile-time safety.
pub fn load(state: &mut state::State) -> NovaResult<()> {
    let path = pop_string(state)?;

    // If file doesn't exist, return None
    if !std::path::Path::new(&path).exists() {
        state.memory.stack.push(VmData::None);
        return Ok(());
    }

    let content = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => {
            state.memory.stack.push(VmData::None);
            return Ok(());
        }
    };

    let json_value: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => {
            state.memory.stack.push(VmData::None);
            return Ok(());
        }
    };

    match deserialize_value(&mut state.memory, &json_value, 0) {
        Ok(vmdata) => {
            state.memory.stack.push(vmdata);
            Ok(())
        }
        Err(_) => {
            // Deserialization failed — return None (type mismatch, bad data, etc.)
            state.memory.stack.push(VmData::None);
            Ok(())
        }
    }
}

/// `Data::toJson(value: T) -> String`
///
/// Serializes any Nova value to a JSON string (without writing to a file).
/// Useful for debugging, networking, or logging.
pub fn to_json(state: &mut state::State) -> NovaResult<()> {
    let value = pop(state)?;

    match serialize_value(&state.memory, &value, 0) {
        Ok(json_value) => {
            let json_str = serde_json::to_string_pretty(&json_value).map_err(|e| {
                runtime_err(format!("Data::toJson: JSON serialization failed: {}", e))
            })?;
            state.memory.dec_value(value);
            state.memory.push_string(json_str);
            Ok(())
        }
        Err(msg) => {
            state.memory.dec_value(value);
            Err(runtime_err(msg))
        }
    }
}

/// `Data::fromJson(json: String) -> Option(T)`
///
/// Deserializes a Nova value from a JSON string.
/// Returns the value (Some) on success, or None if the JSON is invalid.
pub fn from_json(state: &mut state::State) -> NovaResult<()> {
    let json_str = pop_string(state)?;

    let json_value: Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(_) => {
            state.memory.stack.push(VmData::None);
            return Ok(());
        }
    };

    match deserialize_value(&mut state.memory, &json_value, 0) {
        Ok(vmdata) => {
            state.memory.stack.push(vmdata);
            Ok(())
        }
        Err(_) => {
            state.memory.stack.push(VmData::None);
            Ok(())
        }
    }
}
