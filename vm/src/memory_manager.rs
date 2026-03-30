use std::collections::HashMap;
use std::time::Instant;

/// The VM data types.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum VmData {
    // pointer to stack
    StackAddress(usize),

    // jump targets
    Function(usize),

    // basic types
    Int(i64),
    Float(f64),
    Bool(bool),
    Char(char),

    // pointer to any heap object
    Object(usize),

    None,
}

impl std::fmt::Display for VmData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmData::Int(i) => write!(f, "{}", i),
            VmData::Float(v) => write!(f, "{}", v),
            VmData::Bool(b) => write!(f, "{}", b),
            VmData::Char(c) => write!(f, "{}", c),
            VmData::Function(idx) => write!(f, "Function({})", idx),
            VmData::Object(idx) => write!(f, "Object({})", idx),
            VmData::StackAddress(addr) => write!(f, "StackAddress({})", addr),
            VmData::None => write!(f, "None"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ObjectType {
    List,
    String,
    Tuple,
    // Struct : tag
    Struct(String),
    // Enum : name , tag
    Enum { name: String, tag: i64 },
    // Closure : function pointer
    Closure(usize),
}

/// A heap object that can be a list, struct, string, tuple, closure or enum.
/// The object type is used to determine how to handle the object.
/// The table is used to store the fields of the object.
/// The data is the actual data of the object.
/// The table is a HashMap of field names to indices in the data vector.
/// The data is a vector of VmData, which can be any type.
#[derive(Debug, Clone)]
pub struct Object {
    pub object_type: ObjectType,
    pub table: HashMap<String, usize>,
    pub data: Vec<VmData>,
}

impl Object {
    /// Create a new heap object with the given type and data.
    pub fn new(object_type: ObjectType, data: Vec<VmData>) -> Self {
        Object {
            object_type,
            table: HashMap::new(),
            data,
        }
    }

    pub fn closure(func_ptr: usize, data: Vec<VmData>) -> Self {
        Object {
            object_type: ObjectType::Closure(func_ptr),
            table: HashMap::new(),
            data,
        }
    }

    pub fn list(data: Vec<VmData>) -> Self {
        Object {
            object_type: ObjectType::List,
            table: HashMap::new(),
            data,
        }
    }

    pub fn string(str: String) -> Self {
        Object {
            object_type: ObjectType::String,
            table: HashMap::new(),
            data: str.chars().map(VmData::Char).collect(),
        }
    }

    pub fn tuple(data: Vec<VmData>) -> Self {
        Object {
            object_type: ObjectType::Tuple,
            table: HashMap::new(),
            data,
        }
    }

    pub fn enum_object(name: String, tag: i64, data: Vec<VmData>) -> Self {
        Object {
            object_type: ObjectType::Enum { name, tag },
            table: HashMap::new(),
            data,
        }
    }

    pub fn insert(&mut self, key: String, value: usize) {
        self.data.push(VmData::Object(value));
        self.table.insert(key.clone(), self.data.len() - 1);
    }

    pub fn get(&self, key: &str) -> Option<VmData> {
        if let Some(&index) = self.table.get(key) {
            if index < self.data.len() {
                return Some(self.data[index]);
            }
        }
        None
    }

    pub fn push(&mut self, value: VmData) {
        self.data.push(value);
    }

    pub fn pop(&mut self) -> Option<VmData> {
        self.data.pop()
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get_type(&self) -> &ObjectType {
        &self.object_type
    }

    pub fn get_data(&self) -> &Vec<VmData> {
        &self.data
    }

    pub fn get_table(&self) -> &HashMap<String, usize> {
        &self.table
    }

    pub fn as_string(&self) -> Option<String> {
        if let ObjectType::String = self.object_type {
            let mut result = String::new();
            for item in &self.data {
                if let VmData::Char(c) = item {
                    result.push(*c);
                }
            }
            return Some(result);
        }
        None
    }

    pub fn as_list(&self) -> Option<Vec<VmData>> {
        if let ObjectType::List = self.object_type {
            return Some(self.data.clone());
        }
        None
    }

    pub fn as_tuple(&self) -> Option<Vec<VmData>> {
        if let ObjectType::Tuple = self.object_type {
            return Some(self.data.clone());
        }
        None
    }

    pub fn as_struct(&self) -> Option<HashMap<String, VmData>> {
        if let ObjectType::Struct(_) = &self.object_type {
            let mut result = HashMap::new();
            for (key, &index) in &self.table {
                if index < self.data.len() {
                    result.insert(key.clone(), self.data[index]);
                }
            }
            return Some(result);
        }
        None
    }

    pub fn as_enum(&self) -> Option<(String, i64, Vec<VmData>)> {
        if let ObjectType::Enum { name, tag } = &self.object_type {
            return Some((name.clone(), *tag, self.data.clone()));
        }
        None
    }

    pub fn as_closure(&self) -> Option<(usize, Vec<VmData>)> {
        if let ObjectType::Closure(func_ptr) = self.object_type {
            return Some((func_ptr, self.data.clone()));
        }
        None
    }
}

/// Each allocated heap entry carries a reference count and the object.
#[derive(Debug, Clone)]
pub struct HeapEntry {
    pub ref_count: usize,
    object: Object,
}

/// Helper: if a VmData variant holds a heap pointer, return its index.
#[inline(always)]
fn get_heap_index(value: &VmData) -> Option<usize> {
    match value {
        VmData::Object(idx) => Some(*idx),
        _ => None,
    }
}

/// Constants for dynamic threshold tuning.
const TARGET_FRAME_MS: u128 = 16; // ~16ms per frame (60 fps)
const MIN_THRESHOLD: usize = 5_000;
const MAX_THRESHOLD: usize = 1_000_000;

/// MemoryManager (our GC/stack system)
#[derive(Debug, Clone)]
pub struct MemoryManager {
    pub stack: Vec<VmData>,
    pub heap: Vec<Option<HeapEntry>>,
    free_list: Vec<usize>,
    base_threshold: usize,
    next_gc: usize,
    last_gc: Option<Instant>,
    /// Reusable mark-bit buffer — avoids allocating a new Vec every GC cycle.
    mark_bits: Vec<bool>,
}

impl MemoryManager {
    /// Create a new memory manager with a given base threshold.
    pub fn new(base_threshold: usize) -> Self {
        MemoryManager {
            stack: Vec::new(),
            heap: Vec::new(),
            free_list: Vec::new(),
            base_threshold,
            next_gc: base_threshold,
            last_gc: None,
            mark_bits: Vec::new(),
        }
    }

    /// Pretty-print a heap object recursively, with a maximum recursion depth.
    pub fn print_heap_object(&self, index: usize, depth: usize) -> String {
        if depth > 3 {
            return "...".to_string();
        }

        self.heap
            .get(index)
            .and_then(|entry_opt| {
                entry_opt.as_ref().map(|entry| {
                    let format_data = |data: &Vec<VmData>| {
                        data.iter()
                            .map(|item| match item {
                                VmData::Object(idx) => self.heap.get(*idx).map_or_else(
                                    || "Invalid Heap Object".to_string(),
                                    |_| self.print_heap_object(*idx, depth + 1),
                                ),
                                other => other.to_string(),
                            })
                            .collect::<Vec<String>>()
                            .join(", ")
                    };

                    match &entry.object.object_type {
                        ObjectType::List => format!("[{}]", format_data(&entry.object.data)),
                        ObjectType::Struct(name) => {
                            let fields_str = entry
                                .object
                                .table
                                .iter()
                                .map(|(k, &v)| {
                                    format!(
                                        "{}: {}",
                                        k,
                                        match &entry.object.data[v] {
                                            VmData::Object(idx) => self.heap.get(*idx).map_or_else(
                                                || "Invalid Heap Object".to_string(),
                                                |_| self.print_heap_object(*idx, depth + 1)
                                            ),
                                            other => other.to_string(),
                                        }
                                    )
                                })
                                .collect::<Vec<String>>()
                                .join(", ");
                            format!("{name}: {{{}}}", fields_str)
                        }
                        ObjectType::String => {
                            let chars: String = entry
                                .object
                                .data
                                .iter()
                                .filter_map(|item| {
                                    if let VmData::Char(c) = item {
                                        Some(*c)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            chars.to_string()
                        }
                        ObjectType::Closure(func_ptr) => {
                            format!(
                                "<Closure: {}, env: [{}]>",
                                func_ptr,
                                format_data(&entry.object.data)
                            )
                        }
                        ObjectType::Tuple => format!("({})", format_data(&entry.object.data)),
                        ObjectType::Enum { name, tag } => {
                            format!(
                                "Enum({name}: {}, [{}])",
                                tag,
                                format_data(&entry.object.data)
                            )
                        }
                    }
                })
            })
            .unwrap_or_else(|| "None".to_string())
    }

    /// Return the number of live heap entries.
    fn live_count(&self) -> usize {
        self.heap.len() - self.free_list.len()
    }

    /// Allocate a new heap object.
    pub fn allocate(&mut self, object: Object) -> usize {
        if self.live_count() >= self.next_gc {
            let now = Instant::now();
            if let Some(last) = self.last_gc {
                let delta = now.duration_since(last).as_millis();
                if delta < TARGET_FRAME_MS {
                    self.base_threshold = (self.base_threshold * 2).min(MAX_THRESHOLD);
                } else if delta > TARGET_FRAME_MS * 4 {
                    self.base_threshold = (self.base_threshold / 2).max(MIN_THRESHOLD);
                }
            }
            self.collect_cycles();
            self.last_gc = Some(now);
            self.next_gc = self.live_count() + self.base_threshold;
        }

        if let Some(idx) = self.free_list.pop() {
            self.heap[idx] = Some(HeapEntry {
                ref_count: 1,
                object,
            });
            return idx;
        }
        self.heap.push(Some(HeapEntry {
            ref_count: 1,
            object,
        }));
        self.heap.len() - 1
    }

    /// Allocate a ghost object (no reference count).
    /// This is used for temporary objects that are not tracked.
    /// It is the caller's responsibility to ensure that the object is not used after allocation.
    /// This is useful for objects that are not part of the heap, such as temporary values.
    #[inline]
    pub fn allocate_ghost(&mut self, object: Object) -> usize {
        let idx = self.allocate(object);
        if let Some(Some(entry)) = self.heap.get_mut(idx) {
            entry.ref_count = 0;
        }
        idx
    }

    /// push_string: push a string to the heap
    /// This will place the new index on the stack
    pub fn push_string(&mut self, s: String) {
        let obj = Object::string(s);
        let idx = self.allocate(obj);
        self.stack.push(VmData::Object(idx));
    }

    /// push_list: push a list to the heap
    /// This will place the new index on the stack
    /// The list is a vector of VmData
    pub fn push_list(&mut self, list: Vec<VmData>) {
        let obj = Object {
            object_type: ObjectType::List,
            table: HashMap::new(),
            data: list,
        };
        let idx = self.allocate(obj);
        self.stack.push(VmData::Object(idx));
    }

    pub fn dec_value(&mut self, value: VmData) {
        if let VmData::Object(idx) = value {
            self.dec(idx);
        }
    }

    /// Increment the reference count of a heap entry.
    pub fn inc_value(&mut self, value: VmData) {
        if let VmData::Object(idx) = value {
            self.inc(idx);
        }
    }

    #[inline]
    pub fn inc(&mut self, index: usize) {
        if let Some(Some(entry)) = self.heap.get_mut(index) {
            entry.ref_count += 1;
        }
    }

    /// Decrement reference count, freeing the entry and its children iteratively.
    /// Uses an explicit worklist to avoid stack overflow on deeply nested structures.
    #[inline]
    pub fn dec(&mut self, index: usize) {
        // Fast path: just decrement and return if still alive.
        if let Some(Some(entry)) = self.heap.get_mut(index) {
            if entry.ref_count > 1 {
                entry.ref_count -= 1;
                return;
            }
        }
        // Slow path: ref_count reaches 0 — iterative free.
        let mut worklist = vec![index];
        while let Some(idx) = worklist.pop() {
            if let Some(Some(entry)) = self.heap.get_mut(idx) {
                if entry.ref_count > 0 {
                    entry.ref_count -= 1;
                }
                if entry.ref_count == 0 {
                    let Some(entry) = self.heap[idx].take() else {
                        continue;
                    };
                    // Queue children for decrement instead of recursing.
                    for child in &entry.object.data {
                        if let Some(child_idx) = get_heap_index(child) {
                            worklist.push(child_idx);
                        }
                    }
                    // Reclaim slot.
                    self.free_list.push(idx);
                }
            }
        }
    }

    /// Shrink trailing None entries from the heap and clean up the free list.
    /// Only called during collect_cycles to amortise cost.
    fn shrink_heap(&mut self) {
        while let Some(last) = self.heap.last() {
            if last.is_none() {
                self.heap.pop();
            } else {
                break;
            }
        }
        let heap_len = self.heap.len();
        self.free_list.retain(|&idx| idx < heap_len);
    }

    #[inline]
    pub fn push(&mut self, value: VmData) {
        if let VmData::Object(idx) = value {
            self.inc(idx);
        }
        self.stack.push(value);
    }

    #[inline]
    pub fn pop(&mut self) -> Option<VmData> {
        if let Some(value) = self.stack.pop() {
            if let VmData::Object(idx) = value {
                self.dec(idx);
            }
            Some(value)
        } else {
            None
        }
    }

    // function to take top of stack and clone it and push it back on the stack
    #[inline]
    pub fn clone_top(&mut self) {
        if let Some(value) = self.pop() {
            let cloned_value = match value {
                VmData::Object(idx) => {
                    // allocate a new object
                    // and deep clone the object
                    let new_object = self.deep_clone_heap_internal(idx, &mut HashMap::new());
                    VmData::Object(new_object)
                }
                _ => value,
            };
            self.stack.push(cloned_value);
        }
    }

    pub fn ref_from_heap(&self, index: usize) -> Option<&Object> {
        if let Some(Some(entry)) = self.heap.get(index) {
            Some(&entry.object)
        } else {
            None
        }
    }

    pub fn ref_from_heap_mut(&mut self, index: usize) -> Option<&mut Object> {
        self.heap
            .get_mut(index)?
            .as_mut()
            .map(|entry| &mut entry.object)
    }

    pub fn ref_from_stack(&self, index: usize) -> Option<&VmData> {
        self.stack.get(index)
    }

    #[inline]
    pub fn fast_pop(&mut self) -> Option<VmData> {
        self.stack.pop()
    }

    #[inline]
    pub fn fast_push(&mut self, value: VmData) {
        self.stack.push(value);
    }

    pub fn collect(&mut self) {
        self.collect_cycles();
    }

    pub fn clone_reference_from(&mut self, value: usize, destination: usize) {
        if let Some(Some(src_entry)) = self.heap.get(value) {
            let object = src_entry.object.clone();
            if let Some(Some(dest_entry)) = self.heap.get_mut(destination) {
                dest_entry.object = object;
            }
        }
        // decrement the reference count of the old value
        self.dec(value);
    }

    pub fn deep_clone_from(&mut self, value: usize, destination: usize) {
        if let Some(value) = self.stack.get(value).cloned() {
            let cloned_value = match value {
                VmData::Object(idx) => {
                    VmData::Object(self.deep_clone_heap_internal(idx, &mut HashMap::new()))
                }
                _ => value,
            };
            self.stack[destination] = cloned_value;
        }
    }

    /// Shallow clone: given a heap pointer, increment its reference count and return the same pointer.
    pub fn shallow_clone_heap(&mut self, index: usize) -> usize {
        self.inc(index);
        index
    }

    /// Deep clone: given a heap pointer, return a new heap pointer to a deep copy.
    pub fn deep_clone_heap(&mut self, index: usize) -> usize {
        let mut cache = HashMap::new();
        self.deep_clone_heap_internal(index, &mut cache)
    }

    /// Internal deep clone implementation for a heap pointer.
    /// Unified path: deep-clone all children in `data`, preserve table only for structs.
    fn deep_clone_heap_internal(
        &mut self,
        index: usize,
        cache: &mut HashMap<usize, usize>,
    ) -> usize {
        if let Some(&new_index) = cache.get(&index) {
            return new_index;
        }
        let clone_obj = {
            let entry_opt = self.heap.get(index).and_then(|e| e.as_ref());
            if let Some(entry) = entry_opt {
                entry.object.clone()
            } else {
                return 0;
            }
        };

        // Deep-clone all children in data.
        let new_data: Vec<VmData> = clone_obj
            .data
            .iter()
            .map(|item| match item {
                VmData::Object(child) => {
                    VmData::Object(self.deep_clone_heap_internal(*child, cache))
                }
                other => *other,
            })
            .collect();

        // Struct objects carry a table mapping field names → data indices.
        let new_table = if matches!(clone_obj.object_type, ObjectType::Struct(_)) {
            clone_obj.table.clone()
        } else {
            HashMap::new()
        };

        let new_index = self.allocate(Object {
            object_type: clone_obj.object_type.clone(),
            table: new_table,
            data: new_data,
        });
        cache.insert(index, new_index);
        new_index
    }

    /// Internal cycle collection (mark-sweep).
    /// Uses a reusable mark-bit buffer to avoid allocation every cycle,
    /// and a unified traversal for all object types (every child in `data` is walked).
    fn collect_cycles(&mut self) {
        let len = self.heap.len();

        // Reuse the mark-bit buffer; resize and clear.
        self.mark_bits.resize(len, false);
        // Safety: all values are bool; fill with false.
        self.mark_bits.iter_mut().for_each(|b| *b = false);

        // Mark phase — roots are everything reachable from the stack.
        let mut worklist = Vec::new();
        for value in &self.stack {
            if let Some(idx) = get_heap_index(value) {
                if idx < len && !self.mark_bits[idx] {
                    self.mark_bits[idx] = true;
                    worklist.push(idx);
                }
            }
        }
        while let Some(idx) = worklist.pop() {
            if let Some(Some(entry)) = self.heap.get(idx) {
                // Walk all children in data — covers List, Tuple, Closure, Enum, String.
                for item in &entry.object.data {
                    if let Some(child_idx) = get_heap_index(item) {
                        if child_idx < len && !self.mark_bits[child_idx] {
                            self.mark_bits[child_idx] = true;
                            worklist.push(child_idx);
                        }
                    }
                }
                // Structs also keep references via the table.
                if matches!(entry.object.object_type, ObjectType::Struct(_)) {
                    for &data_idx in entry.object.table.values() {
                        if let Some(child_idx) = get_heap_index(&entry.object.data[data_idx]) {
                            if child_idx < len && !self.mark_bits[child_idx] {
                                self.mark_bits[child_idx] = true;
                                worklist.push(child_idx);
                            }
                        }
                    }
                }
            }
        }

        // Sweep phase.
        for i in 0..len {
            if self.heap[i].is_some() && !self.mark_bits[i] {
                self.heap[i] = None;
                self.free_list.push(i);
            }
        }
        self.shrink_heap();

        // GC complete — debug info only in dev builds
        #[cfg(debug_assertions)]
        eprintln!(
            "GC: live={}, heap_size={}, free_list={}",
            self.live_count(),
            self.heap.len(),
            self.free_list.len()
        );
    }

    pub fn store(&mut self, index: usize, value: VmData) {
        // Decrement the reference count of the old value
        if let Some(VmData::Object(idx)) = self.stack.get(index).cloned() {
            self.dec(idx);
        }
        if let Some(slot) = self.stack.get_mut(index) {
            *slot = value;
        }
    }

    pub fn pop_store_index(&mut self, index: usize) {
        if let Some(value) = self.stack.pop() {
            // Decrement the reference count of the old value
            if let Some(VmData::Object(idx)) = self.stack.get(index) {
                self.dec(*idx);
            }
            // Increment the reference count of the new value
            self.stack[index] = value;
        }
    }

    pub fn stack_index_to_stack(&mut self, value: usize) {
        if let Some(value) = self.stack.get(value).cloned() {
            if let VmData::Object(idx) = value {
                self.inc(idx);
            }
            self.stack.push(value);
        }
    }

    // Add helper method to create and push enums
    pub fn push_enum(&mut self, name: String, tag: i64, data: Vec<VmData>) {
        let obj = Object {
            object_type: ObjectType::Enum { name, tag },
            table: HashMap::new(),
            data,
        };
        let idx = self.allocate(obj);
        self.stack.push(VmData::Object(idx));
    }
}
