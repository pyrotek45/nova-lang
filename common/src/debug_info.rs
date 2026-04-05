use std::collections::HashMap;
use std::rc::Rc;

/// Debug metadata emitted by the compiler alongside the ASM stream.
/// Used by the disassembler (dis mode) and interactive debugger (dbg mode)
/// to show human-readable names for functions, variables, globals, and
/// to annotate jump/loop targets.
#[derive(Debug, Clone, Default)]
pub struct DebugInfo {
    /// label_id → human-readable name  ("fib", "main", "closure<println>", …)
    pub label_names: HashMap<u64, String>,

    /// global slot index → name  (0 → "print", 1 → "println", 5 → "fib", …)
    pub global_names: HashMap<u32, String>,

    /// For each function body: label_id → list of (local_index, name) for the
    /// locals in that function scope.  label_id 0 is used for the top-level scope.
    pub local_names: HashMap<u64, Vec<(u32, String)>>,

    /// native function index → name  (0 → "List::push", …)
    pub native_names: HashMap<u64, String>,

    /// Struct name → list of field names (in order)
    pub struct_fields: HashMap<String, Vec<String>>,
}

impl DebugInfo {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a global name, falling back to "global[N]".
    pub fn global_name(&self, index: u32) -> String {
        self.global_names
            .get(&index)
            .cloned()
            .unwrap_or_else(|| format!("global[{}]", index))
    }

    /// Look up a local name within a function scope (identified by label_id).
    /// `scope` is 0 for top-level.
    pub fn local_name(&self, scope: u64, index: u32) -> Option<String> {
        self.local_names.get(&scope).and_then(|locals| {
            locals
                .iter()
                .find(|(i, _)| *i == index)
                .map(|(_, name)| name.clone())
        })
    }

    /// Look up a label name, falling back to "L<id>".
    pub fn label_name(&self, id: u64) -> String {
        self.label_names
            .get(&id)
            .cloned()
            .unwrap_or_else(|| format!("L{}", id))
    }

    /// Look up a native function name.
    pub fn native_name(&self, index: u64) -> String {
        self.native_names
            .get(&index)
            .cloned()
            .unwrap_or_else(|| format!("native#{}", index))
    }

    /// Record the top-level scope local variables.
    pub fn set_toplevel_locals(&mut self, names: Vec<(u32, String)>) {
        self.local_names.insert(0, names);
    }
}

/// Helper to extract DebugInfo from the compiler's tables after compilation.
/// Called from novacore after `compile_program`.
pub fn extract_debug_info(
    global_table: &crate::table::Table<Rc<str>>,
    native_table: &crate::table::Table<Rc<str>>,
    variable_table: &crate::table::Table<Rc<str>>,
    asm: &[crate::code::Asm],
) -> DebugInfo {
    use crate::code::Asm;
    let mut info = DebugInfo::new();

    // Global names
    for (i, name) in global_table.items.iter().enumerate() {
        // Skip compiler-internal mangled string literals
        if !name.starts_with("String__literal__") {
            info.global_names.insert(i as u32, name.to_string());
        }
    }

    // Native function names
    for (i, name) in native_table.items.iter().enumerate() {
        info.native_names.insert(i as u64, name.to_string());
    }

    // Top-level local variable names
    let locals: Vec<(u32, String)> = variable_table
        .items
        .iter()
        .enumerate()
        .filter(|(_, name)| {
            !name.starts_with("__tempcounter__")
                && !name.starts_with("__arrayexpr__")
                && !name.starts_with("___matchexpr___")
        })
        .map(|(i, name)| (i as u32, name.to_string()))
        .collect();
    info.set_toplevel_locals(locals);

    // Scan ASM for FUNCTION/CLOSURE → STOREGLOBAL patterns to map labels to names
    // The compiler emits:  FUNCTION(label) ... LABEL(label) STOREGLOBAL(idx)
    // So label → global_name[idx]
    let mut fn_labels: Vec<u64> = Vec::new();
    let mut cls_labels: Vec<u64> = Vec::new();

    for inst in asm.iter() {
        match inst {
            Asm::FUNCTION(lbl) => fn_labels.push(*lbl),
            Asm::CLOSURE(lbl) => cls_labels.push(*lbl),
            _ => {}
        }
    }

    // Walk through looking for LABEL(x) followed by STOREGLOBAL(idx)
    for i in 0..asm.len().saturating_sub(1) {
        if let Asm::LABEL(lbl) = &asm[i] {
            if let Asm::STOREGLOBAL(idx) = &asm[i + 1] {
                if let Some(name) = info.global_names.get(idx) {
                    info.label_names.insert(*lbl, name.clone());
                }
            }
        }
    }

    // For any FUNCTION/CLOSURE labels not yet named, give them generic names
    let mut unnamed_fn = 0u32;
    let mut unnamed_cls = 0u32;
    for lbl in &fn_labels {
        if !info.label_names.contains_key(lbl) {
            info.label_names
                .insert(*lbl, format!("fn_{}", unnamed_fn));
            unnamed_fn += 1;
        }
    }
    for lbl in &cls_labels {
        if !info.label_names.contains_key(lbl) {
            info.label_names
                .insert(*lbl, format!("closure_{}", unnamed_cls));
            unnamed_cls += 1;
        }
    }

    info
}
