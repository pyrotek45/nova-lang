use common::code::Asm;
use common::error::NovaError;
use common::gen::Gen;
use common::nodes::Statement::{Block, Expression, For, Function, If, Return, Struct, While};
use common::nodes::{Ast, Atom, Expr};
use common::ttype::TType;

#[derive(Debug, Clone)]
pub struct Compiler {
    pub bindings: common::table::Table<String>,
    pub global: common::table::Table<String>,
    pub variables: common::table::Table<String>,
    pub upvalues: common::table::Table<String>,
    pub native_functions: common::table::Table<String>,
    pub output: Vec<u8>,
    pub filepath: String,
    pub entry: usize,
    pub asm: Vec<Asm>,
    pub gen: Gen,
    pub breaks: Vec<u64>,
    pub continues: Vec<u64>,
}

pub fn new() -> Compiler {
    Compiler {
        native_functions: common::table::new(),
        variables: common::table::new(),
        output: Vec::new(),
        filepath: String::new(),
        upvalues: common::table::new(),
        global: common::table::new(),
        entry: 0,
        bindings: common::table::new(),
        asm: vec![],
        gen: common::gen::new(),
        breaks: vec![],
        continues: vec![],
    }
}

impl Compiler {
    pub fn clear(&mut self) {
        self.output.clear()
    }

    pub fn get_entry(&self) -> usize {
        self.entry
    }

    #[inline(always)]
    pub fn compile_program(
        &mut self,
        input: Ast,
        filepath: String,
        alloc: bool,
        global: bool,
        function: bool,
    ) -> Result<Vec<Asm>, NovaError> {
        self.filepath = filepath;

        for statements in input.program.iter() {
            match statements {
                common::nodes::Statement::Foreach {
                    identifier,
                    expr,
                    body,
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
                        .insert(format!("__tempcounter__{}", self.gen.generate()).to_string());
                    let tempcounter_index = self.variables.len() - 1;

                    self.variables
                        .insert(format!("__arrayexpr__{}", self.gen.generate()).to_string());
                    let array_index = self.variables.len() - 1;

                    let id_index = if let Some(index) = self.variables.get_index(identifier.clone())
                    {
                        index
                    } else {
                        self.variables.insert(identifier.to_string());
                        self.variables.len() - 1
                    };

                    self.compile_expr(expr.clone())?;
                    self.asm.push(Asm::STORE(array_index as u32));

                    // storing counter and expression array
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::STORE(tempcounter_index as u32));

                    // if array is empty jump to end
                    self.asm.push(Asm::LABEL(top));

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::GET(array_index as u32));
                    if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
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
                    if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
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
                    self.asm.push(Asm::LIN);

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
                    self.compile_expr(expr.clone())?;

                    if *global {
                        if let Some(index) = self.global.get_index(identifier.to_string()) {
                            self.asm.push(Asm::STOREGLOBAL(index as u32))
                        } else {
                            self.global.insert(identifier.to_string());
                            let index = self.global.len() - 1;
                            self.asm.push(Asm::STOREGLOBAL(index as u32))
                        }
                    } else {
                        if let Some(index) = self.variables.get_index(identifier.to_string()) {
                            self.asm.push(Asm::STORE(index as u32))
                        } else {
                            self.variables.insert(identifier.to_string());
                            let index = self.variables.len() - 1;
                            self.asm.push(Asm::STORE(index as u32))
                        }
                    }
                }
                Function {
                    identifier,
                    parameters,
                    body,
                    captures: captured,
                    ..
                } => {
                    self.global.insert(identifier.to_string());
                    // Clone the current state to prepare for function compilation
                    let mut function_compile = self.clone();
                    function_compile.variables.clear();
                    function_compile.asm.clear();

                    // Register parameter names in the function's local variable scope
                    for param in parameters.iter() {
                        function_compile
                            .variables
                            .insert(param.identifier.to_string());
                    }

                    // Register captured variables in the function's local variable scope
                    for capture in captured.iter() {
                        function_compile.variables.insert(capture.to_string());
                    }

                    // Compile captured variables for the closure
                    for captured_var in captured.iter().cloned() {
                        if let Some(index) = self.variables.get_index(captured_var.to_string()) {
                            // Get the local variable if it exists in the current scope
                            self.asm.push(Asm::GET(index as u32));
                        } else if let Some(index) = self.global.get_index(captured_var.to_string())
                        {
                            // Otherwise, get the global variable if it exists
                            self.asm.push(Asm::GETGLOBAL(index as u32));
                        } else {
                            // Debug output for missing variable
                            dbg!(&captured_var);
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
                    self.global.insert(identifier.to_string());
                    let structjump = self.gen.generate();
                    self.asm.push(Asm::FUNCTION(structjump));
                    self.asm
                        .push(Asm::OFFSET((fields.len() - 1) as u32, 0 as u32));
                    self.asm.push(Asm::STRING(identifier.clone()));
                    self.asm.push(Asm::LIST(fields.len() as u64));
                    self.asm.push(Asm::RET(true));
                    self.asm.push(Asm::LABEL(structjump));
                    let index = self.global.len() - 1;
                    self.asm.push(Asm::STOREGLOBAL(index as u32));
                }

                Return { ttype, expr } => {
                    self.compile_expr(expr.clone())?;
                    if ttype != &TType::Void {
                        self.asm.push(Asm::RET(true))
                    } else {
                        self.asm.push(Asm::RET(false))
                    }
                }
                Expression { ttype, expr } => {
                    self.compile_expr(expr.clone())?;
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
                    self.compile_expr(test.clone())?;
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
                    self.compile_expr(test.clone())?;
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
                    self.compile_expr(init.clone())?;
                    self.asm.push(Asm::LABEL(top));
                    self.compile_expr(test.clone())?;
                    self.asm.push(Asm::JUMPIFFALSE(end));
                    let whilebody = Ast {
                        program: body.clone(),
                    };
                    self.compile_program(whilebody, self.filepath.clone(), false, false, false)?;
                    self.asm.pop();
                    self.asm.push(Asm::LABEL(next));
                    self.compile_expr(inc.clone())?;
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
                    if let Some(index) = self.variables.get_index(identifier.to_string()) {
                        self.asm.push(Asm::GET(index as u32))
                    }
                    if let Some(index) = self
                        .native_functions
                        .get_index("Option::isSome".to_string())
                    {
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
                    self.compile_expr(expr.clone())?;
                    if let Some(index) = self
                        .native_functions
                        .get_index("Option::isSome".to_string())
                    {
                        self.asm.push(Asm::NATIVE(index as u64))
                    }
                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::ISSOME);
                    self.asm.push(Asm::JUMPIFFALSE(skip));
                    let id_index = if let Some(index) = self.variables.get_index(identifier.clone())
                    {
                        index
                    } else {
                        self.variables.insert(identifier.to_string());
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
                        if field.identifier == "type" {
                            continue;
                        }

                        self.global
                            .insert(format!("{}::{}", identifier, field.identifier).to_string());

                        //dbg!(format!("{}::{}", identifier, field.identifier));

                        let structjump = self.gen.generate();
                        self.asm.push(Asm::FUNCTION(structjump));
                        // offset is what it will accept
                        // enum is stored as a tuple [value,tag,type]
                        if field.ttype != TType::None {
                            self.asm.push(Asm::OFFSET((1) as u32, 0 as u32));
                        } else {
                            self.asm.push(Asm::OFFSET((0) as u32, 0 as u32));
                            self.asm.push(Asm::NONE);
                        }

                        self.asm.push(Asm::INTEGER(tag as i64));
                        self.asm.push(Asm::STRING(identifier.clone()));

                        self.asm.push(Asm::LIST(3 as u64));
                        self.asm.push(Asm::RET(true));

                        self.asm.push(Asm::LABEL(structjump));
                        let index = self.global.len() - 1;
                        self.asm.push(Asm::STOREGLOBAL(index as u32));
                    }
                }
                common::nodes::Statement::Match {
                    expr,
                    arms,
                    default,
                    ..
                } => {
                    // we will do it old school, each arm will make a new if branch
                    // and if it fails it will jump to the next arm
                    // if there is a default it will jump to the default
                    // if there is no default it will jump to the end
                    let end = self.gen.generate();
                    self.compile_expr(expr.clone())?;
                    // will test the expression
                    self.asm.push(Asm::INTEGER(1 as i64));
                    self.compile_expr(expr.clone())?;
                    self.asm.push(Asm::LIN);
                    // store in temp variable
                    self.variables
                        .insert(format!("__matchexpr__{}", self.gen.generate()).to_string());
                    let temp_matchexpr = self.variables.len() - 1;
                    self.asm.push(Asm::STORE(temp_matchexpr as u32));
                    for arm in arms.iter() {
                        let next = self.gen.generate();
                        self.asm.push(Asm::GET(temp_matchexpr as u32));
                        // self.asm.push(Asm::DUP);
                        // self.asm.push(Asm::PRINT);
                        //dbg!(arm.0);
                        self.asm.push(Asm::INTEGER(arm.0 as i64));
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::JUMPIFFALSE(next));
                        if let Some(vid) = &arm.1 {
                            self.asm.push(Asm::INTEGER(0 as i64));
                            self.compile_expr(expr.clone())?;
                            self.asm.push(Asm::LIN);
                            // store the vid in the variable
                            if let Some(index) = self.variables.get_index(vid.to_string()) {
                                self.asm.push(Asm::STORE(index as u32))
                            } else {
                                self.variables.insert(vid.to_string());
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

    pub fn getref_expr(&mut self, expr: Expr) -> Result<(), NovaError> {
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

                self.asm.push(Asm::INTEGER(index as i64));
                self.getref_expr(*expr)?;
                self.asm.push(Asm::PIN(position));
            }
            Expr::Indexed {
                container,
                index,
                position,
                ..
            } => {
                self.compile_expr(*index)?;
                let negitive_step = self.gen.generate();
                let negitive_step_end = self.gen.generate();

                self.compile_expr(*container.clone())?;
                self.variables
                    .insert(format!("__arrayexpr__{}", self.gen.generate()).to_string());
                let array_index = self.variables.len() - 1;
                self.asm.push(Asm::STORE(array_index as u32));

                self.asm.push(Asm::DUP);
                self.asm.push(Asm::INTEGER(0));
                self.asm.push(Asm::ILSS);
                self.asm.push(Asm::JUMPIFFALSE(negitive_step));
                self.asm.push(Asm::GET(array_index as u32));
                if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
                    self.asm.push(Asm::NATIVE(index as u64))
                } else {
                    todo!()
                }
                self.asm.push(Asm::IADD);
                self.asm.push(Asm::LABEL(negitive_step));

                self.getref_expr(*container)?;
                self.asm.push(Asm::PIN(position));
            }
            Expr::Call { .. } => todo!(),
            Expr::Unary { .. } => todo!(),
            Expr::Binop { .. } => todo!(),
            Expr::Literal { value, .. } => {
                self.getref_atom(value)?;
            }
            Expr::Closure { .. } => todo!(),
            Expr::ListCompConstructor { .. } => todo!(),
            Expr::Sliced {
                ttype,
                name,
                container,
                start: index,
                end,
                step,
                position,
            } => todo!(),
        }
        Ok(())
    }

    pub fn getref_atom(&mut self, atom: Atom) -> Result<(), NovaError> {
        match atom {
            Atom::Bool { value } => {
                self.asm.push(Asm::BOOL(value));
            }
            Atom::Id { name } => {
                if let Some(index) = self.variables.get_index(name.to_string()) {
                    self.asm.push(Asm::STACKREF(index as u32));
                } else {
                    self.variables.insert(name.to_string());
                    let index = self.variables.len() - 1;
                    self.asm.push(Asm::STACKREF(index as u32));
                }
            }
            Atom::Float { value: float } => {
                self.asm.push(Asm::FLOAT(float));
            }
            Atom::String { value: str } => {
                self.asm.push(Asm::STRING(str.clone()));
            }
            Atom::Integer { value: int } => {
                self.asm.push(Asm::INTEGER(int));
            }
            Atom::Call {
                name, arguments, ..
            } => {
                for expr in arguments.iter() {
                    self.compile_expr(expr.clone())?;
                }
                match name.as_str() {
                    "print" => self.asm.push(Asm::PRINT),
                    "free" => self.asm.push(Asm::FREE),
                    "clone" => self.asm.push(Asm::CLONE),
                    identifier => {
                        if let Some(index) = self.variables.get_index(identifier.to_string()) {
                            self.asm.push(Asm::GET(index as u32));
                            self.asm.push(Asm::CALL);
                        } else if let Some(index) = self.global.get_index(identifier.to_string()) {
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

    pub fn compile_expr(&mut self, expr: Expr) -> Result<(), NovaError> {
        match expr {
            Expr::None => {
                //    Ok(self.output.push(Code::NONE))
                Ok(())
            }
            Expr::ListConstructor { elements, .. } => {
                for x in elements.iter().cloned() {
                    self.compile_expr(x)?;
                }
                self.asm.push(Asm::LIST(elements.len() as u64));
                Ok(())
            }
            Expr::Field { index, expr, .. } => {
                self.asm.push(Asm::INTEGER(index as i64));
                self.compile_expr(*expr)?;
                self.asm.push(Asm::LIN);
                Ok(())
            }
            Expr::Indexed {
                container, index, ..
            } => {
                self.compile_expr(*index)?;
                let negitive_step = self.gen.generate();
                let negitive_step_end = self.gen.generate();

                self.compile_expr(*container)?;
                self.variables
                    .insert(format!("__arrayexpr__{}", self.gen.generate()).to_string());
                let array_index = self.variables.len() - 1;
                self.asm.push(Asm::STORE(array_index as u32));

                self.asm.push(Asm::DUP);
                self.asm.push(Asm::INTEGER(0));
                self.asm.push(Asm::ILSS);
                self.asm.push(Asm::JUMPIFFALSE(negitive_step));
                self.asm.push(Asm::GET(array_index as u32));
                if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
                    self.asm.push(Asm::NATIVE(index as u64))
                } else {
                    todo!()
                }
                self.asm.push(Asm::IADD);
                self.asm.push(Asm::JMP(negitive_step_end));
                self.asm.push(Asm::LABEL(negitive_step));
                self.asm.push(Asm::POP);
                self.asm.push(Asm::LABEL(negitive_step_end));

                self.asm.push(Asm::GET(array_index as u32));
                self.asm.push(Asm::LIN);
                Ok(())
            }
            Expr::Call { function, args, .. } => {
                for e in args.iter().cloned() {
                    self.compile_expr(e)?;
                }
                self.compile_expr(*function)?;
                self.asm.push(Asm::CALL);
                Ok(())
            }
            Expr::Unary { op, expr, .. } => match op {
                common::tokens::Unary::Positive => {
                    self.compile_expr(*expr)?;
                    Ok(())
                }
                common::tokens::Unary::Negitive => {
                    self.compile_expr(*expr)?;
                    self.asm.push(Asm::NEG);
                    Ok(())
                }
                common::tokens::Unary::Not => {
                    self.compile_expr(*expr)?;
                    Ok(self.asm.push(Asm::NOT))
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
                    common::tokens::Operator::GreaterThan => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IGTR);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FGTR);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::LessThan => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::ILSS);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FLSS);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Assignment => {
                        self.compile_expr(*rhs.clone())?;
                        self.getref_expr(*lhs.clone())?;

                        self.asm.push(Asm::ASSIGN)
                    }
                    common::tokens::Operator::Addition => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IADD);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FADD);
                        } else if lhs.get_type() == TType::String {
                            self.asm.push(Asm::CONCAT)
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Subtraction => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::ISUB);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FSUB);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Division => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IDIV);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FDIV);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Multiplication => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs)?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::IMUL);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FMUL);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::Equality => {
                        self.compile_expr(*lhs)?;
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::EQUALS);
                    }
                    common::tokens::Operator::Access => todo!(),
                    common::tokens::Operator::ListAccess => todo!(),
                    common::tokens::Operator::Call => todo!(),
                    common::tokens::Operator::Modulo => {
                        self.compile_expr(*lhs)?;
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::IMODULO);
                    }
                    common::tokens::Operator::NotEqual => {
                        self.compile_expr(*lhs)?;
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::NOT);
                    }
                    common::tokens::Operator::Not => {
                        self.compile_expr(*lhs)?;
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::NOT);
                    }
                    common::tokens::Operator::DoubleColon => todo!(),
                    common::tokens::Operator::Colon => todo!(),
                    common::tokens::Operator::GtrOrEqu => {
                        let sc = self.gen.generate();

                        // if lhs is true, return its value
                        // else return the other value
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs.clone())?;
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
                        self.compile_expr(*lhs)?;
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::LssOrEqu => {
                        let sc = self.gen.generate();

                        // if lhs is true, return its value
                        // else return the other value
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs.clone())?;
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
                        self.compile_expr(*lhs)?;
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::EQUALS);
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::And => {
                        let sc = self.gen.generate();

                        // if lhs is false, return its value
                        // else return other value
                        self.compile_expr(*lhs)?;
                        self.asm.push(Asm::DUP);
                        self.asm.push(Asm::JUMPIFFALSE(sc));
                        self.asm.push(Asm::POP);
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::Or => {
                        let sc = self.gen.generate();

                        // if lhs is true, return its value
                        // else return the other value
                        self.compile_expr(*lhs)?;
                        self.asm.push(Asm::DUP);
                        self.asm.push(Asm::NOT);
                        self.asm.push(Asm::JUMPIFFALSE(sc));
                        self.asm.push(Asm::POP);
                        self.compile_expr(*rhs)?;
                        self.asm.push(Asm::LABEL(sc))
                    }
                    common::tokens::Operator::AdditionAssignment => {
                        if lhs.get_type() == TType::Int {
                            self.compile_expr(*rhs.clone())?;
                            self.compile_expr(*lhs.clone())?;
                            self.asm.push(Asm::IADD);
                        } else if lhs.get_type() == TType::Float {
                            self.compile_expr(*rhs.clone())?;
                            self.compile_expr(*lhs.clone())?;
                            self.asm.push(Asm::FADD);
                        } else if lhs.get_type() == TType::String {
                            self.compile_expr(*lhs.clone())?;
                            self.compile_expr(*rhs.clone())?;
                            self.asm.push(Asm::CONCAT);
                        } else {
                            dbg!(&ttype);
                        }
                        self.getref_expr(*lhs.clone())?;
                        self.asm.push(Asm::ASSIGN)
                    }
                    common::tokens::Operator::SubtractionAssignment => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs.clone())?;
                        if lhs.get_type() == TType::Int {
                            self.asm.push(Asm::ISUB);
                        } else if lhs.get_type() == TType::Float {
                            self.asm.push(Asm::FSUB);
                        } else {
                            dbg!(&ttype);
                        }
                        self.getref_expr(*lhs.clone())?;

                        self.asm.push(Asm::ASSIGN)
                    }
                    common::tokens::Operator::Concat => {
                        self.compile_expr(*lhs.clone())?;
                        self.compile_expr(*rhs)?;
                        if lhs.get_type() == TType::String {
                            self.asm.push(Asm::CONCAT);
                        } else {
                            dbg!(&ttype);
                        }
                    }
                    common::tokens::Operator::LeftArrow => todo!(),
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
                    function_compile
                        .variables
                        .insert(param.identifier.to_string());
                }

                // Register captured variables in the function's local variable scope
                for capture in captured.iter() {
                    function_compile.variables.insert(capture.to_string());
                }

                // Compile captured variables for the closure
                for captured_var in captured.iter().cloned() {
                    //dbg!(&captured);
                    if let Some(index) = self.variables.get_index(captured_var.to_string()) {
                        // Get the local variable if it exists in the current scope
                        self.asm.push(Asm::GET(index as u32));
                    } else if let Some(index) = self.global.get_index(captured_var.to_string()) {
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
                list,
                expr,
                guards,
                identifier,
                ..
            } => {
                let top = self.gen.generate();
                let end = self.gen.generate();

                let mid = self.gen.generate();
                let step = self.gen.generate();

                let next = self.gen.generate();

                self.breaks.push(end);
                self.continues.push(next);
                // create temp list to hold new values

                self.variables
                    .insert(format!("__listexpr__{}", self.gen.generate()).to_string());
                let list_index = self.variables.len() - 1;
                self.asm.push(Asm::LIST(0));
                self.asm.push(Asm::STORE(list_index as u32));

                // insert temp counter
                self.variables
                    .insert(format!("__tempcounter__{}", self.gen.generate()).to_string());
                let tempcounter_index = self.variables.len() - 1;

                self.variables
                    .insert(format!("__arrayexpr__{}", self.gen.generate()).to_string());
                let array_index = self.variables.len() - 1;

                let id_index = if let Some(index) = self.variables.get_index(identifier.clone()) {
                    index
                } else {
                    self.variables.insert(identifier.to_string());
                    self.variables.len() - 1
                };
                // compile list expr
                self.compile_expr(*list.clone())?;
                self.asm.push(Asm::STORE(array_index as u32));

                // storing counter and expression array
                self.asm.push(Asm::INTEGER(0));
                self.asm.push(Asm::STORE(tempcounter_index as u32));

                // if array is empty jump to end
                self.asm.push(Asm::LABEL(top));

                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
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
                if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
                    self.asm.push(Asm::NATIVE(index as u64))
                }
                self.asm.push(Asm::EQUALS);

                self.asm.push(Asm::LABEL(step));
                self.asm.push(Asm::JUMPIFFALSE(mid));
                self.asm.push(Asm::JMP(end));
                self.asm.push(Asm::LABEL(mid));

                // bind value to identifier
                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                self.asm.push(Asm::LIN);

                self.asm.push(Asm::STORE(id_index as u32));

                // -- expr and then push to temp array
                self.compile_expr(*expr.clone())?;
                self.asm.push(Asm::STORE(id_index as u32));

                // store value in identifier

                // // compile guards
                for guard in guards.iter() {
                    self.compile_expr(guard.clone())?;
                    self.asm.push(Asm::JUMPIFFALSE(next));
                }

                self.asm.push(Asm::GET(list_index as u32));
                self.asm.push(Asm::GET(id_index as u32));

                if let Some(index) = self.native_functions.get_index("List::push".to_string()) {
                    self.asm.push(Asm::NATIVE(index as u64))
                } else {
                    todo!()
                }

                self.asm.push(Asm::LABEL(next));
                // increment counter
                self.asm.push(Asm::INTEGER(1));
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
            Expr::Sliced {
                ttype,
                name,
                container,
                start: startstep,
                end: endstep,
                step: stepstep,
                position,
            } => {
                let top = self.gen.generate();
                let end = self.gen.generate();

                let mid = self.gen.generate();
                let step = self.gen.generate();

                let next = self.gen.generate();

                let negitive_start = self.gen.generate();
                let negitive_start_end = self.gen.generate();

                let negitive_end = self.gen.generate();
                let negitive_end_end = self.gen.generate();

                let negitive_step = self.gen.generate();
                let negitive_step_end = self.gen.generate();

                self.breaks.push(end);
                self.continues.push(next);
                // create temp list to hold new values

                self.variables
                    .insert(format!("__listexpr__{}", self.gen.generate()).to_string());
                let list_index = self.variables.len() - 1;
                self.asm.push(Asm::LIST(0));
                self.asm.push(Asm::STORE(list_index as u32));

                // insert temp counter
                self.variables
                    .insert(format!("__tempcounter__{}", self.gen.generate()).to_string());
                let tempcounter_index = self.variables.len() - 1;

                self.variables
                    .insert(format!("__arrayexpr__{}", self.gen.generate()).to_string());
                let array_index = self.variables.len() - 1;

                self.variables
                    .insert(format!("__tempexpr__{}", self.gen.generate()).to_string());
                let id_index = self.variables.len() - 1;

                // compile list expr
                self.compile_expr(*container.clone())?;
                self.asm.push(Asm::STORE(array_index as u32));

                // compiling start as integer
                if let Some(startstep) = startstep.clone() {
                    self.compile_expr(*startstep.clone())?;
                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::ILSS);
                    self.asm.push(Asm::JUMPIFFALSE(negitive_start));
                    self.asm.push(Asm::GET(array_index as u32));
                    if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
                        self.asm.push(Asm::NATIVE(index as u64))
                    } else {
                        todo!()
                    }
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::JMP(negitive_start_end));
                    self.asm.push(Asm::LABEL(negitive_start));
                    self.asm.push(Asm::POP);
                    self.asm.push(Asm::LABEL(negitive_start_end));
                } else {
                    self.asm.push(Asm::INTEGER(0));
                }

                self.asm.push(Asm::STORE(tempcounter_index as u32));

                // if array is empty jump to end
                self.asm.push(Asm::LABEL(top));
                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
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
                if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
                    self.asm.push(Asm::NATIVE(index as u64))
                }
                self.asm.push(Asm::EQUALS);

                self.asm.push(Asm::LABEL(step));
                self.asm.push(Asm::JUMPIFFALSE(mid));
                self.asm.push(Asm::JMP(end));
                self.asm.push(Asm::LABEL(mid));

                // compile upper bound check
                if let Some(endstep) = endstep {
                    self.compile_expr(*endstep.clone())?;

                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::ILSS);
                    self.asm.push(Asm::JUMPIFFALSE(negitive_end));
                    self.asm.push(Asm::GET(array_index as u32));
                    if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
                        self.asm.push(Asm::NATIVE(index as u64))
                    } else {
                        todo!()
                    }
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::JMP(negitive_end_end));
                    self.asm.push(Asm::LABEL(negitive_end));
                    self.asm.push(Asm::POP);
                    self.asm.push(Asm::LABEL(negitive_end_end));

                    self.asm.push(Asm::GET(tempcounter_index as u32));
                    self.asm.push(Asm::IGTR);
                    self.asm.push(Asm::JUMPIFFALSE(end));
                }

                // bind value to identifier for (x in list) // x is identifier
                self.asm.push(Asm::GET(tempcounter_index as u32));
                self.asm.push(Asm::GET(array_index as u32));
                self.asm.push(Asm::LIN);
                self.asm.push(Asm::STORE(id_index as u32));

                // -- expr and then push to temp array
                self.asm.push(Asm::GET(list_index as u32));
                self.asm.push(Asm::GET(id_index as u32));

                if let Some(index) = self.native_functions.get_index("List::push".to_string()) {
                    self.asm.push(Asm::NATIVE(index as u64))
                } else {
                    todo!()
                }

                self.asm.push(Asm::LABEL(next));
                // increment counter
                if let Some(stepstep) = stepstep {
                    self.compile_expr(*stepstep.clone())?;

                    self.asm.push(Asm::DUP);
                    self.asm.push(Asm::INTEGER(0));
                    self.asm.push(Asm::ILSS);
                    self.asm.push(Asm::JUMPIFFALSE(negitive_step));
                    self.asm.push(Asm::GET(array_index as u32));
                    if let Some(index) = self.native_functions.get_index("List::len".to_string()) {
                        self.asm.push(Asm::NATIVE(index as u64))
                    } else {
                        todo!()
                    }
                    self.asm.push(Asm::IADD);
                    self.asm.push(Asm::JMP(negitive_step_end));
                    self.asm.push(Asm::LABEL(negitive_step));
                    self.asm.push(Asm::POP);
                    self.asm.push(Asm::LABEL(negitive_step_end));
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
        }
    }

    pub fn compile_atom(&mut self, atom: Atom) -> Result<(), NovaError> {
        match atom {
            Atom::Bool { value: bool } => {
                self.asm.push(Asm::BOOL(bool));
            }
            Atom::Id { name: identifier } => {
                if let Some(index) = self.variables.get_index(identifier.to_string()) {
                    self.asm.push(Asm::GET(index as u32));
                } else if let Some(index) = self.global.get_index(identifier.to_string()) {
                    self.asm.push(Asm::GETGLOBAL(index as u32));
                }
            }
            Atom::Float { value: float } => {
                self.asm.push(Asm::FLOAT(float));
            }
            Atom::String { value: str } => {
                self.asm.push(Asm::STRING(str.clone()));
            }
            Atom::Integer { value: int } => {
                self.asm.push(Asm::INTEGER(int));
            }
            Atom::Call {
                name: caller,
                arguments: list,
                position,
            } => {
                match caller.as_str() {
                    "typeof" => {
                        self.asm.push(Asm::STRING(list[0].get_type().to_string()));
                        return Ok(());
                    }
                    _ => {}
                }
                for expr in list.iter() {
                    self.compile_expr(expr.clone())?;
                }
                match caller.as_str() {
                    "println" => {
                        self.asm.push(Asm::PRINT);
                        self.asm.push(Asm::STRING("\n".to_string()));
                        self.asm.push(Asm::PRINT);
                    }
                    "print" => {
                        self.asm.push(Asm::PRINT);
                    }
                    "None" => self.asm.push(Asm::NONE),
                    "Option::unwrap" => self.asm.push(Asm::UNWRAP),
                    "Some" => {}
                    "Option::isSome" => self.asm.push(Asm::ISSOME),
                    "free" => self.asm.push(Asm::FREE),
                    "clone" => self.asm.push(Asm::CLONE),
                    "exit" => self.asm.push(Asm::EXIT),
                    "error" => self.asm.push(Asm::ERROR(position)),
                    identifier => {
                        if let Some(index) = self.native_functions.get_index(identifier.to_string())
                        {
                            self.asm.push(Asm::NATIVE(index as u64));
                        } else if let Some(index) = self.variables.get_index(identifier.to_string())
                        {
                            self.asm.push(Asm::GET(index as u32));
                            self.asm.push(Asm::CALL);
                        } else if let Some(index) = self.global.get_index(identifier.to_string()) {
                            self.asm.push(Asm::DCALL(index as u32));
                        } else {
                            dbg!(&self.variables);
                            dbg!(identifier);
                            todo!()
                        }
                    }
                }
            }
            Atom::Char { value: c } => self.asm.push(Asm::Char(c)),
            Atom::None => self.asm.push(Asm::NONE),
        }
        Ok(())
    }
}
