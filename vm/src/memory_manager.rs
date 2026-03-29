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

impl VmData {
    pub fn to_string(&self) -> String {
        match self {
            VmData::Int(i) => i.to_string(),
            VmData::Float(f) => f.to_string(),
            VmData::Bool(b) => b.to_string(),
            VmData::Char(c) => c.to_string(),
            VmData::Function(idx) => format!("Function({})", idx),
            VmData::Object(idx) => format!("Object({})", idx),
            _ => "None".to_string(),
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
                return Some(self.data[index].clone());
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
                    result.insert(key.clone(), self.data[index].clone());
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
                            format!("{}", chars)
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
        let chars: Vec<VmData> = s.chars().map(VmData::Char).collect();
        let obj = Object {
            object_type: ObjectType::String,
            table: HashMap::new(),
            data: chars,
        };
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

    #[inline]
    pub fn dec(&mut self, index: usize) {
        if let Some(slot) = self.heap.get_mut(index) {
            if let Some(entry) = slot {
                if entry.ref_count > 0 {
                    entry.ref_count -= 1;
                }
                if entry.ref_count == 0 {
                    let entry = self.heap[index].take().unwrap();
                    // print out freed debug info
                    //println!("Freed: {:?}", entry.object);
                    // free the entry and its children

                    for child in &entry.object.data {
                        if let Some(child_idx) = get_heap_index(child) {
                            self.dec(child_idx);
                        }
                    }

                    // After recursive dec of children, the heap may have shrunk.
                    // Re-check whether this index is still within bounds.
                    if index >= self.heap.len() {
                        // Already removed by shrink_heap during child processing
                        return;
                    }

                    if index == self.heap.len() - 1 {
                        self.heap.pop();
                        self.shrink_heap();
                    } else {
                        self.free_list.push(index);
                    }
                }
            }
        }
    }

    #[inline]
    fn shrink_heap(&mut self) {
        while let Some(last) = self.heap.last() {
            if last.is_none() {
                self.heap.pop();
            } else {
                break;
            }
        }
        self.free_list.retain(|&idx| idx < self.heap.len());
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
        let new_index = match clone_obj.object_type {
            ObjectType::List => {
                let new_items: Vec<VmData> = clone_obj
                    .data
                    .iter()
                    .map(|item| match item {
                        VmData::Object(child) => {
                            VmData::Object(self.deep_clone_heap_internal(*child, cache))
                        }
                        other => *other,
                    })
                    .collect();
                self.allocate(Object {
                    object_type: ObjectType::List,
                    table: HashMap::new(),
                    data: new_items,
                })
            }
            ObjectType::Struct(name) => {
                let mut new_map = HashMap::new();
                let mut new_data = clone_obj.data.clone();
                for (k, &v) in &clone_obj.table {
                    let new_v = match &clone_obj.data[v] {
                        VmData::Object(child) => {
                            VmData::Object(self.deep_clone_heap_internal(*child, cache))
                        }
                        other => *other,
                    };
                    new_data[v] = new_v;
                    new_map.insert(k.clone(), v);
                }
                self.allocate(Object {
                    object_type: ObjectType::Struct(name),
                    table: new_map,
                    data: new_data,
                })
            }
            ObjectType::String => self.allocate(Object {
                object_type: ObjectType::String,
                table: HashMap::new(),
                data: clone_obj.data.clone(),
            }),
            ObjectType::Closure(func_ptr) => {
                let new_env: Vec<VmData> = clone_obj
                    .data
                    .iter()
                    .map(|item| match item {
                        VmData::Object(child) => {
                            VmData::Object(self.deep_clone_heap_internal(*child, cache))
                        }
                        other => *other,
                    })
                    .collect();
                self.allocate(Object {
                    object_type: ObjectType::Closure(func_ptr),
                    table: HashMap::new(),
                    data: new_env,
                })
            }
            ObjectType::Tuple => {
                let new_items: Vec<VmData> = clone_obj
                    .data
                    .iter()
                    .map(|item| match item {
                        VmData::Object(child) => {
                            VmData::Object(self.deep_clone_heap_internal(*child, cache))
                        }
                        other => *other,
                    })
                    .collect();
                self.allocate(Object {
                    object_type: ObjectType::Tuple,
                    table: HashMap::new(),
                    data: new_items,
                })
            }
            ObjectType::Enum { name, tag } => {
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
                self.allocate(Object {
                    object_type: ObjectType::Enum {
                        name: name.clone(),
                        tag: tag,
                    },
                    table: HashMap::new(),
                    data: new_data,
                })
            }
        };
        cache.insert(index, new_index);
        new_index
    }

    /// Internal cycle collection (mark-sweep).
    fn collect_cycles(&mut self) {
        let len = self.heap.len();
        let mut marked = vec![false; len];

        // Mark phase.
        let mut worklist = Vec::new();
        for value in &self.stack {
            if let Some(idx) = get_heap_index(value) {
                if idx < len && !marked[idx] {
                    marked[idx] = true;
                    worklist.push(idx);
                }
            }
        }
        while let Some(idx) = worklist.pop() {
            if let Some(Some(entry)) = self.heap.get(idx) {
                match entry.object.object_type {
                    ObjectType::List | ObjectType::Tuple => {
                        for item in &entry.object.data {
                            if let Some(child_idx) = get_heap_index(item) {
                                if child_idx < len && !marked[child_idx] {
                                    marked[child_idx] = true;
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    ObjectType::Struct(_) => {
                        for (_key, &item) in &entry.object.table {
                            if let Some(child_idx) = get_heap_index(&entry.object.data[item]) {
                                if child_idx < len && !marked[child_idx] {
                                    marked[child_idx] = true;
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    ObjectType::Closure(_) => {
                        for item in &entry.object.data {
                            if let Some(child_idx) = get_heap_index(item) {
                                if child_idx < len && !marked[child_idx] {
                                    marked[child_idx] = true;
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    ObjectType::Enum { .. } => {
                        for item in &entry.object.data {
                            if let Some(child_idx) = get_heap_index(item) {
                                if child_idx < len && !marked[child_idx] {
                                    marked[child_idx] = true;
                                    worklist.push(child_idx);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Sweep phase.
        for (i, maybe_entry) in self.heap.iter_mut().enumerate() {
            if maybe_entry.is_some() && !marked[i] {
                *maybe_entry = None;
                self.free_list.push(i);
            }
        }
        self.shrink_heap();

        // show debug info about heap after collection
        println!(
            "GC: live objects = {}, heap size = {}, free list size = {}",
            self.live_count(),
            self.heap.len(),
            self.free_list.len()
        );  
    }

    pub fn store(&mut self, index: usize, value: VmData) {
        // Decrement the reference count of the old value
        if let Some(old_value) = self.stack.get(index).cloned() {
            if let VmData::Object(idx) = old_value {
                self.dec(idx);
            }
        }
        if let Some(slot) = self.stack.get_mut(index) {
            *slot = value;
        }
    }

    pub fn pop_store_index(&mut self, index: usize) {
        if let Some(value) = self.stack.pop() {
            // Decrement the reference count of the old value
            if let Some(old_value) = self.stack.get(index) {
                if let VmData::Object(idx) = old_value {
                    self.dec(*idx);
                }
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
