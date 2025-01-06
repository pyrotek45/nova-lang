use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;
use std::rc::Rc;

use common::code::Asm;
use common::error::NovaError;
use common::fileposition::FilePosition;
use common::gen::Gen;
use common::nodes::Statement::{Block, Expression, For, Function, If, Return, Struct, While};
use common::nodes::{Ast, Atom, Expr};
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
    }
}

impl Compiler {
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
        self.asm.push(Asm::STRING("\n".into()));
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
                        self.asm.push(Asm::NATIVE(index as u64));
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
    ) -> Result<Vec<Asm>, NovaError> {
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
                    if let Some(index) = self.native_functions.get_index("List::len") {
                        self.asm.push(Asm::NATIVE(index as u64))
                    } else {
                        todo!()
                    }

                    self.asm.push(Asm::IGTR);
                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::NOT);

                    self.asm.push(Asm::JUMPIFFALSE(step));
                    self.asm.push(Asm::POP);

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    if let Some(index) = self.native_functions.get_index("List::len") {
                        self.asm.push(Asm::NATIVE(index as u64))
                    }
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
                    self.compile_program(foreach_body, self.filepath.clone(), false, false, false)?;
                    self.asm.pop();
                    // -- body

                    self.asm.push(Asm::LABEL(next));
                    // increment counter
                    self.asm.push(Asm::INTEGER(1));
                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::STACKREF(tempcounter_index as u32));
                    self.asm.push(Asm::ASSIGN);
                    self.asm.push(Asm::BJMP(top));
                    self.asm.push(Asm::LABEL(end));

                    self.breaks.pop();
                    self.continues.pop();
                }
                common::nodes::Statement::Pass => {}
                common::nodes::Statement::Let {
                    ttype: _,
                    identifier,
                    expr,
                    global,
                } => {
                    self.compile_expr(expr)?;

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
                }
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
                            // Debug output for missing variable
                            dbg!(captured_var);
                            panic!("Captured variable not found in local or global scope.");
                        }
                    }

                    // Generate a jump label for the closure function
                    let closure_jump_label = function_compile.gen.generate();

                    // Prepare the closure with a function label or a captured list if necessary
                    if captured.is_empty() {
                        self.asm.push(Asm::FUNCTION(closure_jump_label));
                    } else {
                        self.asm.push(Asm::LIST(captured.len() as u64));
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

                    // Append compiled function instructions to the current scope
                    self.gen = function_compile.gen;
                    function_compile.asm.pop(); // Remove the last instruction from function compilation
                    self.asm.extend_from_slice(&function_compile.asm);
                    self.asm.push(Asm::LABEL(closure_jump_label));

                    // storeing global function
                    let index = self.global.len() - 1;
                    self.asm.push(Asm::STOREGLOBAL(index as u32));
                }

                Struct {
                    ttype: _,
                    identifier,
                    fields,
                } => {
                    self.global.insert(identifier.clone());
                    let structjump = self.gen.generate();
                    self.asm.push(Asm::FUNCTION(structjump));
                    self.asm.push(Asm::OFFSET((fields.len() - 1) as u32, 0_u32));
                    self.asm.push(Asm::STRING(identifier.clone()));
                    self.asm.push(Asm::LIST(fields.len() as u64));
                    self.asm.push(Asm::RET(true));
                    self.asm.push(Asm::LABEL(structjump));
                    let index = self.global.len() - 1;
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
                    if ttype != &TType::Void {
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
                    self.compile_program(body_ast, self.filepath.clone(), false, false, false)?;
                    self.asm.pop();

                    if let Some(alternative) = alternative {
                        self.asm.push(Asm::JMP(alterjump));
                        self.asm.push(Asm::LABEL(bodyjump));
                        let alt = Ast {
                            program: alternative.clone(),
                        };
                        self.compile_program(alt, self.filepath.clone(), false, false, false)?;
                        self.asm.pop();
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
                    self.compile_program(whilebody, self.filepath.clone(), false, false, false)?;
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
                    self.compile_program(whilebody, self.filepath.clone(), false, false, false)?;
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
                    self.compile_program(body, filepath.clone(), false, false, false)?;
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
                        self.asm.push(Asm::NATIVE(index as u64))
                    }
                    self.asm.push(Asm::ISSOME);
                    self.asm.push(Asm::JUMPIFFALSE(skip));
                    let body = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(body, self.filepath.clone(), false, false, false)?;
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
                        self.asm.push(Asm::NATIVE(index as u64))
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
                    self.compile_program(body, self.filepath.clone(), false, false, false)?;
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
                        self.asm.push(Asm::STRING(identifier.clone()));

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
                        self.compile_program(arm, self.filepath.clone(), false, false, false)?;
                        self.asm.pop();

                        self.asm.push(Asm::JMP(end));
                        self.asm.push(Asm::LABEL(next));
                    }
                    if let Some(default) = default {
                        let default = Ast {
                            program: default.clone(),
                        };
                        self.compile_program(default, self.filepath.clone(), false, false, false)?;
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
                    self.compile_program(whilebody, self.filepath.clone(), false, false, false)?;
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

        match (function, alloc) {
            (true, _) => {}
            (false, true) => {
                self.asm
                    .insert(0, Asm::ALLOCLOCALS(self.variables.len() as u32));
            }
            _ => {}
        }

        if global {
            self.asm
                .insert(0, Asm::ALLOCGLOBBALS(self.global.len() as u32));
        }

        self.asm.push(Asm::RET(false));
        Ok(self.asm.to_owned())
    }

    pub fn getref_expr(&mut self, expr: &Expr) -> Result<(), NovaError> {
        match expr {
            Expr::None => {
                // self.output.push(Code::NONE)
            }
            Expr::ListConstructor { .. } => todo!(),
            Expr::Field {
                index,
                expr,
                position,
                ..
            } => {
                // dbg!(id, t);
                self.asm.push(Asm::INTEGER(*index as i64));
                self.getref_expr(expr)?;
                self.asm.push(Asm::PIN(position.clone()));
            }
            Expr::Indexed {
                container,
                index,
                position,
                ..
            } => {
                self.compile_expr(index)?;

                let negitive_step = self.gen.generate();
                self.compile_expr(container)?;
                self.variables
                    .insert(format!("__arrayexpr__{}", self.gen.generate()).into());
                let array_index = self.variables.len() - 1;
                self.asm.push(Asm::STORE(array_index as u32));

                self.asm.push(Asm::DUP);
                self.asm.push(Asm::INTEGER(0));
                self.asm.push(Asm::ILSS);
                self.asm.push(Asm::JUMPIFFALSE(negitive_step));
                self.asm.push(Asm::GET(array_index as u32));
                if let Some(index) = self.native_functions.get_index("List::len") {
                    self.asm.push(Asm::NATIVE(index as u64))
                } else {
                    todo!()
                }
                self.asm.push(Asm::IADD);
                self.asm.push(Asm::LABEL(negitive_step));

                self.getref_expr(container)?;
                self.asm.push(Asm::PIN(position.clone()));
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
        }
        Ok(())
    }

    pub fn getref_atom(&mut self, atom: &Atom) -> Result<(), NovaError> {
        match atom {
            Atom::Bool { value } => {
                self.asm.push(Asm::BOOL(*value));
            }
            Atom::Id { name } => {
                if let Some(index) = self.variables.get_index(name) {
                    self.asm.push(Asm::STACKREF(index as u32));
                } else {
                    self.variables.insert(name.clone());
                    let index = self.variables.len() - 1;
                    self.asm.push(Asm::STACKREF(index as u32));
                }
            }
            Atom::Float { value: float } => {
                self.asm.push(Asm::FLOAT(*float));
            }
            Atom::String { value: str } => {
                self.asm.push(Asm::STRING(str.clone()));
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

    pub fn compile_expr(&mut self, expr: &Expr) -> Result<(), NovaError> {
        match expr {
            Expr::None => {
                //    Ok(self.output.push(Code::NONE))
                Ok(())
            }
            Expr::ListConstructor { elements, .. } => {
                for x in elements {
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
            Expr::Indexed {
                container,
                index,
                position,
                ..
            } => {
                self.compile_expr(index)?;
                let negitive_step = self.gen.generate();

                self.compile_expr(container)?;
                self.variables
                    .insert(format!("__arrayexpr__{}", self.gen.generate()).into());
                let array_index = self.variables.len() - 1;
                self.asm.push(Asm::STORE(array_index as u32));

                self.asm.push(Asm::DUP);
                self.asm.push(Asm::INTEGER(0));
                self.asm.push(Asm::ILSS);
                self.asm.push(Asm::JUMPIFFALSE(negitive_step));
                self.asm.push(Asm::GET(array_index as u32));
                if let Some(index) = self.native_functions.get_index("List::len") {
                    self.asm.push(Asm::NATIVE(index as u64))
                } else {
                    todo!()
                }
                self.asm.push(Asm::IADD);
                self.asm.push(Asm::LABEL(negitive_step));

                self.asm.push(Asm::GET(array_index as u32));
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
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Assignment => {
                        self.compile_expr(rhs)?;
                        self.getref_expr(lhs)?;

                        self.asm.push(Asm::ASSIGN)
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
                        self.asm.push(Asm::ASSIGN)
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

                        self.asm.push(Asm::ASSIGN)
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
                    common::tokens::Operator::LeftArrow => todo!(),
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
                        // Debug output for missing variable
                        // dbg!(&captured_var);
                        // dbg!(&self.variables);
                        panic!("Captured variable not found in local or global scope.");
                    }
                }
                // Generate a jump label for the closure function
                let closure_jump_label = function_compile.gen.generate();

                // Prepare the closure with a function label or a captured list if necessary
                if captured.is_empty() {
                    self.asm.push(Asm::FUNCTION(closure_jump_label));
                } else {
                    self.asm.push(Asm::LIST(captured.len() as u64));
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
                )?;

                // Adjust the function's offset to account for parameters and captured variables
                let num_parameters = parameters.len() as u32;
                let num_captures = captured.len() as u32;
                let local_vars = function_compile.variables.len() as u32;
                self.asm.push(Asm::OFFSET(
                    num_parameters + num_captures,
                    local_vars - (num_parameters + num_captures),
                ));

                // Append compiled function instructions to the current scope
                self.gen = function_compile.gen;
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
                    if let Some(index) = self.native_functions.get_index("List::len") {
                        self.asm.push(Asm::NATIVE(index as u64))
                    } else {
                        todo!()
                    }
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
                if let Some(index) = self.native_functions.get_index("List::len") {
                    self.asm.push(Asm::NATIVE(index as u64))
                } else {
                    todo!()
                }

                self.asm.push(Asm::IGTR);
                self.asm.push(Asm::DUP);
                self.asm.push(Asm::NOT);

                self.asm.push(Asm::JUMPIFFALSE(step));
                self.asm.push(Asm::POP);

                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                if let Some(index) = self.native_functions.get_index("List::len") {
                    self.asm.push(Asm::NATIVE(index as u64))
                }
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
                    if let Some(index) = self.native_functions.get_index("List::len") {
                        self.asm.push(Asm::NATIVE(index as u64))
                    } else {
                        todo!()
                    }
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
                    self.asm.push(Asm::NATIVE(index as u64))
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
                self.asm.push(Asm::STACKREF(tempcounter_index as u32));
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
                self.compile_program(body, self.filepath.clone(), false, false, false)?;
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
                self.asm.pop();
                self.asm.push(Asm::JMP(end));
                self.asm.push(Asm::LABEL(next));
                self.compile_expr(alternative)?;
                self.asm.push(Asm::LABEL(end));
                Ok(())
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
    ) -> Result<(), NovaError> {
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
        if let Some(index) = self.native_functions.get_index("List::len") {
            self.asm.push(Asm::NATIVE(index as u64))
        } else {
            todo!()
        }
        self.asm.push(Asm::IGTR);
        self.asm.push(Asm::DUP);
        self.asm.push(Asm::NOT);
        self.asm.push(Asm::JUMPIFFALSE(step));
        self.asm.push(Asm::POP);
        self.asm.push(Asm::GET(tempcounter_index as u32));
        self.asm.push(Asm::GET(array_index as u32));
        if let Some(index) = self.native_functions.get_index("List::len") {
            self.asm.push(Asm::NATIVE(index as u64))
        }
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
                self.asm.push(Asm::NATIVE(index as u64))
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
        self.asm.push(Asm::STACKREF(tempcounter_index as u32));
        self.asm.push(Asm::ASSIGN);
        self.asm.push(Asm::BJMP(top));
        self.asm.push(Asm::LABEL(end));

        self.breaks.pop();
        self.continues.pop();
        Ok(())
    }

    pub fn compile_atom(&mut self, atom: &Atom) -> Result<(), NovaError> {
        match atom {
            Atom::Bool { value: bool } => {
                self.asm.push(Asm::BOOL(*bool));
            }
            Atom::Id { name: identifier } => {
                if let Some(index) = self.variables.get_index(identifier) {
                    self.asm.push(Asm::GET(index as u32));
                } else if let Some(index) = self.global.get_index(identifier) {
                    self.asm.push(Asm::GETGLOBAL(index as u32));
                } else {
                    return Err(NovaError::Compiler {
                        msg: format!("Identifier \"{}\" not found", identifier).into(),
                        note: "Identifier could not be loaded".into(),
                    });
                }
            }
            Atom::Float { value: float } => {
                self.asm.push(Asm::FLOAT(*float));
            }
            Atom::String { value: str } => {
                self.asm.push(Asm::STRING(str.clone()));
            }
            Atom::Integer { value: int } => {
                self.asm.push(Asm::INTEGER(*int));
            }
            Atom::Call {
                name: caller,
                arguments: list,
                position,
            } => {
                if caller.deref() == "typeof" {
                    self.asm
                        .push(Asm::STRING(list[0].get_type().to_string().into()));
                    return Ok(());
                }
                for expr in list {
                    self.compile_expr(expr)?;
                }
                match caller.deref() {
                    "unreachable" => self.asm.push(Asm::ERROR(position.clone())),
                    "todo" => {
                        // show a panic message before exiting
                        self.asm.push(Asm::STRING("Not yet implemented\n".into()));
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
                    identifier => {
                        //dbg!(identifier);
                        if let Some(index) = self.native_functions.get_index(identifier) {
                            self.asm.push(Asm::NATIVE(index as u64));
                        } else if let Some(index) = self.variables.get_index(identifier) {
                            self.asm.push(Asm::GET(index as u32));
                            self.asm.push(Asm::CALL);
                        } else if let Some(index) = self.global.get_index(identifier) {
                            self.asm.push(Asm::DCALL(index as u32));
                        } else {
                            return Err(NovaError::Compiler {
                                msg: format!("Function \"{}\" not found", identifier).into(),
                                note: "Function could not be loaded".into(),
                            });
                        }
                    }
                }
            }
            Atom::Char { value: c } => self.asm.push(Asm::Char(*c)),
            Atom::None => self.asm.push(Asm::NONE),
        }
        Ok(())
    }
}
