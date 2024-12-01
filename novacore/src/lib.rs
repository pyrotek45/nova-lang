use assembler::Assembler;
use common::{
    error::NovaError,
    nodes::SymbolKind,
    ttype::{generate_unique_string, TType},
};
use compiler::Compiler;
use lexer::Lexer;
use optimizer::Optimizer;
use parser::Parser;
use vm::{state::State, Vm};

#[derive(Debug)]
pub struct NovaCore {
    filepath: String,
    lexer: Lexer,
    parser: Parser,
    compiler: Compiler,
    _optimizer: Optimizer,
    assembler: Assembler,
    vm: Vm,
}

impl NovaCore {
    pub fn new(filepath: &str) -> Result<NovaCore, NovaError> {
        Ok(NovaCore {
            filepath: filepath.to_string(),
            lexer: Lexer::new(filepath)?,
            parser: parser::new(filepath),
            compiler: compiler::new(),
            _optimizer: optimizer::new(),
            assembler: assembler::new_empty(),
            vm: vm::new(),
        })
    }

    pub fn add_function(
        &mut self,
        function_id: &str,
        function_type: TType,
        function_kind: SymbolKind,
        function_pointer: fn(&mut State) -> Result<(), NovaError>,
    ) {
        match function_kind {
            SymbolKind::Function => {
                let compiler_id = {
                    let types = match function_type.clone() {
                        TType::Function { parameters, .. } => parameters,
                        _ => panic!("Expected function type"),
                    };
                    generate_unique_string(function_id, &types)
                };

                self.parser.environment.insert_symbol(
                    function_id,
                    function_type,
                    None,
                    function_kind,
                );

                self.compiler
                    .native_functions
                    .insert(compiler_id.to_string());
                self.vm
                    .native_functions
                    .insert(self.vm.native_functions.len(), function_pointer);
            }
            _ => {
                self.parser.environment.insert_symbol(
                    function_id,
                    function_type,
                    None,
                    function_kind,
                );

                self.compiler
                    .native_functions
                    .insert(function_id.to_string());
                self.vm
                    .native_functions
                    .insert(self.vm.native_functions.len(), function_pointer);
            }
        };

        self.parser.modules.insert("terminal".to_string());
        self.parser.modules.insert("Cast".to_string());
    }

    fn initnova(&mut self) {
        self.add_function(
            "terminal::args",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::List {
                    inner: Box::new(TType::String),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::terminal::retrieve_command_line_args,
        );
        self.add_function(
            "terminal::hideCursor",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::terminal::hide_cursor,
        );
        self.add_function(
            "terminal::showCursor",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::terminal::show_cursor,
        );
        self.add_function(
            "Cast::int",
            TType::Function {
                parameters: vec![TType::Generic {
                    name: "a".to_string(),
                }],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Int),
                }),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::str::to_int,
        );
        self.add_function(
            "Cast::string",
            TType::Function {
                parameters: vec![TType::Generic {
                    name: "a".to_string(),
                }],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::str::to_string,
        );
        self.add_function(
            "List::len",
            TType::Function {
                parameters: vec![TType::List {
                    inner: Box::new(TType::Generic {
                        name: "a".to_string(),
                    }),
                }],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::len,
        );
        self.add_function(
            "sleep",
            TType::Function {
                parameters: vec![TType::Int],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::time::sleep,
        );
        self.add_function(
            "terminal::rawmode",
            TType::Function {
                parameters: vec![TType::Bool],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::terminal::rawmode,
        );
        self.add_function(
            "terminal::getch",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Char),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::terminal::getch,
        );
        self.add_function(
            "terminal::rawread",
            TType::Function {
                parameters: vec![TType::Int],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Char),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::terminal::rawread,
        );
        self.add_function(
            "readln",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::io::read_line,
        );
        self.add_function(
            "terminal::clearScreen",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::terminal::clear_screen,
        );
        self.add_function(
            "List::push",
            TType::Function {
                parameters: vec![
                    TType::List {
                        inner: Box::new(TType::Generic {
                            name: "a".to_string(),
                        }),
                    },
                    TType::Generic {
                        name: "a".to_string(),
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::push,
        );
        self.add_function(
            "List::pop",
            TType::Function {
                parameters: vec![TType::List {
                    inner: Box::new(TType::Generic {
                        name: "a".to_string(),
                    }),
                }],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Generic {
                        name: "a".to_string(),
                    }),
                }),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::pop,
        );
        self.add_function(
            "random",
            TType::Function {
                parameters: vec![TType::Int, TType::Int],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::rand::random_int,
        );
        self.add_function(
            "String::len",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::str::strlen,
        );
        self.add_function(
            "String::chars",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::List {
                    inner: Box::new(TType::Char),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_to_chars,
        );
        self.add_function(
            "List::string",
            TType::Function {
                parameters: vec![TType::List {
                    inner: Box::new(TType::Char),
                }],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::chars_to_str,
        );
        self.add_function(
            "Int::toChar",
            TType::Function {
                parameters: vec![TType::Int],
                return_type: Box::new(TType::Char),
            },
            common::nodes::SymbolKind::Function,
            native::char::int_to_char,
        );
        self.add_function(
            "readFile",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::io::read_file,
        );
    }

    fn process(&mut self) -> Result<(), NovaError> {
        self.initnova();
        let tokenlist = self.lexer.tokenize()?;
        self.parser.input = tokenlist;
        self.parser.parse()?;
        let ast = self.parser.ast.clone();
        let filepath = self.filepath.clone();
        let asm = self
            .compiler
            .compile_program(ast.clone(), filepath, true, true, false)?;
        self.assembler.input = asm;
        self.assembler.assemble();
        self.vm.runtime_errors_table = self.assembler.runtime_error_table.clone();
        self.vm.state.program = self.assembler.output.clone();
        Ok(())
    }

    pub fn run(mut self) -> Result<(), NovaError> {
        self.process()?;
        self.vm.run()?;
        Ok(())
    }

    pub fn check(mut self) -> Result<(), NovaError> {
        let start = std::time::Instant::now();
        self.initnova();
        println!("OK | Initialize time: {}ms", start.elapsed().as_millis());

        let tokenlist = self.lexer.tokenize()?;
        self.lexer.check();
        println!("OK | Lexing time: {}ms", start.elapsed().as_millis());

        self.parser.input = tokenlist;
        self.parser.parse()?;
        println!(
            "OK | Parsing + Typechecking time: {}ms",
            start.elapsed().as_millis()
        );

        let ast = self.parser.ast;
        let asm = self
            .compiler
            .compile_program(ast, self.filepath, true, true, false)?;
        println!("OK | Compile time: {}ms", start.elapsed().as_millis());

        self.assembler.input = asm;
        self.assembler.assemble();
        println!("OK | Assembler time: {}ms", start.elapsed().as_millis());

        self.vm.runtime_errors_table = self.assembler.runtime_error_table.clone();
        self.vm.state.program = self.assembler.output;
        Ok(())
    }

    pub fn run_debug(mut self) -> Result<(), NovaError> {
        self.process()?;
        self.vm.run_debug()?;
        Ok(())
    }

    pub fn dis_file(mut self) -> Result<(), NovaError> {
        self.initnova();
        let tokenlist = self.lexer.tokenize()?;
        self.parser.input = tokenlist;
        self.parser.parse()?;
        let ast = self.parser.ast;
        let asm = self
            .compiler
            .compile_program(ast, self.filepath, true, true, false)?;
        let mut dis = disassembler::new();
        dis.dis_asm(asm);
        Ok(())
    }
}
