use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;

use common::code::Asm;
use common::error::{NovaError, NovaResult};
use common::fileposition::FilePosition;
use common::gen::Gen;
use common::nodes::Statement::{Block, Expression, For, Function, If, Return, Struct, While};
use common::nodes::{Ast, Atom, Expr, Pattern, Statement};
use common::table::Table;
use common::ttype::TType;

#[derive(Debug, Clone)]
pub struct Compiler {
    pub bindings: common::table::Table<Rc<str>>,
    pub global: common::table::Table<Rc<str>>,
    pub variables: common::table::Table<Rc<str>>,
    pub upvalues: common::table::Table<Rc<str>>,
    pub native_functions: common::table::Table<Rc<str>>,
    pub native_functions_types: HashMap<Rc<str>, TType>,
    pub output: Vec<u8>,
    pub filepath: Option<Rc<Path>>,
    pub entry: usize,
    pub asm: Vec<Asm>,
    pub gen: Gen,
    pub breaks: Vec<u64>,
    pub continues: Vec<u64>,
    pub global_strings: HashMap<Rc<str>, usize>,
    pub unrolled_index: HashMap<Rc<str>, usize>,
    /// Per-function local variable names: (label_id, [(slot, name), ...])
    pub fn_local_names: Vec<(u64, Vec<(u32, String)>)>,
}

pub fn new() -> Compiler {
    Compiler {
        native_functions: Table::new(),
        variables: Table::new(),
        output: Vec::new(),
        filepath: None,
        upvalues: Table::new(),
        global: Table::new(),
        entry: 0,
        bindings: Table::new(),
        asm: vec![],
        gen: common::gen::new(),
        breaks: vec![],
        continues: vec![],
        native_functions_types: HashMap::default(),
        global_strings: HashMap::default(),
        unrolled_index: HashMap::default(),
        fn_local_names: Vec::new(),
    }
}

impl Compiler {
    fn compile_string_literal(&mut self, string: &str) {
        let index = self.insert_string_global(string.into());
        self.asm.push(Asm::GETGLOBAL(index as u32));
        //dbg!(&index, &string);
    }

    fn insert_string_global(&mut self, string: Rc<str>) -> usize {
        // mangle string input to String__literal__<string>
        let mangled_string = Rc::from(format!("String__literal__{}", string));
        if let Some(index) = self.global_strings.get(&string) {
            *index
        } else {
            self.global.insert(mangled_string);
            let index = self.global.len() - 1;
            self.global_strings.insert(string, index);
            index
        }
    }

    pub fn clear(&mut self) {
        self.output.clear()
    }

    pub fn get_entry(&self) -> usize {
        self.entry
    }

    pub fn init(&mut self) {
        //dbg!("init");
        // compile wrapper functions into global scope
        // print function
        self.global.insert("print".into());
        let w_index = self.global.len() - 1;
        let jump = self.gen.generate();
        self.asm.push(Asm::FUNCTION(jump));
        self.asm.push(Asm::OFFSET(1, 0));
        self.asm.push(Asm::PRINT);
        self.asm.push(Asm::RET(false));
        self.asm.push(Asm::LABEL(jump));
        self.asm.push(Asm::STOREGLOBAL(w_index as u32));

        // println function
        self.global.insert("println".into());
        let w_index = self.global.len() - 1;
        let jump = self.gen.generate();
        self.asm.push(Asm::FUNCTION(jump));
        self.asm.push(Asm::OFFSET(1, 0));
        self.asm.push(Asm::PRINT);
        self.compile_string_literal("\n");
        self.asm.push(Asm::PRINT);
        self.asm.push(Asm::RET(false));
        self.asm.push(Asm::LABEL(jump));
        self.asm.push(Asm::STOREGLOBAL(w_index as u32));

        for (index, native) in self.native_functions.items.iter().enumerate() {
            if let Some(ntype) = self.native_functions_types.get(native) {
                match ntype {
                    TType::Function {
                        parameters,
                        return_type,
                    } => {
                        self.global.insert(native.clone());
                        let w_index = self.global.len() - 1;
                        let jump = self.gen.generate();
                        self.asm.push(Asm::FUNCTION(jump));
                        self.asm.push(Asm::OFFSET(parameters.len() as u32, 0));
                        self.asm.push(Asm::NATIVE(index as u64, None));
                        if **return_type != TType::Void {
                            self.asm.push(Asm::RET(true));
                        } else {
                            self.asm.push(Asm::RET(false));
                        }
                        self.asm.push(Asm::LABEL(jump));
                        self.asm.push(Asm::STOREGLOBAL(w_index as u32));
                    }
                    _ => {
                        todo!("not implemented");
                    }
                }
            } else {
                // dbg!(native);
                // todo!( "not implemented" );
            }
        }
    }

    #[inline(always)]
    pub fn compile_program(
        &mut self,
        input: Ast,
        filepath: impl Into<Option<Rc<Path>>>,
        alloc: bool,
        global: bool,
        function: bool,
        keep: bool,
    ) -> NovaResult<Vec<Asm>> {
        self.filepath = filepath.into();
        // create wrapper functions for builtin functions
        //dbg!(&self.native_functions);

        for statements in input.program.iter() {
            match statements {
                common::nodes::Statement::Foreach {
                    identifier,
                    expr,
                    body,
                    position,
                } => {
                    let top = self.gen.generate();
                    let end = self.gen.generate();

                    let mid = self.gen.generate();
                    let step = self.gen.generate();

                    let next = self.gen.generate();

                    self.breaks.push(end);
                    self.continues.push(next);

                    // insert temp counter
                    self.variables
                        .insert(format!("__tempcounter__{}", self.gen.generate()).into());
                    let tempcounter_index = self.variables.len() - 1;

                    self.variables
                        .insert(format!("__arrayexpr__{}", self.gen.generate()).into());
                    let array_index = self.variables.len() - 1;

                    let id_index = if let Some(index) = self.variables.get_index(identifier) {
                        index
                    } else {
                        self.variables.insert(identifier.clone());
                        self.variables.len() - 1
                    };

                    self.compile_expr(expr)?;
                    self.asm.push(Asm::STORE(array_index as u32));

                    // storing counter and expression array
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::STORE(tempcounter_index as u32));

                    // if array is empty jump to end
                    self.asm.push(Asm::LABEL(top));

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LEN);

                    self.asm.push(Asm::IGTR);
                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::NOT);

                    self.asm.push(Asm::JUMPIFFALSE(step));
                    self.asm.push(Asm::POP);

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LEN);
                    self.asm.push(Asm::EQUALS);

                    self.asm.push(Asm::LABEL(step));
                    self.asm.push(Asm::JUMPIFFALSE(mid));
                    self.asm.push(Asm::JMP(end));
                    self.asm.push(Asm::LABEL(mid));

                    // bind value
                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LIN(position.clone()));

                    self.asm.push(Asm::STORE(id_index as u32));

                    // -- body
                    let foreach_body = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(
                        foreach_body,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        false,
                    )?;

                    self.asm.pop();
                    // -- body

                    self.asm.push(Asm::LABEL(next));
                    // increment counter
                    self.asm.push(Asm::INTEGER(1));
                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::INTEGER(tempcounter_index as i64));
                    self.asm.push(Asm::ASSIGN);
                    self.asm.push(Asm::BJMP(top));
                    self.asm.push(Asm::LABEL(end));

                    self.breaks.pop();
                    self.continues.pop();
                }
                common::nodes::Statement::ForeachDestructure {
                    pattern,
                    expr,
                    body,
                    position,
                } => {
                    let top = self.gen.generate();
                    let end = self.gen.generate();
                    let mid = self.gen.generate();
                    let step = self.gen.generate();
                    let next = self.gen.generate();

                    self.breaks.push(end);
                    self.continues.push(next);

                    // temp counter
                    self.variables
                        .insert(format!("__tempcounter__{}", self.gen.generate()).into());
                    let tempcounter_index = self.variables.len() - 1;

                    // temp array
                    self.variables
                        .insert(format!("__arrayexpr__{}", self.gen.generate()).into());
                    let array_index = self.variables.len() - 1;

                    // temp for the current element (used for pattern binding)
                    self.variables
                        .insert(format!("__foreachelem__{}", self.gen.generate()).into());
                    let elem_temp = self.variables.len() - 1;

                    self.compile_expr(expr)?;
                    self.asm.push(Asm::STORE(array_index as u32));

                    // counter = 0
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::STORE(tempcounter_index as u32));

                    // loop top
                    self.asm.push(Asm::LABEL(top));

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LEN);

                    self.asm.push(Asm::IGTR);
                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::NOT);

                    self.asm.push(Asm::JUMPIFFALSE(step));
                    self.asm.push(Asm::POP);

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LEN);
                    self.asm.push(Asm::EQUALS);

                    self.asm.push(Asm::LABEL(step));
                    self.asm.push(Asm::JUMPIFFALSE(mid));
                    self.asm.push(Asm::JMP(end));
                    self.asm.push(Asm::LABEL(mid));

                    // Get current element: array[counter]
                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LIN(position.clone()));

                    // Store element in temp, then destructure using pattern bindings
                    self.asm.push(Asm::STORE(elem_temp as u32));
                    self.compile_pattern_bindings(pattern, elem_temp, position)?;

                    // -- body
                    let foreach_body = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(
                        foreach_body,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        false,
                    )?;
                    self.asm.pop();
                    // -- body

                    self.asm.push(Asm::LABEL(next));
                    // increment counter
                    self.asm.push(Asm::INTEGER(1));
                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::INTEGER(tempcounter_index as i64));
                    self.asm.push(Asm::ASSIGN);
                    self.asm.push(Asm::BJMP(top));
                    self.asm.push(Asm::LABEL(end));

                    self.breaks.pop();
                    self.continues.pop();
                }
                common::nodes::Statement::Pass => {}
                Function {
                    identifier,
                    parameters,
                    body,
                    captures: captured,
                    ..
                } => {
                    self.global.insert(identifier.clone());
                    // Clone the current state to prepare for function compilation
                    let mut function_compile = self.clone();
                    function_compile.variables.clear();
                    function_compile.asm.clear();
                    function_compile.fn_local_names.clear();

                    // Register parameter names in the function's local variable scope
                    for param in parameters.iter() {
                        function_compile.variables.insert(param.identifier.clone());
                    }

                    // Register captured variables in the function's local variable scope
                    for capture in captured.iter() {
                        function_compile.variables.insert(capture.clone());
                    }

                    // Compile captured variables for the closure
                    //dbg!(captured);
                    for captured_var in captured {
                        if let Some(index) = self.variables.get_index(captured_var) {
                            // Get the local variable if it exists in the current scope
                            self.asm.push(Asm::GET(index as u32));
                        } else if let Some(index) = self.global.get_index(captured_var) {
                            // Otherwise, get the global variable if it exists
                            self.asm.push(Asm::GETGLOBAL(index as u32));
                        } else {
                            return Err(Box::new(common::error::NovaError::Compiler {
                                msg: format!(
                                    "Captured variable '{}' not found in local or global scope",
                                    captured_var
                                )
                                .into(),
                                note: format!("The closure captures `{}` but it could not be found. Make sure the variable is defined in an enclosing scope before the closure.", captured_var).into(),
                            }));
                        }
                    }

                    // Generate a jump label for the closure function
                    let closure_jump_label = function_compile.gen.generate();

                    // Prepare the closure with a function label or a captured list if necessary
                    if captured.is_empty() {
                        self.asm.push(Asm::FUNCTION(closure_jump_label));
                    } else {
                        self.asm.push(Asm::INTEGER(captured.len() as i64));
                        self.asm.push(Asm::CLOSURE(closure_jump_label));
                    }

                    // Compile the function body
                    let function_body = Ast {
                        program: body.clone(),
                    };
                    let _ = function_compile.compile_program(
                        function_body,
                        self.filepath.clone(),
                        true,
                        false,
                        true,
                        false,
                    )?;

                    // Adjust the function's offset to account for parameters and captured variables
                    let num_parameters = parameters.len() as u32;
                    let num_captures = captured.len() as u32;
                    let local_vars = function_compile.variables.len() as u32;
                    //dbg!(ttype, identifier, num_parameters, num_captures, local_vars);
                    self.asm.push(Asm::OFFSET(
                        num_parameters + num_captures,
                        local_vars - (num_parameters + num_captures),
                    ));

                    // Record per-function local names for debug info
                    let fn_locals: Vec<(u32, String)> = function_compile
                        .variables
                        .items
                        .iter()
                        .enumerate()
                        .map(|(i, name)| (i as u32, name.to_string()))
                        .collect();
                    self.fn_local_names.push((closure_jump_label, fn_locals));
                    self.fn_local_names.append(&mut function_compile.fn_local_names);

                    // Append compiled function instructions to the current scope
                    self.gen = function_compile.gen;
                    self.global = function_compile.global;
                    self.global_strings = function_compile.global_strings;
                    function_compile.asm.pop(); // Remove the last instruction from function compilation
                    self.asm.extend_from_slice(&function_compile.asm);
                    self.asm.push(Asm::LABEL(closure_jump_label));

                    // storeing global function
                    if let Some(index) = self.global.get_index(identifier) {
                        self.asm.push(Asm::STOREGLOBAL(index as u32));
                    } else {
                        self.global.insert(identifier.clone());
                        let index = self.global.len() - 1;
                        self.asm.push(Asm::STOREGLOBAL(index as u32));
                    }
                }

                Struct {
                    ttype: _,
                    identifier,
                    fields,
                } => {
                    self.global.insert(identifier.clone());
                    let index = self.global.len() - 1;
                    let structjump = self.gen.generate();
                    self.asm.push(Asm::FUNCTION(structjump));
                    // fields includes the auto-added "type" field at the end
                    // real user fields = fields.len() - 1
                    let num_real_fields = fields.len() - 1;
                    self.asm.push(Asm::OFFSET(num_real_fields as u32, 0_u32));
                    // Push field name strings (in order) for each real field
                    for field in fields.iter().take(num_real_fields) {
                        self.compile_string_literal(&field.identifier);
                    }
                    // Push struct name string
                    self.compile_string_literal(identifier);
                    // NEWSTRUCT(num_real_fields) creates Object with Struct type + populated table
                    self.asm.push(Asm::NEWSTRUCT(num_real_fields as u64));
                    self.asm.push(Asm::RET(true));
                    self.asm.push(Asm::LABEL(structjump));

                    self.asm.push(Asm::STOREGLOBAL(index as u32));
                }

                Return { ttype, expr } => {
                    self.compile_expr(expr)?;
                    if ttype != &TType::Void {
                        self.asm.push(Asm::RET(true))
                    } else {
                        self.asm.push(Asm::RET(false))
                    }
                }
                Expression { ttype, expr } => {
                    self.compile_expr(expr)?;
                    if !keep && ttype != &TType::Void {
                        self.asm.push(Asm::POP);
                    }
                }
                If {
                    ttype: _,
                    test,
                    body,
                    alternative,
                } => {
                    let (bodyjump, alterjump) = (self.gen.generate(), self.gen.generate());
                    self.compile_expr(test)?;
                    self.asm.push(Asm::JUMPIFFALSE(bodyjump));
                    let body_ast = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(
                        body_ast,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        keep,
                    )?;
                    self.asm.pop(); // remove trailing RET(false)

                    if let Some(alternative) = alternative {
                        self.asm.push(Asm::JMP(alterjump));
                        self.asm.push(Asm::LABEL(bodyjump));
                        let alt = Ast {
                            program: alternative.clone(),
                        };
                        self.compile_program(
                            alt,
                            self.filepath.clone(),
                            false,
                            false,
                            false,
                            keep,
                        )?;
                        self.asm.pop(); // remove trailing RET(false)
                        self.asm.push(Asm::LABEL(alterjump));
                    } else {
                        self.asm.push(Asm::LABEL(bodyjump));
                    }
                }

                While { test, body } => {
                    let top = self.gen.generate();
                    let end = self.gen.generate();
                    self.breaks.push(end);
                    self.continues.push(top);
                    self.asm.push(Asm::LABEL(top));
                    self.compile_expr(test)?;
                    self.asm.push(Asm::JUMPIFFALSE(end));
                    let whilebody = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(
                        whilebody,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        false,
                    )?;
                    self.asm.pop();
                    self.asm.push(Asm::BJMP(top));
                    self.asm.push(Asm::LABEL(end));
                    self.breaks.pop();
                    self.continues.pop();
                }
                For {
                    init,
                    test,
                    inc,
                    body,
                } => {
                    let top = self.gen.generate();
                    let end = self.gen.generate();
                    let next = self.gen.generate();
                    self.breaks.push(end);
                    self.continues.push(next);
                    self.compile_expr(init)?;
                    self.asm.push(Asm::LABEL(top));
                    self.compile_expr(test)?;
                    self.asm.push(Asm::JUMPIFFALSE(end));
                    let whilebody = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(
                        whilebody,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        false,
                    )?;
                    self.asm.pop();
                    self.asm.push(Asm::LABEL(next));
                    self.compile_expr(inc)?;
                    self.asm.push(Asm::BJMP(top));
                    self.asm.push(Asm::LABEL(end));
                    self.breaks.pop();
                    self.continues.pop();
                }
                common::nodes::Statement::Break => {
                    if let Some(target) = self.breaks.last() {
                        self.asm.push(Asm::JMP(*target));
                    } else {
                        todo!()
                    }
                }
                common::nodes::Statement::Continue => {
                    if let Some(target) = self.continues.last() {
                        self.asm.push(Asm::BJMP(*target));
                    } else {
                        todo!()
                    }
                }
                Block { body, filepath } => {
                    let body = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(body, filepath.clone(), false, false, false, false)?;
                    self.asm.pop();
                }
                common::nodes::Statement::Unwrap {
                    ttype: _,
                    identifier,
                    body,
                    alternative,
                } => {
                    let skip = self.gen.generate();
                    let end = self.gen.generate();
                    if let Some(index) = self.variables.get_index(identifier) {
                        self.asm.push(Asm::GET(index as u32))
                    }
                    if let Some(index) = self.native_functions.get_index("Option::isSome") {
                        self.asm.push(Asm::NATIVE(index as u64, None))
                    }
                    self.asm.push(Asm::ISSOME);
                    self.asm.push(Asm::JUMPIFFALSE(skip));
                    let body = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(body, self.filepath.clone(), false, false, false, false)?;
                    self.asm.pop();
                    self.asm.push(Asm::JMP(end));
                    self.asm.push(Asm::LABEL(skip));
                    if let Some(alternative) = alternative {
                        let alternative = Ast {
                            program: alternative.clone(),
                        };
                        self.compile_program(
                            alternative,
                            self.filepath.clone(),
                            false,
                            false,
                            false,
                            false,
                        )?;
                        self.asm.pop();
                    }
                    self.asm.push(Asm::LABEL(end));
                }
                common::nodes::Statement::IfLet {
                    ttype: _,
                    identifier,
                    expr,
                    body,
                    global,
                    alternative,
                } => {
                    let skip = self.gen.generate();
                    let end = self.gen.generate();
                    self.compile_expr(expr)?;
                    if let Some(index) = self.native_functions.get_index("Option::isSome") {
                        self.asm.push(Asm::NATIVE(index as u64, None))
                    }
                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::ISSOME);
                    self.asm.push(Asm::JUMPIFFALSE(skip));
                    let id_index = if let Some(index) = self.variables.get_index(identifier) {
                        index
                    } else {
                        self.variables.insert(identifier.clone());
                        self.variables.len() - 1
                    };
                    if *global {
                        self.asm.push(Asm::STOREGLOBAL(id_index as u32))
                    } else {
                        self.asm.push(Asm::STORE(id_index as u32))
                    }
                    let body = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(body, self.filepath.clone(), false, false, false, false)?;
                    self.asm.pop();
                    self.asm.push(Asm::JMP(end));
                    self.asm.push(Asm::LABEL(skip));
                    self.asm.push(Asm::POP);
                    if let Some(alternative) = alternative {
                        let alternative = Ast {
                            program: alternative.clone(),
                        };
                        self.compile_program(
                            alternative,
                            self.filepath.clone(),
                            false,
                            false,
                            false,
                            false,
                        )?;
                        self.asm.pop();
                    }
                    self.asm.push(Asm::LABEL(end));
                }
                common::nodes::Statement::Enum {
                    identifier, fields, ..
                } => {
                    for (tag, field) in fields.iter().enumerate() {
                        if field.identifier.as_ref() == "type" {
                            continue;
                        }

                        self.global
                            .insert(format!("{}::{}", identifier, field.identifier).into());
                        let index = self.global.len() - 1;

                        //dbg!(format!("{}::{}", identifier, field.identifier));

                        let structjump = self.gen.generate();
                        self.asm.push(Asm::FUNCTION(structjump));
                        // offset is what it will accept
                        // enum is stored as a tuple [value,tag,type]
                        if field.ttype != TType::None {
                            self.asm.push(Asm::OFFSET(1_u32, 0_u32));
                        } else {
                            self.asm.push(Asm::OFFSET(0_u32, 0_u32));
                            self.asm.push(Asm::NONE);
                        }

                        self.asm.push(Asm::INTEGER(tag as i64));
                        self.compile_string_literal(identifier);
                        self.asm.push(Asm::LIST(3_u64));
                        self.asm.push(Asm::RET(true));

                        self.asm.push(Asm::LABEL(structjump));

                        self.asm.push(Asm::STOREGLOBAL(index as u32));
                    }
                }
                common::nodes::Statement::Match {
                    expr,
                    arms,
                    default,
                    position,
                    ..
                } => {
                    // we will do it old school, each arm will make a new if branch
                    // and if it fails it will jump to the next arm
                    // if there is a default it will jump to the default
                    // if there is no default it will jump to the end
                    let end = self.gen.generate();

                    self.compile_expr(expr)?;
                    // store in temp variable
                    self.variables
                        .insert(format!("___matchexpr___{}", self.gen.generate()).into());
                    let temp_matchexpr = self.variables.len() - 1;
                    self.asm.push(Asm::STORE(temp_matchexpr as u32));

                    //dbg!(temp_matchexpr);
                    //dbg!(arms.len());
                    for arm in arms.iter() {
                        //dbg!(arm.0);
                        let next = self.gen.generate();
                        self.asm.push(Asm::INTEGER(1_i64));
                        self.asm.push(Asm::GET(temp_matchexpr as u32));
                        self.asm.push(Asm::LIN(position.clone()));
                        self.asm.push(Asm::INTEGER(arm.0 as i64));
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::JUMPIFFALSE(next));
                        if let Some(vid) = &arm.1 {
                            self.asm.push(Asm::INTEGER(0_i64));
                            self.asm.push(Asm::GET(temp_matchexpr as u32));
                            self.asm.push(Asm::LIN(position.clone()));
                            // store the vid in the variable

                            if let Some(index) = self.variables.get_index(vid) {
                                self.asm.push(Asm::STORE(index as u32))
                            } else {
                                //dbg!(vid);
                                self.variables.insert(vid.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32))
                            }
                        }

                        let arm = Ast {
                            program: arm.2.clone(),
                        };
                        self.compile_program(
                            arm,
                            self.filepath.clone(),
                            false,
                            false,
                            false,
                            false,
                        )?;
                        self.asm.pop();

                        self.asm.push(Asm::JMP(end));
                        self.asm.push(Asm::LABEL(next));
                    }
                    if let Some(default) = default {
                        let default = Ast {
                            program: default.clone(),
                        };
                        self.compile_program(
                            default,
                            self.filepath.clone(),
                            false,
                            false,
                            false,
                            false,
                        )?;
                        self.asm.pop();
                    }
                    self.asm.push(Asm::LABEL(end));
                }
                common::nodes::Statement::ForRange {
                    identifier,
                    start: start_expr,
                    end: end_expr,
                    inclusive,
                    step,
                    body,
                } => {
                    match (&start_expr, &end_expr) {
                        (
                            Expr::Literal {
                                value: Atom::Integer { value: start },
                                ..
                            },
                            Expr::Literal {
                                value: Atom::Integer { value: end },
                                ..
                            },
                        ) if *end < 255 => {
                            // unroll loop

                            let endp = self.gen.generate();
                            self.breaks.push(endp);

                            // If the loop variable already exists as a real variable
                            // (from a prior loop), we must also store the unrolled
                            // value into the variable slot so inner code that reads
                            // from variables (checked before unrolled_index) sees the
                            // correct value for this iteration.
                            let existing_var_index =
                                self.variables.get_index(identifier).map(|i| i as u32);

                            if *inclusive {
                                for value in *start..*end {
                                    let nextp = self.gen.generate();
                                    self.continues.push(nextp);
                                    self.unrolled_index
                                        .insert(identifier.clone(), value as usize);
                                    if let Some(idx) = existing_var_index {
                                        self.asm.push(Asm::INTEGER(value));
                                        self.asm.push(Asm::STORE(idx));
                                    }
                                    let whilebody = Ast {
                                        program: body.clone(),
                                    };
                                    self.compile_program(
                                        whilebody,
                                        self.filepath.clone(),
                                        false,
                                        false,
                                        false,
                                        false,
                                    )?;
                                    self.asm.pop();
                                    self.unrolled_index.remove(identifier);
                                    self.asm.push(Asm::LABEL(nextp));
                                    self.continues.pop();
                                }
                            } else {
                                for value in *start..=*end {
                                    let nextp = self.gen.generate();
                                    self.continues.push(nextp);
                                    self.unrolled_index
                                        .insert(identifier.clone(), value as usize);
                                    if let Some(idx) = existing_var_index {
                                        self.asm.push(Asm::INTEGER(value));
                                        self.asm.push(Asm::STORE(idx));
                                    }
                                    let whilebody = Ast {
                                        program: body.clone(),
                                    };
                                    self.compile_program(
                                        whilebody,
                                        self.filepath.clone(),
                                        false,
                                        false,
                                        false,
                                        false,
                                    )?;
                                    self.asm.pop();
                                    self.unrolled_index.remove(identifier);
                                    self.asm.push(Asm::LABEL(nextp));
                                    self.continues.pop();
                                }
                            }
                            self.asm.push(Asm::LABEL(endp));
                            self.breaks.pop();
                            self.continues.pop();
                        }
                        _ => {
                            let top = self.gen.generate();
                            let end = self.gen.generate();
                            let next = self.gen.generate();

                            self.breaks.push(end);
                            self.continues.push(next);

                            // start of range
                            self.compile_expr(start_expr)?;
                            if let Some(index) = self.variables.get_index(identifier) {
                                self.asm.push(Asm::STORE(index as u32))
                            } else {
                                self.variables.insert(identifier.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32))
                            }

                            // top of loop
                            self.asm.push(Asm::LABEL(top));
                            // test if we are at the end
                            self.compile_expr(end_expr)?;
                            if let Some(index) = self.variables.get_index(identifier) {
                                self.asm.push(Asm::GET(index as u32))
                            }
                            // todo inclusive
                            if *inclusive {
                                self.asm.push(Asm::IGTR);
                            } else {
                                let sc = self.gen.generate();
                                self.asm.push(Asm::IGTR);
                                self.asm.push(Asm::DUP);
                                self.asm.push(Asm::NOT);
                                self.asm.push(Asm::JUMPIFFALSE(sc));
                                self.asm.push(Asm::POP);
                                self.compile_expr(end_expr)?;
                                if let Some(index) = self.variables.get_index(identifier) {
                                    self.asm.push(Asm::GET(index as u32))
                                }
                                self.asm.push(Asm::EQUALS);
                                self.asm.push(Asm::LABEL(sc))
                            }

                            self.asm.push(Asm::JUMPIFFALSE(end));

                            let whilebody = Ast {
                                program: body.clone(),
                            };
                            self.compile_program(
                                whilebody,
                                self.filepath.clone(),
                                false,
                                false,
                                false,
                                false,
                            )?;
                            self.asm.pop();

                            self.asm.push(Asm::LABEL(next));
                            if let Some(index) = self.variables.get_index(identifier) {
                                self.asm.push(Asm::GET(index as u32))
                            }
                            if let Some(step) = step {
                                self.compile_expr(step)?;
                                self.asm.push(Asm::IADD);
                            } else {
                                self.asm.push(Asm::INTEGER(1));
                                self.asm.push(Asm::IADD);
                            }
                            if let Some(index) = self.variables.get_index(identifier) {
                                self.asm.push(Asm::STORE(index as u32))
                            }
                            self.asm.push(Asm::BJMP(top));
                            self.asm.push(Asm::LABEL(end));

                            self.breaks.pop();
                            self.continues.pop();
                        }
                    }
                }
                common::nodes::Statement::ForwardDec { identifier, .. } => {
                    //create a wrapper function
                    self.global.insert(identifier.clone());
                }
                common::nodes::Statement::WhileLet {
                    identifier,
                    expr,
                    body,
                } => {
                    let top = self.gen.generate();
                    let end = self.gen.generate();
                    let next = self.gen.generate();
                    self.breaks.push(end);
                    self.continues.push(next);
                    self.asm.push(Asm::LABEL(top));
                    self.compile_expr(expr)?;
                    if let Some(index) = self.variables.get_index(identifier) {
                        self.asm.push(Asm::STORE(index as u32))
                    } else {
                        self.variables.insert(identifier.clone());
                        let index = self.variables.len() - 1;
                        self.asm.push(Asm::STORE(index as u32))
                    }
                    // get the value
                    if let Some(index) = self.variables.get_index(identifier) {
                        self.asm.push(Asm::GET(index as u32))
                    }
                    if let Some(index) = self.native_functions.get_index("Option::isSome") {
                        self.asm.push(Asm::NATIVE(index as u64, None))
                    }
                    self.asm.push(Asm::ISSOME);
                    self.asm.push(Asm::JUMPIFFALSE(end));
                    let whilebody = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(
                        whilebody,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        false,
                    )?;
                    self.asm.pop();
                    self.asm.push(Asm::BJMP(top));
                    self.asm.push(Asm::LABEL(end));
                    self.breaks.pop();
                    self.continues.pop();
                }
                common::nodes::Statement::ValueMatch {
                    expr,
                    arms,
                    default,
                    position,
                    ..
                } => {
                    self.compile_value_match_statement(expr, arms, default, position)?;
                }
            }
        }

        match (function, alloc) {
            (true, _) => {}
            (false, true) => {
                self.asm
                    .insert(0, Asm::ALLOCLOCALS(self.variables.len() as u32));
            }
            _ => {}
        }

        if global {
            //println!("global len {}", self.global.len());
            let mut package = vec![Asm::ALLOCGLOBBALS(self.global.len() as u32)];

            for s in self.global_strings.iter() {
                //println!("s0 {} s1 {}", s.0, s.1);
                package.push(Asm::STRING(s.0.clone()));
                package.push(Asm::STOREGLOBAL(*s.1 as u32));
            }

            package.extend_from_slice(&self.asm);
            self.asm = package;
        }

        self.asm.push(Asm::RET(false));
        Ok(self.asm.to_owned())
    }

    pub fn getref_expr(&mut self, expr: &Expr) -> NovaResult<()> {
        match expr {
            Expr::None => {
                // self.output.push(Code::NONE)
            }
            Expr::ListConstructor { .. } => todo!(),
            Expr::Field { index, expr, .. } => {
                // dbg!(id, t);
                self.asm.push(Asm::INTEGER(*index as i64));
                self.compile_expr(expr)?;
                // self.asm.push(Asm::PIN(position.clone()));
            }
            Expr::Indexed {
                container, index, ..
            } => {
                self.compile_expr(index)?;
                self.compile_expr(container)?;
            }
            Expr::Call { .. } => todo!(),
            Expr::Unary { .. } => todo!(),
            Expr::Binop { .. } => todo!(),
            Expr::Literal { value, .. } => {
                self.getref_atom(value)?;
            }
            Expr::Closure { .. } => todo!(),
            Expr::ListCompConstructor { .. } => todo!(),
            Expr::Sliced { .. } => todo!(),
            Expr::StoreExpr { .. } => todo!(),
            Expr::Return { .. } => todo!(),
            Expr::IfExpr { .. } => todo!(),
            Expr::MatchExpr { .. } => todo!(),
            Expr::ValueMatchExpr { .. } => todo!(),
            Expr::Block { .. } => todo!(),
            Expr::Let { .. } => todo!(),
            Expr::LetDestructure { .. } => todo!(),
            Expr::DynField {
                name,
                expr,
                position,
                ..
            } => {
                self.compile_string_literal(name);
                self.compile_expr(expr)?;
                self.asm.push(Asm::PINF(position.clone()));
            }
            Expr::Void => {}
        }
        Ok(())
    }

    pub fn getref_atom(&mut self, atom: &Atom) -> NovaResult<()> {
        match atom {
            Atom::Bool { value } => {
                self.asm.push(Asm::BOOL(*value));
            }
            Atom::Id { name } => {
                if let Some(index) = self.variables.get_index(name) {
                    self.asm.push(Asm::INTEGER(index as i64));
                } else {
                    self.variables.insert(name.clone());
                    let index = self.variables.len() - 1;
                    self.asm.push(Asm::INTEGER(index as i64));
                }
            }
            Atom::Float { value: float } => {
                self.asm.push(Asm::FLOAT(*float));
            }
            Atom::String { value: str } => {
                let index = self.insert_string_global(str.clone());
                self.asm.push(Asm::GETGLOBAL(index as u32));
            }
            Atom::Integer { value: int } => {
                self.asm.push(Asm::INTEGER(*int));
            }
            Atom::Call {
                name, arguments, ..
            } => {
                for expr in arguments {
                    self.compile_expr(expr)?;
                }
                match name.deref() {
                    "print" => self.asm.push(Asm::PRINT),
                    "free" => self.asm.push(Asm::FREE),
                    "clone" => self.asm.push(Asm::CLONE),
                    identifier => {
                        if let Some(index) = self.variables.get_index(identifier) {
                            self.asm.push(Asm::GET(index as u32));
                            self.asm.push(Asm::CALL);
                        } else if let Some(index) = self.global.get_index(identifier) {
                            self.asm.push(Asm::DCALL(index as u32));
                        } else {
                            dbg!(identifier);
                            todo!()
                        }
                    }
                }
            }
            Atom::Char { .. } => todo!(),
            Atom::None => todo!(),
        }
        Ok(())
    }

    pub fn compile_expr(&mut self, expr: &Expr) -> NovaResult<()> {
        match expr {
            Expr::None => {
                //    Ok(self.output.push(Code::NONE))
                Ok(())
            }
            Expr::ListConstructor { elements, .. } => {
                for x in elements.iter() {
                    self.compile_expr(x)?;
                }
                self.asm.push(Asm::LIST(elements.len() as u64));
                Ok(())
            }
            Expr::Field {
                index,
                expr,
                position,
                ..
            } => {
                self.asm.push(Asm::INTEGER(*index as i64));
                self.compile_expr(expr)?;
                self.asm.push(Asm::LIN(position.clone()));
                Ok(())
            }
            Expr::DynField {
                name,
                expr,
                position,
                ..
            } => {
                self.compile_expr(expr)?;
                self.compile_string_literal(name);
                self.asm.push(Asm::GETF(position.clone()));
                Ok(())
            }
            Expr::Indexed {
                container,
                index,
                position,
                ..
            } => {
                self.compile_expr(index)?;
                self.compile_expr(container)?;
                self.asm.push(Asm::LIN(position.clone()));
                Ok(())
            }
            Expr::Call { function, args, .. } => {
                for e in args.iter() {
                    self.compile_expr(e)?;
                }
                self.compile_expr(function)?;
                self.asm.push(Asm::CALL);
                Ok(())
            }
            Expr::Unary { op, expr, .. } => match op {
                common::tokens::Unary::Positive => {
                    self.compile_expr(expr)?;
                    Ok(())
                }
                common::tokens::Unary::Negative => {
                    self.compile_expr(expr)?;
                    self.asm.push(Asm::NEG);
                    Ok(())
                }
                common::tokens::Unary::Not => {
                    self.compile_expr(expr)?;
                    self.asm.push(Asm::NOT);
                    Ok(())
                }
            },
            Expr::Binop {
                ttype,
                op,
                lhs,
                rhs,
            } => {
                match op {
                    common::tokens::Operator::RightArrow => todo!(),
                    common::tokens::Operator::Greater => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IGTR);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FGTR);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Less => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::ILSS);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FLSS);
                        } else {
                            todo!();
                        }
                    }
                    common::tokens::Operator::Assignment => {
                        self.compile_expr(rhs)?;
                        self.getref_expr(lhs)?;
                        match **lhs {
                            Expr::Field { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            Expr::Indexed { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            Expr::DynField { .. } => {
                                // PINF is already emitted inside getref_expr
                            }
                            _ => self.asm.push(Asm::ASSIGN),
                        }
                    }
                    common::tokens::Operator::Addition => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        match lhs.get_type() {
                            TType::Int => self.asm.push(Asm::IADD),
                            TType::Float => self.asm.push(Asm::FADD),
                            TType::String => self.asm.push(Asm::CONCAT),
                            TType::List { .. } => self.asm.push(Asm::CONCAT),
                            _ => {
                                dbg!(&ttype);
                            }
                        }
                    }
                    common::tokens::Operator::Subtraction => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::ISUB);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FSUB);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Division => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IDIV);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FDIV);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Multiplication => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IMUL);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FMUL);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Equal => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::EQUALS);
                    }
                    common::tokens::Operator::Access => todo!(),
                    common::tokens::Operator::ListAccess => todo!(),
                    common::tokens::Operator::Call => todo!(),
                    common::tokens::Operator::Modulo => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::IMODULO);
                    }
                    common::tokens::Operator::NotEqual => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::NOT);
                    }
                    common::tokens::Operator::Not => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::NOT);
                    }
                    common::tokens::Operator::DoubleColon => todo!(),
                    common::tokens::Operator::Colon => todo!(),
                    common::tokens::Operator::GreaterOrEqual => {
                        let sc = self.gen.generate();

                        // if lhs is true, return its value
                        // else return the other value
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IGTR);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FGTR);
                        } else {
                            dbg!(&ttype);
                        }
                        self.asm.push(Asm::DUP);
                        self.asm.push(Asm::NOT);
                        self.asm.push(Asm::JUMPIFFALSE(sc));
                        self.asm.push(Asm::POP);
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::LessOrEqual => {
                        let sc = self.gen.generate();

                        // if lhs is true, return its value
                        // else return the other value
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::ILSS);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FLSS);
                        } else {
                            dbg!(&ttype);
                        }
                        self.asm.push(Asm::DUP);
                        self.asm.push(Asm::NOT);
                        self.asm.push(Asm::JUMPIFFALSE(sc));
                        self.asm.push(Asm::POP);
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::And => {
                        let sc = self.gen.generate();

                        // if lhs is false, return its value
                        // else return other value
                        self.compile_expr(lhs)?;
                        self.asm.push(Asm::DUP);
                        self.asm.push(Asm::JUMPIFFALSE(sc));
                        self.asm.push(Asm::POP);
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::Or => {
                        let sc = self.gen.generate();
                        // if lhs is true, return its value
                        // else return the other value
                        self.compile_expr(lhs)?;
                        self.asm.push(Asm::DUP);
                        self.asm.push(Asm::NOT);
                        self.asm.push(Asm::JUMPIFFALSE(sc));
                        self.asm.push(Asm::POP);
                        self.compile_expr(rhs)?;
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::AddAssign => {
                        match lhs.get_type() {
                            TType::Int => {
                                self.compile_expr(rhs)?;
                                self.compile_expr(lhs)?;
                                self.asm.push(Asm::IADD);
                            }
                            TType::Float => {
                                self.compile_expr(rhs)?;
                                self.compile_expr(lhs)?;
                                self.asm.push(Asm::FADD);
                            }
                            TType::String => {
                                self.compile_expr(lhs)?;
                                self.compile_expr(rhs)?;
                                self.asm.push(Asm::CONCAT);
                            }
                            TType::List { .. } => {
                                self.compile_expr(lhs)?;
                                self.compile_expr(rhs)?;
                                self.asm.push(Asm::CONCAT);
                            }
                            _ => {
                                dbg!(&lhs.get_type());
                            }
                        }
                        self.getref_expr(lhs)?;
                        match **lhs {
                            Expr::Field { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            Expr::Indexed { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            _ => self.asm.push(Asm::ASSIGN),
                        }
                    }
                    common::tokens::Operator::SubAssign => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::ISUB);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FSUB);
                        } else {
                            dbg!(&ttype);
                        }
                        self.getref_expr(lhs)?;

                        match **lhs {
                            Expr::Field { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            Expr::Indexed { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            _ => self.asm.push(Asm::ASSIGN),
                        }
                    }
                    common::tokens::Operator::MulAssign => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IMUL);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FMUL);
                        } else {
                            dbg!(&ttype);
                        }
                        self.getref_expr(lhs)?;

                        match **lhs {
                            Expr::Field { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            Expr::Indexed { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            _ => self.asm.push(Asm::ASSIGN),
                        }
                    }
                    common::tokens::Operator::DivAssign => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IDIV);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FDIV);
                        } else {
                            dbg!(&ttype);
                        }
                        self.getref_expr(lhs)?;

                        match **lhs {
                            Expr::Field { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            Expr::Indexed { .. } => {
                                self.asm.push(Asm::PIN(FilePosition {
                                    filepath: None,
                                    line: 0,
                                    col: 0,
                                }));
                            }
                            _ => self.asm.push(Asm::ASSIGN),
                        }
                    }
                    common::tokens::Operator::Concat => {
                        self.compile_expr(lhs)?;
                        self.compile_expr(rhs)?;
                        if lhs.get_type() == TType::String {
                            self.asm.push(Asm::CONCAT);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::RightTilde => todo!(),
                    common::tokens::Operator::LeftTilde => todo!(),
                    common::tokens::Operator::ExclusiveRange => todo!(),
                    common::tokens::Operator::InclusiveRange => todo!(),
                    common::tokens::Operator::FatArrow => todo!(),
                    common::tokens::Operator::PipeArrow => todo!(),
                }
                Ok(())
            }
            Expr::Literal {
                ttype: _,
                value: atom,
            } => self.compile_atom(atom),

            Expr::Closure {
                ttype: _,
                args: parameters,
                body: input,
                captures: captured,
            } => {
                //dbg!(&captured, &self.variables);
                // Clone the current state to prepare for function compilation
                let mut function_compile = self.clone();
                // track all the varaibles in the function list and match the captured list
                // with the function list
                //dbg!(&captured);
                function_compile.variables.clear();
                //dbg!(&function_compile.variables);
                function_compile.asm.clear();
                function_compile.fn_local_names.clear();
                //dbg!(&parameters, &captured);
                // Register parameter names in the function's local variable scope
                for param in parameters.iter() {
                    function_compile.variables.insert(param.identifier.clone());
                }

                // Register captured variables in the function's local variable scope
                for capture in captured.iter() {
                    function_compile.variables.insert(capture.clone());
                }

                // Compile captured variables for the closure
                for captured_var in captured.iter() {
                    //dbg!(&captured);
                    if let Some(index) = self.variables.get_index(captured_var) {
                        // Get the local variable if it exists in the current scope
                        self.asm.push(Asm::GET(index as u32));
                    } else if let Some(index) = self.global.get_index(captured_var) {
                        // Otherwise, get the global variable if it exists
                        self.asm.push(Asm::GETGLOBAL(index as u32));
                    } else {
                        return Err(Box::new(common::error::NovaError::Compiler {
                            msg: format!(
                                "Captured variable '{}' not found in local or global scope",
                                captured_var
                            )
                            .into(),
                            note: format!("The closure captures `{}` but it could not be found. Make sure the variable is defined in an enclosing scope before the closure.", captured_var).into(),
                        }));
                    }
                }
                // Generate a jump label for the closure function
                let closure_jump_label = function_compile.gen.generate();

                // Prepare the closure with a function label or a captured list if necessary
                if captured.is_empty() {
                    self.asm.push(Asm::FUNCTION(closure_jump_label));
                } else {
                    self.asm.push(Asm::INTEGER(captured.len() as i64));
                    self.asm.push(Asm::CLOSURE(closure_jump_label));
                }

                // Compile the function body
                let function_body_ast = Ast {
                    program: input.clone(),
                };
                let _ = function_compile.compile_program(
                    function_body_ast,
                    self.filepath.clone(),
                    true,
                    false,
                    true,
                    false,
                )?;

                // Adjust the function's offset to account for parameters and captured variables
                let num_parameters = parameters.len() as u32;
                let num_captures = captured.len() as u32;
                let local_vars = function_compile.variables.len() as u32;
                self.asm.push(Asm::OFFSET(
                    num_parameters + num_captures,
                    local_vars - (num_parameters + num_captures),
                ));

                // Record per-function local names for debug info
                let fn_locals: Vec<(u32, String)> = function_compile
                    .variables
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (i as u32, name.to_string()))
                    .collect();
                self.fn_local_names.push((closure_jump_label, fn_locals));
                self.fn_local_names.append(&mut function_compile.fn_local_names);

                // Append compiled function instructions to the current scope
                self.gen = function_compile.gen;
                self.global = function_compile.global;
                self.global_strings = function_compile.global_strings;
                function_compile.asm.pop(); // Remove the last instruction from function compilation
                self.asm.extend_from_slice(&function_compile.asm);
                self.asm.push(Asm::LABEL(closure_jump_label));

                Ok(())
            }
            Expr::ListCompConstructor {
                expr,
                guards,
                loops,
                position,
                ..
            } => {
                let mut loops = loops.clone();
                loops.reverse();
                let (identifier, list) = loops.pop().unwrap();
                // create temp list to hold new values

                self.variables
                    .insert(format!("__listexpr__{}", self.gen.generate()).into());
                let list_index = self.variables.len() - 1;
                self.asm.push(Asm::LIST(0));
                self.asm.push(Asm::STORE(list_index as u32));

                self.for_in_loop(
                    identifier,
                    list,
                    expr.clone(),
                    guards.clone(),
                    list_index,
                    loops,
                    position.clone(),
                )?;

                // return the list
                self.asm.push(Asm::GET(list_index as u32));

                Ok(())
            }
            Expr::Sliced {
                container,
                start: startstep,
                end: endstep,
                step: stepstep,
                position,
                ..
            } => {
                let top = self.gen.generate();
                let end = self.gen.generate();

                let mid = self.gen.generate();
                let step = self.gen.generate();

                let next = self.gen.generate();

                let negitive_start = self.gen.generate();
                let negitive_end = self.gen.generate();

                self.breaks.push(end);
                self.continues.push(next);
                // create temp list to hold new values

                self.variables
                    .insert(format!("__listexpr__{}", self.gen.generate()).into());
                let list_index = self.variables.len() - 1;
                self.asm.push(Asm::LIST(0));
                self.asm.push(Asm::STORE(list_index as u32));

                // insert temp counter
                self.variables
                    .insert(format!("__tempcounter__{}", self.gen.generate()).into());
                let tempcounter_index = self.variables.len() - 1;

                self.variables
                    .insert(format!("__arrayexpr__{}", self.gen.generate()).into());
                let array_index = self.variables.len() - 1;

                self.variables
                    .insert(format!("__tempexpr__{}", self.gen.generate()).into());
                let id_index = self.variables.len() - 1;

                // compile list expr
                self.compile_expr(container)?;
                self.asm.push(Asm::STORE(array_index as u32));

                // compiling start as integer
                if let Some(startstep) = startstep {
                    self.compile_expr(startstep)?;
                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::ILSS);
                    self.asm.push(Asm::JUMPIFFALSE(negitive_start));

                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LEN);
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::LABEL(negitive_start));
                } else {
                    self.asm.push(Asm::INTEGER(0));
                }

                self.asm.push(Asm::STORE(tempcounter_index as u32));

                // if array is empty jump to end
                self.asm.push(Asm::LABEL(top));
                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                self.asm.push(Asm::LEN);

                self.asm.push(Asm::IGTR);
                self.asm.push(Asm::DUP);
                self.asm.push(Asm::NOT);

                self.asm.push(Asm::JUMPIFFALSE(step));
                self.asm.push(Asm::POP);

                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                self.asm.push(Asm::LEN);
                self.asm.push(Asm::EQUALS);

                self.asm.push(Asm::LABEL(step));
                self.asm.push(Asm::JUMPIFFALSE(mid));
                self.asm.push(Asm::JMP(end));
                self.asm.push(Asm::LABEL(mid));

                // compile upper bound check
                if let Some(endstep) = endstep {
                    self.compile_expr(endstep)?;

                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::ILSS);
                    self.asm.push(Asm::JUMPIFFALSE(negitive_end));
                    self.asm.push(Asm::GET(array_index as u32));
                    self.asm.push(Asm::LEN);
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::LABEL(negitive_end));

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::IGTR);
                    self.asm.push(Asm::JUMPIFFALSE(end));
                }

                // bind value to identifier for (x in list) // x is identifier
                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                self.asm.push(Asm::LIN(position.clone()));
                self.asm.push(Asm::STORE(id_index as u32));

                // -- expr and then push to temp array
                self.asm.push(Asm::GET(list_index as u32));
                self.asm.push(Asm::GET(id_index as u32));

                if let Some(index) = self.native_functions.get_index("List::push") {
                    self.asm.push(Asm::NATIVE(index as u64, None))
                } else {
                    todo!()
                }

                self.asm.push(Asm::LABEL(next));
                // increment counter
                if let Some(stepstep) = stepstep {
                    self.compile_expr(stepstep)?;
                } else {
                    self.asm.push(Asm::INTEGER(1));
                }
                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::IADD);
                self.asm.push(Asm::INTEGER(tempcounter_index as i64));
                self.asm.push(Asm::ASSIGN);
                self.asm.push(Asm::BJMP(top));
                self.asm.push(Asm::LABEL(end));

                // return the list
                self.asm.push(Asm::GET(list_index as u32));

                self.breaks.pop();
                self.continues.pop();
                Ok(())
            }
            Expr::StoreExpr {
                ttype,
                name,
                expr,
                body,
            } => {
                self.compile_expr(expr)?;
                if let Some(index) = self.variables.get_index(name) {
                    self.asm.push(Asm::STORE(index as u32))
                } else {
                    self.variables.insert(name.clone());
                    let index = self.variables.len() - 1;
                    self.asm.push(Asm::STORE(index as u32))
                }

                let body = Ast {
                    program: body.clone(),
                };
                self.compile_program(body, self.filepath.clone(), false, false, false, false)?;
                self.asm.pop();
                // total hack, happens when last statement gets autopopped when compiling a statement expr
                // so we need to pop it again
                if TType::Void != *ttype {
                    self.asm.pop();
                }
                //self.asm.pop();
                Ok(())
            }
            Expr::Return { expr, .. } => {
                self.compile_expr(expr)?;
                self.asm.push(Asm::RET(true));
                Ok(())
            }
            Expr::IfExpr {
                test,
                body,
                alternative,
                ..
            } => {
                let end = self.gen.generate();
                let next = self.gen.generate();
                self.compile_expr(test)?;
                self.asm.push(Asm::JUMPIFFALSE(next));
                self.compile_expr(body)?;
                self.asm.push(Asm::JMP(end));
                self.asm.push(Asm::LABEL(next));
                self.compile_expr(alternative)?;
                self.asm.push(Asm::LABEL(end));
                Ok(())
            }
            Expr::MatchExpr {
                expr,
                arms,
                default,
                position,
                ..
            } => {
                let end = self.gen.generate();

                self.compile_expr(expr)?;
                // store in temp variable
                self.variables
                    .insert(format!("___matchexpr___{}", self.gen.generate()).into());
                let temp_matchexpr = self.variables.len() - 1;
                self.asm.push(Asm::STORE(temp_matchexpr as u32));

                for arm in arms.iter() {
                    let next = self.gen.generate();
                    self.asm.push(Asm::INTEGER(1_i64));
                    self.asm.push(Asm::GET(temp_matchexpr as u32));
                    self.asm.push(Asm::LIN(position.clone()));
                    self.asm.push(Asm::INTEGER(arm.0 as i64));
                    self.asm.push(Asm::EQUALS);
                    self.asm.push(Asm::JUMPIFFALSE(next));
                    if let Some(vid) = &arm.1 {
                        self.asm.push(Asm::INTEGER(0_i64));
                        self.asm.push(Asm::GET(temp_matchexpr as u32));
                        self.asm.push(Asm::LIN(position.clone()));

                        if let Some(index) = self.variables.get_index(vid) {
                            self.asm.push(Asm::STORE(index as u32))
                        } else {
                            self.variables.insert(vid.clone());
                            let index = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(index as u32))
                        }
                    }

                    let arm_ast = Ast {
                        program: arm.2.clone(),
                    };
                    // keep=true so the arm's result stays on the stack
                    self.compile_program(
                        arm_ast,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        true,
                    )?;
                    self.asm.pop();

                    self.asm.push(Asm::JMP(end));
                    self.asm.push(Asm::LABEL(next));
                }
                if let Some(default) = default {
                    let default_ast = Ast {
                        program: default.clone(),
                    };
                    self.compile_program(
                        default_ast,
                        self.filepath.clone(),
                        false,
                        false,
                        false,
                        true,
                    )?;
                    self.asm.pop();
                }
                self.asm.push(Asm::LABEL(end));
                Ok(())
            }
            Expr::Block { body, .. } => {
                //dbg!(ttype);
                let b = Ast {
                    program: body.clone(),
                };
                self.compile_program(b, self.filepath.clone(), false, false, false, true)?;
                self.asm.pop();
                Ok(())
            }
            Expr::Let {
                identifier,
                expr,
                global,
                ..
            } => {
                self.compile_expr(expr)?;

                // `_` is a discard — just pop the value off the stack.
                if identifier.as_ref() == "_" {
                    self.asm.push(Asm::POP);
                    return Ok(());
                }

                if *global {
                    if let Some(index) = self.global.get_index(identifier) {
                        self.asm.push(Asm::STOREGLOBAL(index as u32))
                    } else {
                        self.global.insert(identifier.clone());
                        let index = self.global.len() - 1;
                        self.asm.push(Asm::STOREGLOBAL(index as u32))
                    }
                } else if let Some(index) = self.variables.get_index(identifier) {
                    self.asm.push(Asm::STORE(index as u32))
                } else {
                    self.variables.insert(identifier.clone());
                    let index = self.variables.len() - 1;
                    self.asm.push(Asm::STORE(index as u32))
                }
                Ok(())
            }
            Expr::LetDestructure {
                pattern,
                expr,
                position,
                ..
            } => {
                self.compile_expr(expr)?;
                // Store the expression result in a temp variable
                self.variables
                    .insert(format!("___letdestr___{}", self.gen.generate()).into());
                let temp = self.variables.len() - 1;
                self.asm.push(Asm::STORE(temp as u32));
                // Emit bindings using the same machinery as value-match
                self.compile_pattern_bindings(pattern, temp, position)?;
                Ok(())
            }
            Expr::Void => Ok(()),
            Expr::ValueMatchExpr {
                expr,
                arms,
                default,
                position,
                ..
            } => {
                self.compile_value_match_expr(expr, arms, default, position)
            }
        }
    }

    // needs to be recursive function
    #[allow(clippy::too_many_arguments)]
    fn for_in_loop(
        &mut self,
        identifier: Rc<str>,
        list: Expr,
        expr: Vec<Expr>,
        guards: Vec<Expr>,
        list_index: usize,
        mut loops: Vec<(Rc<str>, Expr)>,
        position: FilePosition,
    ) -> NovaResult<()> {
        // compile list and create counter
        self.variables
            .insert(format!("__tempcounter__{}", self.gen.generate()).into());
        let tempcounter_index = self.variables.len() - 1;
        self.variables
            .insert(format!("__arrayexpr__{}", self.gen.generate()).into());
        let array_index = self.variables.len() - 1;
        let id_index = if let Some(index) = self.variables.get_index(&*identifier) {
            index
        } else {
            self.variables.insert(identifier);
            self.variables.len() - 1
        };
        // generate labels
        let top = self.gen.generate();
        let end = self.gen.generate();
        let mid = self.gen.generate();
        let step = self.gen.generate();
        let next = self.gen.generate();
        self.breaks.push(end);
        self.continues.push(next);

        // compile list expr
        self.compile_expr(&list)?;
        self.asm.push(Asm::STORE(array_index as u32));
        self.asm.push(Asm::INTEGER(0));
        self.asm.push(Asm::STORE(tempcounter_index as u32));
        self.asm.push(Asm::LABEL(top));
        self.asm.push(Asm::GET(tempcounter_index as u32));
        self.asm.push(Asm::GET(array_index as u32));
        self.asm.push(Asm::LEN);
        self.asm.push(Asm::IGTR);
        self.asm.push(Asm::DUP);
        self.asm.push(Asm::NOT);
        self.asm.push(Asm::JUMPIFFALSE(step));
        self.asm.push(Asm::POP);
        self.asm.push(Asm::GET(tempcounter_index as u32));
        self.asm.push(Asm::GET(array_index as u32));
        self.asm.push(Asm::LEN);
        self.asm.push(Asm::EQUALS);
        self.asm.push(Asm::LABEL(step));
        self.asm.push(Asm::JUMPIFFALSE(mid));
        self.asm.push(Asm::JMP(end));
        self.asm.push(Asm::LABEL(mid));

        // bind value from list to identifier
        self.asm.push(Asm::GET(tempcounter_index as u32));
        self.asm.push(Asm::GET(array_index as u32));
        self.asm.push(Asm::LIN(position.clone()));
        self.asm.push(Asm::STORE(id_index as u32));
        let nextloop = loops.pop();
        if let Some((identifier, list)) = nextloop {
            self.for_in_loop(identifier, list, expr, guards, list_index, loops, position)?;
        } else {
            // create new label for guards
            let skipcurrent = self.gen.generate();
            // check guards
            for guard in guards.iter() {
                self.compile_expr(guard)?;
                self.asm.push(Asm::JUMPIFFALSE(skipcurrent));
            }

            // -- expr and then push to temp array
            for expr in expr.iter() {
                self.compile_expr(expr)?;
            }
            self.asm.push(Asm::STORE(id_index as u32));

            self.asm.push(Asm::GET(list_index as u32));
            self.asm.push(Asm::GET(id_index as u32));
            if let Some(index) = self.native_functions.get_index("List::push") {
                self.asm.push(Asm::NATIVE(index as u64, None))
            } else {
                todo!()
            }

            // skip current label
            self.asm.push(Asm::LABEL(skipcurrent));
        }

        // increment counter
        self.asm.push(Asm::LABEL(next));
        self.asm.push(Asm::INTEGER(1));
        self.asm.push(Asm::GET(tempcounter_index as u32));
        self.asm.push(Asm::IADD);
        self.asm.push(Asm::INTEGER(tempcounter_index as i64));
        self.asm.push(Asm::ASSIGN);
        self.asm.push(Asm::BJMP(top));
        self.asm.push(Asm::LABEL(end));

        self.breaks.pop();
        self.continues.pop();
        Ok(())
    }

    pub fn compile_atom(&mut self, atom: &Atom) -> NovaResult<()> {
        match atom {
            Atom::Bool { value: bool } => {
                self.asm.push(Asm::BOOL(*bool));
            }
            Atom::Id { name: identifier } => {
                if let Some(index) = self.variables.get_index(identifier) {
                    self.asm.push(Asm::GET(index as u32));
                } else if let Some(index) = self.global.get_index(identifier) {
                    self.asm.push(Asm::GETGLOBAL(index as u32));
                } else if let Some(value) = self.unrolled_index.get(identifier) {
                    self.asm.push(Asm::INTEGER(*value as i64));
                } else {
                    return Err(Box::new(NovaError::Compiler {
                        msg: format!("Function \"{}\" not found", identifier).into(),
                        note: format!("The identifier `{}` could not be resolved to a local variable, global variable, or function.\n  Make sure it is defined before this point in the code.", identifier).into(),
                    }));
                }
            }
            Atom::Float { value: float } => {
                self.asm.push(Asm::FLOAT(*float));
            }
            Atom::String { value: str } => {
                let index = self.insert_string_global(str.clone());
                // clone string to prevent mutation
                self.asm.push(Asm::GETGLOBAL(index as u32));
                self.asm.push(Asm::CLONE);
            }
            Atom::Integer { value: int } => {
                self.asm.push(Asm::INTEGER(*int));
            }
            Atom::Call {
                name: caller,
                arguments: list,
                position,
            } => {
                // println!("Call: {}", caller);
                if caller.deref() == "typeof" {
                    self.compile_string_literal(&list[0].get_type().to_string());
                    return Ok(());
                }
                for expr in list {
                    self.compile_expr(expr)?;
                }
                match caller.deref() {
                    "unreachable" => self.asm.push(Asm::ERROR(position.clone())),
                    "todo" => {
                        // show a panic message before exiting
                        self.compile_string_literal("Not yet implemented");
                        self.asm.push(Asm::PRINT);
                        self.asm.push(Asm::ERROR(position.clone()));
                    }
                    "None" => self.asm.push(Asm::NONE),
                    "Option::unwrap" => self.asm.push(Asm::UNWRAP(position.clone())),
                    "Some" => {}
                    "Option::isSome" => self.asm.push(Asm::ISSOME),
                    "free" => self.asm.push(Asm::FREE),
                    "clone" => self.asm.push(Asm::CLONE),
                    "exit" => self.asm.push(Asm::EXIT),
                    "error" => self.asm.push(Asm::ERROR(position.clone())),
                    "List::len" | "String::len" => self.asm.push(Asm::LEN),
                    identifier => {
                        //dbg!(identifier);
                        if let Some(index) = self.native_functions.get_index(identifier) {
                            self.asm.push(Asm::NATIVE(index as u64, Some(position.clone())));
                        } else if let Some(index) = self.variables.get_index(identifier) {
                            self.asm.push(Asm::GET(index as u32));
                            self.asm.push(Asm::CALL);
                        } else if let Some(index) = self.global.get_index(identifier) {
                            //dbg!(identifier, &index);
                            if "println" == identifier || "print" == identifier {
                                // look for toString function
                                let typelist =
                                    list.iter().map(|x| x.get_type()).collect::<Vec<TType>>();
                                //dbg!(typelist[0].custom_to_string());
                                if let Some(firsttype) = typelist[0].custom_to_string() {
                                    //dbg!(firsttype);
                                    if let Some(index) = self.global.get_index(
                                        format!("{}::toString_{}", firsttype, firsttype).as_str(),
                                    ) {
                                        self.asm.push(Asm::DCALL(index as u32));
                                        // handles generic toString for datatypes
                                    } else if let Some(index) = self
                                        .global
                                        .get_index(format!("{}::toString", firsttype).as_str())
                                    {
                                        self.asm.push(Asm::DCALL(index as u32));
                                    }
                                }
                            }

                            self.asm.push(Asm::DCALL(index as u32));
                        } else if let Some(value) = self.unrolled_index.get(identifier) {
                            self.asm.push(Asm::INTEGER(*value as i64));
                        } else {
                            return Err(Box::new(NovaError::Compiler {
                                msg: format!("Function \"{}\" not found", identifier).into(),
                                note: format!("The function `{}` could not be found as a native function, local variable, or global definition.\n  Make sure it is defined or imported before this call.", identifier).into(),
                            }));
                        }
                    }
                }
            }
            Atom::Char { value: c } => self.asm.push(Asm::Char(*c)),
            Atom::None => self.asm.push(Asm::NONE),
        }
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────
    //  Generalized pattern matching compilation
    // ─────────────────────────────────────────────────────────────────

    /// Emit code that tests whether the value on top of the stack (already
    /// stored in `temp`) matches `pattern`.  Pushes `Bool` on the stack.
    /// For patterns that bind variables, the bindings are emitted as stores.
    fn compile_pattern_test(
        &mut self,
        pattern: &Pattern,
        temp: usize,
        position: &FilePosition,
    ) -> NovaResult<()> {
        use common::nodes::Pattern::*;
        match pattern {
            Wildcard => {
                // always matches
                self.asm.push(Asm::BOOL(true));
            }
            Variable(_name) => {
                // always matches (binding happens separately after the test succeeds)
                self.asm.push(Asm::BOOL(true));
            }
            IntLiteral(v) => {
                self.asm.push(Asm::INTEGER(*v));
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::EQUALS);
            }
            FloatLiteral(v) => {
                self.asm.push(Asm::FLOAT(*v));
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::EQUALS);
            }
            StringLiteral(v) => {
                let idx = self.insert_string_global(v.clone());
                self.asm.push(Asm::GETGLOBAL(idx as u32));
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::EQUALS);
            }
            BoolLiteral(v) => {
                self.asm.push(Asm::BOOL(*v));
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::EQUALS);
            }
            CharLiteral(v) => {
                self.asm.push(Asm::Char(*v));
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::EQUALS);
            }
            EmptyList => {
                // Check that length == 0
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::LEN);
                self.asm.push(Asm::INTEGER(0));
                self.asm.push(Asm::EQUALS);
            }
            List(pats) => {
                // Check length == pats.len(), then check each element
                let fail = self.gen.generate();
                let ok = self.gen.generate();

                // length check
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::LEN);
                self.asm.push(Asm::INTEGER(pats.len() as i64));
                self.asm.push(Asm::EQUALS);
                self.asm.push(Asm::JUMPIFFALSE(fail));

                // check each element
                for (i, pat) in pats.iter().enumerate() {
                    match pat {
                        Wildcard | Variable(_) => {
                            // no test needed for wildcards/variables
                        }
                        _ => {
                            // store element in a sub-temp
                            self.asm.push(Asm::INTEGER(i as i64));
                            self.asm.push(Asm::GET(temp as u32));
                            self.asm.push(Asm::LIN(position.clone()));
                            self.variables.insert(
                                format!("___matchelem___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_test(pat, sub_temp, position)?;
                            self.asm.push(Asm::JUMPIFFALSE(fail));
                        }
                    }
                }

                self.asm.push(Asm::BOOL(true));
                self.asm.push(Asm::JMP(ok));
                self.asm.push(Asm::LABEL(fail));
                self.asm.push(Asm::BOOL(false));
                self.asm.push(Asm::LABEL(ok));
            }
            ListCons(head_pats, _tail_name) => {
                // Check length >= head_pats.len(), then check head elements
                let fail = self.gen.generate();
                let ok = self.gen.generate();

                // length check: len >= head_pats.len()
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::LEN);
                self.asm.push(Asm::INTEGER(head_pats.len() as i64));
                // len >= n  is  !(len < n)
                self.asm.push(Asm::ILSS);
                self.asm.push(Asm::NOT);
                self.asm.push(Asm::JUMPIFFALSE(fail));

                // check each head element
                for (i, pat) in head_pats.iter().enumerate() {
                    match pat {
                        Wildcard | Variable(_) => {}
                        _ => {
                            self.asm.push(Asm::INTEGER(i as i64));
                            self.asm.push(Asm::GET(temp as u32));
                            self.asm.push(Asm::LIN(position.clone()));
                            self.variables.insert(
                                format!("___matchelem___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_test(pat, sub_temp, position)?;
                            self.asm.push(Asm::JUMPIFFALSE(fail));
                        }
                    }
                }

                self.asm.push(Asm::BOOL(true));
                self.asm.push(Asm::JMP(ok));
                self.asm.push(Asm::LABEL(fail));
                self.asm.push(Asm::BOOL(false));
                self.asm.push(Asm::LABEL(ok));
            }
            Tuple(pats) => {
                // same as list: check length == pats.len(), then each element
                let fail = self.gen.generate();
                let ok = self.gen.generate();

                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::LEN);
                self.asm.push(Asm::INTEGER(pats.len() as i64));
                self.asm.push(Asm::EQUALS);
                self.asm.push(Asm::JUMPIFFALSE(fail));

                for (i, pat) in pats.iter().enumerate() {
                    match pat {
                        Wildcard | Variable(_) => {}
                        _ => {
                            self.asm.push(Asm::INTEGER(i as i64));
                            self.asm.push(Asm::GET(temp as u32));
                            self.asm.push(Asm::LIN(position.clone()));
                            self.variables.insert(
                                format!("___matchelem___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_test(pat, sub_temp, position)?;
                            self.asm.push(Asm::JUMPIFFALSE(fail));
                        }
                    }
                }

                self.asm.push(Asm::BOOL(true));
                self.asm.push(Asm::JMP(ok));
                self.asm.push(Asm::LABEL(fail));
                self.asm.push(Asm::BOOL(false));
                self.asm.push(Asm::LABEL(ok));
            }
            Or(alternatives) => {
                // Try each alternative; if any matches, the whole Or matches.
                let ok = self.gen.generate();
                let fail = self.gen.generate();

                for alt in alternatives.iter() {
                    self.compile_pattern_test(alt, temp, position)?;
                    // If true, jump to ok
                    let next = self.gen.generate();
                    self.asm.push(Asm::JUMPIFFALSE(next));
                    self.asm.push(Asm::JMP(ok));
                    self.asm.push(Asm::LABEL(next));
                }

                // none matched
                self.asm.push(Asm::BOOL(false));
                self.asm.push(Asm::JMP(fail));
                self.asm.push(Asm::LABEL(ok));
                self.asm.push(Asm::BOOL(true));
                self.asm.push(Asm::LABEL(fail));
            }
            Enum { variant, binding, tag } => {
                // Enums are stored as [value, tag, type_name] with LIST(3).
                // Tag is an integer index matching the variant's position in the enum definition.
                let fail = self.gen.generate();
                let ok = self.gen.generate();

                // Get the tag (index 1)
                self.asm.push(Asm::INTEGER(1));
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::LIN(position.clone()));

                // Compare with the variant's tag index (integer)
                if let Some(tag_idx) = tag {
                    self.asm.push(Asm::INTEGER(*tag_idx as i64));
                } else {
                    // Fallback: compare by string name (shouldn't happen after resolve)
                    let idx = self.insert_string_global(variant.clone());
                    self.asm.push(Asm::GETGLOBAL(idx as u32));
                }
                self.asm.push(Asm::EQUALS);
                self.asm.push(Asm::JUMPIFFALSE(fail));

                // If binding has a sub-pattern (not just a variable/wildcard), test it too
                if let Some(sub_pat) = binding {
                    match sub_pat.as_ref() {
                        Wildcard | Variable(_) => {
                            // no extra test needed
                        }
                        _ => {
                            // Get the value (index 0) and test sub-pattern
                            self.asm.push(Asm::INTEGER(0));
                            self.asm.push(Asm::GET(temp as u32));
                            self.asm.push(Asm::LIN(position.clone()));
                            self.variables.insert(
                                format!("___matchenumval___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_test(sub_pat, sub_temp, position)?;
                            self.asm.push(Asm::JUMPIFFALSE(fail));
                        }
                    }
                }

                self.asm.push(Asm::BOOL(true));
                self.asm.push(Asm::JMP(ok));
                self.asm.push(Asm::LABEL(fail));
                self.asm.push(Asm::BOOL(false));
                self.asm.push(Asm::LABEL(ok));
            }
            Struct { name: _, fields } => {
                // Structs are stored as objects. Fields are indexed by position.
                // We need to check each field pattern.
                let fail = self.gen.generate();
                let ok = self.gen.generate();

                for (i, (_field_name, pat)) in fields.iter().enumerate() {
                    match pat {
                        Wildcard | Variable(_) => {}
                        _ => {
                            self.asm.push(Asm::INTEGER(i as i64));
                            self.asm.push(Asm::GET(temp as u32));
                            self.asm.push(Asm::LIN(position.clone()));
                            self.variables.insert(
                                format!("___matchfield___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_test(pat, sub_temp, position)?;
                            self.asm.push(Asm::JUMPIFFALSE(fail));
                        }
                    }
                }

                self.asm.push(Asm::BOOL(true));
                self.asm.push(Asm::JMP(ok));
                self.asm.push(Asm::LABEL(fail));
                self.asm.push(Asm::BOOL(false));
                self.asm.push(Asm::LABEL(ok));
            }
            OptionSome(binding) => {
                // Option values: Some(x) is just the value on stack, None is VmData::None.
                // ISSOME returns true for anything that isn't VmData::None.
                let fail = self.gen.generate();
                let ok = self.gen.generate();

                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::ISSOME);
                self.asm.push(Asm::JUMPIFFALSE(fail));

                // If binding has a sub-pattern beyond variable/wildcard, test it
                if let Some(sub_pat) = binding {
                    match sub_pat.as_ref() {
                        Wildcard | Variable(_) => {}
                        _ => {
                            // The value itself is the payload (no indexing needed)
                            self.compile_pattern_test(sub_pat, temp, position)?;
                            self.asm.push(Asm::JUMPIFFALSE(fail));
                        }
                    }
                }

                self.asm.push(Asm::BOOL(true));
                self.asm.push(Asm::JMP(ok));
                self.asm.push(Asm::LABEL(fail));
                self.asm.push(Asm::BOOL(false));
                self.asm.push(Asm::LABEL(ok));
            }
            OptionNone => {
                // None is VmData::None. ISSOME returns false for None.
                self.asm.push(Asm::GET(temp as u32));
                self.asm.push(Asm::ISSOME);
                self.asm.push(Asm::NOT);
            }
        }
        Ok(())
    }

    /// Emit variable bindings for a pattern after it has matched.
    fn compile_pattern_bindings(
        &mut self,
        pattern: &Pattern,
        temp: usize,
        position: &FilePosition,
    ) -> NovaResult<()> {
        use common::nodes::Pattern::*;
        match pattern {
            Wildcard => {}
            Variable(name) => {
                // bind the whole value
                self.asm.push(Asm::GET(temp as u32));
                if let Some(index) = self.variables.get_index(name) {
                    self.asm.push(Asm::STORE(index as u32));
                } else {
                    self.variables.insert(name.clone());
                    let index = self.variables.len() - 1;
                    self.asm.push(Asm::STORE(index as u32));
                }
            }
            IntLiteral(_) | FloatLiteral(_) | StringLiteral(_) | BoolLiteral(_)
            | CharLiteral(_) | EmptyList => {
                // no bindings for literals
            }
            List(pats) => {
                for (i, pat) in pats.iter().enumerate() {
                    if matches!(pat, Wildcard) {
                        continue;
                    }
                    self.asm.push(Asm::INTEGER(i as i64));
                    self.asm.push(Asm::GET(temp as u32));
                    self.asm.push(Asm::LIN(position.clone()));
                    match pat {
                        Variable(name) => {
                            if let Some(index) = self.variables.get_index(name) {
                                self.asm.push(Asm::STORE(index as u32));
                            } else {
                                self.variables.insert(name.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32));
                            }
                        }
                        _ => {
                            // nested pattern: store in a sub-temp and recurse
                            self.variables.insert(
                                format!("___matchbind___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_bindings(pat, sub_temp, position)?;
                        }
                    }
                }
            }
            ListCons(head_pats, tail_name) => {
                // bind head elements
                for (i, pat) in head_pats.iter().enumerate() {
                    if matches!(pat, Wildcard) {
                        continue;
                    }
                    self.asm.push(Asm::INTEGER(i as i64));
                    self.asm.push(Asm::GET(temp as u32));
                    self.asm.push(Asm::LIN(position.clone()));
                    match pat {
                        Variable(name) => {
                            if let Some(index) = self.variables.get_index(name) {
                                self.asm.push(Asm::STORE(index as u32));
                            } else {
                                self.variables.insert(name.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32));
                            }
                        }
                        _ => {
                            self.variables.insert(
                                format!("___matchbind___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_bindings(pat, sub_temp, position)?;
                        }
                    }
                }
                // bind tail: slice from head_pats.len()..
                // We need to use native slice for this
                // Push start index, then None for end, then the list
                // Use the SLICEFROM approach or emit a call
                // Actually we can use: list[head_len..] which is a slice
                self.asm.push(Asm::INTEGER(head_pats.len() as i64));
                self.asm.push(Asm::GET(temp as u32));
                // We need a way to slice. Let's use the native function if available.
                // Actually, let's look at how slicing works in the VM.
                // For now emit a native call to List::__tail_slice
                if let Some(index) = self.native_functions.get_index("List::__tail_slice") {
                    self.asm.push(Asm::NATIVE(index as u64, None));
                }
                if let Some(index) = self.variables.get_index(tail_name) {
                    self.asm.push(Asm::STORE(index as u32));
                } else {
                    self.variables.insert(tail_name.clone());
                    let index = self.variables.len() - 1;
                    self.asm.push(Asm::STORE(index as u32));
                }
            }
            Tuple(pats) => {
                for (i, pat) in pats.iter().enumerate() {
                    if matches!(pat, Wildcard) {
                        continue;
                    }
                    self.asm.push(Asm::INTEGER(i as i64));
                    self.asm.push(Asm::GET(temp as u32));
                    self.asm.push(Asm::LIN(position.clone()));
                    match pat {
                        Variable(name) => {
                            if let Some(index) = self.variables.get_index(name) {
                                self.asm.push(Asm::STORE(index as u32));
                            } else {
                                self.variables.insert(name.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32));
                            }
                        }
                        _ => {
                            self.variables.insert(
                                format!("___matchbind___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_bindings(pat, sub_temp, position)?;
                        }
                    }
                }
            }
            Or(alternatives) => {
                // Or patterns cannot bind variables (enforced at parse time).
                // But if the first alternative has bindings, use it.
                // For safety, we just try the first alternative.
                if let Some(first) = alternatives.first() {
                    self.compile_pattern_bindings(first, temp, position)?;
                }
            }
            Enum { variant: _, binding, .. } => {
                // Bind the payload (index 0 of the enum list [value, tag, type_name])
                if let Some(sub_pat) = binding {
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::GET(temp as u32));
                    self.asm.push(Asm::LIN(position.clone()));
                    match sub_pat.as_ref() {
                        Variable(name) => {
                            if let Some(index) = self.variables.get_index(name) {
                                self.asm.push(Asm::STORE(index as u32));
                            } else {
                                self.variables.insert(name.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32));
                            }
                        }
                        Wildcard => {
                            // discard
                            self.asm.pop(); // pop the LIN
                            self.asm.pop(); // pop the GET
                            self.asm.pop(); // pop the INTEGER
                        }
                        _ => {
                            self.variables.insert(
                                format!("___matchenumbind___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_bindings(sub_pat.as_ref(), sub_temp, position)?;
                        }
                    }
                }
            }
            Struct { name: _, fields } => {
                // Bind field patterns by their index in the struct
                for (i, (_field_name, pat)) in fields.iter().enumerate() {
                    if matches!(pat, Wildcard) {
                        continue;
                    }
                    self.asm.push(Asm::INTEGER(i as i64));
                    self.asm.push(Asm::GET(temp as u32));
                    self.asm.push(Asm::LIN(position.clone()));
                    match pat {
                        Variable(name) => {
                            if let Some(index) = self.variables.get_index(name) {
                                self.asm.push(Asm::STORE(index as u32));
                            } else {
                                self.variables.insert(name.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32));
                            }
                        }
                        _ => {
                            self.variables.insert(
                                format!("___matchfieldbind___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_bindings(pat, sub_temp, position)?;
                        }
                    }
                }
            }
            OptionSome(binding) => {
                // Option Some: the value IS the payload (no indexing into a list)
                if let Some(sub_pat) = binding {
                    // Bind the value directly from temp (it's the payload itself)
                    match sub_pat.as_ref() {
                        Variable(name) => {
                            self.asm.push(Asm::GET(temp as u32));
                            if let Some(index) = self.variables.get_index(name) {
                                self.asm.push(Asm::STORE(index as u32));
                            } else {
                                self.variables.insert(name.clone());
                                let index = self.variables.len() - 1;
                                self.asm.push(Asm::STORE(index as u32));
                            }
                        }
                        Wildcard => {
                            // discard
                        }
                        _ => {
                            // nested pattern: store temp value and recurse
                            self.asm.push(Asm::GET(temp as u32));
                            self.variables.insert(
                                format!("___matchoptbind___{}", self.gen.generate()).into(),
                            );
                            let sub_temp = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(sub_temp as u32));
                            self.compile_pattern_bindings(sub_pat.as_ref(), sub_temp, position)?;
                        }
                    }
                }
            }
            OptionNone => {
                // No bindings for None
            }
        }
        Ok(())
    }
    fn compile_value_match_statement(
        &mut self,
        expr: &Expr,
        arms: &[(Pattern, Option<Expr>, Vec<Statement>)],
        default: &Option<Vec<Statement>>,
        position: &FilePosition,
    ) -> NovaResult<()> {
        let end = self.gen.generate();

        // evaluate the match subject and store in temp
        self.compile_expr(expr)?;
        self.variables
            .insert(format!("___matchexpr___{}", self.gen.generate()).into());
        let temp = self.variables.len() - 1;
        self.asm.push(Asm::STORE(temp as u32));

        for arm in arms.iter() {
            let next = self.gen.generate();

            // emit pattern test
            self.compile_pattern_test(&arm.0, temp, position)?;
            self.asm.push(Asm::JUMPIFFALSE(next));

            // emit pattern bindings (needed before guard can reference bound variables)
            self.compile_pattern_bindings(&arm.0, temp, position)?;

            // emit optional if-guard
            if let Some(guard) = &arm.1 {
                self.compile_expr(guard)?;
                self.asm.push(Asm::JUMPIFFALSE(next));
            }

            // emit arm body
            let arm_ast = Ast {
                program: arm.2.clone(),
            };
            self.compile_program(arm_ast, self.filepath.clone(), false, false, false, false)?;
            self.asm.pop();

            self.asm.push(Asm::JMP(end));
            self.asm.push(Asm::LABEL(next));
        }

        if let Some(default) = default {
            let default_ast = Ast {
                program: default.clone(),
            };
            self.compile_program(
                default_ast,
                self.filepath.clone(),
                false,
                false,
                false,
                false,
            )?;
            self.asm.pop();
        }

        self.asm.push(Asm::LABEL(end));
        Ok(())
    }

    /// Compile a value-match expression (non-enum pattern matching, result on stack).
    fn compile_value_match_expr(
        &mut self,
        expr: &Expr,
        arms: &[(Pattern, Option<Expr>, Vec<Statement>)],
        default: &Option<Vec<Statement>>,
        position: &FilePosition,
    ) -> NovaResult<()> {
        let end = self.gen.generate();

        // evaluate the match subject and store in temp
        self.compile_expr(expr)?;
        self.variables
            .insert(format!("___matchexpr___{}", self.gen.generate()).into());
        let temp = self.variables.len() - 1;
        self.asm.push(Asm::STORE(temp as u32));

        for arm in arms.iter() {
            let next = self.gen.generate();

            // emit pattern test
            self.compile_pattern_test(&arm.0, temp, position)?;
            self.asm.push(Asm::JUMPIFFALSE(next));

            // emit pattern bindings (needed before guard can reference bound variables)
            self.compile_pattern_bindings(&arm.0, temp, position)?;

            // emit optional if-guard
            if let Some(guard) = &arm.1 {
                self.compile_expr(guard)?;
                self.asm.push(Asm::JUMPIFFALSE(next));
            }

            // emit arm body (keep=true so result stays on stack)
            let arm_ast = Ast {
                program: arm.2.clone(),
            };
            self.compile_program(arm_ast, self.filepath.clone(), false, false, false, true)?;
            self.asm.pop();

            self.asm.push(Asm::JMP(end));
            self.asm.push(Asm::LABEL(next));
        }

        if let Some(default) = default {
            let default_ast = Ast {
                program: default.clone(),
            };
            self.compile_program(
                default_ast,
                self.filepath.clone(),
                false,
                false,
                false,
                true,
            )?;
            self.asm.pop();
        }

        self.asm.push(Asm::LABEL(end));
        Ok(())
    }
}

