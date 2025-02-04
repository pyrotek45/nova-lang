use std::{path::{Path, PathBuf}, rc::Rc};

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

#[derive(Debug, Clone)]
pub struct NovaCore {
    pub current_repl: String,
    filepath: Option<Rc<Path>>,
    lexer: Lexer,
    pub parser: Parser,
    compiler: Compiler,
    _optimizer: Optimizer,
    assembler: Assembler,
    vm: Vm,
}

impl NovaCore {
    pub fn repl() -> NovaCore {
        NovaCore {
            filepath: None,
            lexer: Lexer::default(),
            parser: parser::default(),
            compiler: compiler::new(),
            _optimizer: optimizer::new(),
            assembler: Assembler::empty(),
            vm: vm::new(),
            current_repl: "".to_string(),
        }
    }

    pub fn new(path: &Path) -> Result<NovaCore, NovaError> {
        Ok(NovaCore {
            filepath: Some(path.into()),
            lexer: Lexer::read_file(path)?,
            parser: parser::new(path),
            compiler: compiler::new(),
            _optimizer: optimizer::new(),
            assembler: Assembler::empty(),
            vm: vm::new(),
            current_repl: String::new(),
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
                    function_type.clone(),
                    None,
                    function_kind,
                );

                self.compiler.native_functions.insert(compiler_id.into());
                self.compiler
                    .native_functions_types
                    .insert(function_id.into(), function_type.clone());
                self.vm
                    .native_functions
                    .insert(self.vm.native_functions.len(), function_pointer);
            }
            _ => {
                self.parser.environment.insert_symbol(
                    function_id,
                    function_type.clone(),
                    None,
                    function_kind,
                );

                let function_id: Rc<str> = function_id.into();
                self.compiler.native_functions.insert(function_id.clone());
                self.compiler
                    .native_functions_types
                    .insert(function_id, function_type);
                self.vm
                    .native_functions
                    .insert(self.vm.native_functions.len(), function_pointer);
            }
        };
    }

    fn initnova(&mut self) {
        self.parser.modules.clear();
        self.parser.modules.insert("terminal".into());
        self.parser.modules.insert("Cast".into());
        self.parser.modules.insert("Regex".into());
        self.parser.modules.insert("raylib".into());
        self.parser
            .environment
            .custom_types
            .insert("Sprite".into(), vec![]);
        // add remove for list
        self.add_function(
            "List::remove",
            TType::Function {
                parameters: vec![
                    TType::List {
                        inner: Box::new(TType::Generic { name: "a".into() }),
                    },
                    TType::Int,
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::remove,
        );
        // add regex captures function, takes two strings and returns a list of strings
        // add printf function
        self.add_function(
            "printf",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::io::printf,
        );
        self.add_function(
            "Regex::captures",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::List {
                    inner: Box::new(TType::String),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::regex::regex_captures,
        );
        // raylib circle
        self.add_function(
            "raylib::drawCircle",
            TType::Function {
                parameters: vec![
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Tuple {
                        elements: vec![TType::Int, TType::Int, TType::Int],
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_circle,
        );
        // raylib line
        self.add_function(
            "raylib::drawLine",
            TType::Function {
                parameters: vec![
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Tuple {
                        elements: vec![TType::Int, TType::Int, TType::Int],
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_line,
        );
        // raylib load sprite function
        self.add_function(
            "raylib::loadSprite",
            TType::Function {
                parameters: vec![TType::String, TType::Int, TType::Int],
                return_type: Box::new(TType::Custom {
                    name: "Sprite".into(),
                    type_params: vec![],
                }),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_load_texture,
        );
        // raylib draw sprite function
        self.add_function(
            "raylib::drawSprite",
            TType::Function {
                parameters: vec![
                    TType::Custom {
                        name: "Sprite".into(),
                        type_params: vec![],
                    },
                    TType::Int,
                    TType::Int,
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_texture,
        );
        // raylib init
        self.add_function(
            "raylib::init",
            TType::Function {
                parameters: vec![TType::String, TType::Int, TType::Int, TType::Int],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_init,
        );
        // raylib get time
        self.add_function(
            "raylib::getTime",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_time,
        );
        // add raylib sleep function
        self.add_function(
            "raylib::sleep",
            TType::Function {
                parameters: vec![TType::Int],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_sleep,
        );
        // add raylib draw function
        self.add_function(
            "raylib::rendering",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_rendering,
        );
        // add raylib draw text function
        self.add_function(
            "raylib::drawText",
            TType::Function {
                parameters: vec![
                    TType::String,
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Tuple {
                        elements: vec![TType::Int, TType::Int, TType::Int],
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_text,
        );
        // raylib clear screen function
        self.add_function(
            "raylib::clear",
            TType::Function {
                parameters: vec![TType::Tuple {
                    elements: vec![TType::Int, TType::Int, TType::Int],
                }],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_clear,
        );
        // raylib draw fps
        self.add_function(
            "raylib::drawFPS",
            TType::Function {
                parameters: vec![TType::Int, TType::Int],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_fps,
        );
        // raylib mouse position
        self.add_function(
            "raylib::mousePosition",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Tuple {
                    elements: vec![TType::Int, TType::Int],
                }),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_mouse_position,
        );
        // raylib rectangle
        self.add_function(
            "raylib::drawRectangle",
            TType::Function {
                parameters: vec![
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Tuple {
                        elements: vec![TType::Int, TType::Int, TType::Int],
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_rectangle,
        );
        // return key pressed which returns Option<string>
        self.add_function(
            "raylib::getKey",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::String),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_key_as_string,
        );
        // is key pressed which returns bool
        self.add_function(
            "raylib::isKeyPressed",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_is_key_down,
        );
        // raylib gettimeframe
        self.add_function(
            "raylib::getFrameTime",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_frame_time,
        );
        // add regex match, takes two strings and returns a bool
        self.add_function(
            "Regex::matches",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::regex::regex_match,
        );
        self.add_function(
            "Regex::first",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Tuple {
                        elements: vec![TType::Int, TType::Int, TType::String],
                    }),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::regex::regex_first,
        );
        // add printf function
        self.add_function(
            "printf",
            TType::Function {
                parameters: vec![
                    TType::String,
                    TType::List {
                        inner: Box::new(TType::String),
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::io::printf,
        );
        // format function, same as printf but returns a string
        self.add_function(
            "format",
            TType::Function {
                parameters: vec![
                    TType::String,
                    TType::List {
                        inner: Box::new(TType::String),
                    },
                ],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::io::format,
        );
        self.add_function(
            "terminal::args",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::List {
                        inner: Box::new(TType::String),
                    }),
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
                parameters: vec![TType::Generic { name: "a".into() }],
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
                parameters: vec![TType::Generic { name: "a".into() }],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::str::to_string,
        );
        // alias for toString
        // self.add_function(
        //     "toString",
        //     TType::Function {
        //         parameters: vec![TType::Generic { name: "a".into() }],
        //         return_type: Box::new(TType::String),
        //     },
        //     common::nodes::SymbolKind::GenericFunction,
        //     native::str::to_string,
        // );
        self.add_function(
            "Cast::float",
            TType::Function {
                parameters: vec![TType::Any],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Float),
                }),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::float::int_to_float,
        );
        self.add_function(
            "List::len",
            TType::Function {
                parameters: vec![TType::List {
                    inner: Box::new(TType::Generic { name: "a".into() }),
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
                        inner: Box::new(TType::Generic { name: "a".into() }),
                    },
                    TType::Generic { name: "a".into() },
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
                    inner: Box::new(TType::Generic { name: "a".into() }),
                }],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Generic { name: "a".into() }),
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
            native::random::random_int,
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
            "chr",
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
        self.compiler.init();
        let asm = self
            .compiler
            .compile_program(ast, filepath, true, true, false, false)?;
        self.assembler.input = asm;
        self.assembler.assemble();
        self.vm.runtime_errors_table = self.assembler.runtime_error_table.clone();
        self.vm.state.program = self.assembler.output.clone();
        self.vm.state.current_dir = PathBuf::from(
            self.filepath
                .as_ref()
                .unwrap()
                .parent()
                .unwrap()
                .to_str()
                .unwrap(),
        );
        Ok(())
    }

    pub fn run_line(&mut self, line: &str, store: bool) -> Result<(), NovaError> {
        let oldrepl = self.current_repl.clone();

        self.current_repl.push_str(line);

        self.lexer = Lexer::new(self.current_repl.as_str(), None);
        self.initnova();

        self.parser = parser::default();
        self.parser.input = self.lexer.tokenize()?;
        self.initnova();

        self.parser.parse()?;
        let ast = self.parser.ast.clone();

        self.compiler = compiler::new();
        self.initnova();
        self.compiler.init();
        let asm = self
            .compiler
            .compile_program(ast, None, true, true, false, false)?;

        self.assembler = Assembler::empty();
        self.initnova();
        self.assembler.input = asm;
        self.assembler.assemble();

        self.vm = vm::new();
        self.initnova();

        self.vm.runtime_errors_table = self.assembler.runtime_error_table.clone();
        self.vm.state.program = self.assembler.output.clone();

        self.vm.run()?;
        if !store && (line.contains("println") || line.contains("print")) {
            self.current_repl = oldrepl;
        }

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
        println!("OK | Lexing time: {}ms", start.elapsed().as_millis());

        self.parser.input = tokenlist;
        self.parser.parse()?;
        println!(
            "OK | Parsing + Typechecking time: {}ms",
            start.elapsed().as_millis()
        );

        let ast = self.parser.ast;
        self.compiler.init();
        let asm = self
            .compiler
            .compile_program(ast, self.filepath, true, true, false, false)?;
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
        let ast = self.parser.ast.clone();
        let filepath = self.filepath.clone();
        self.compiler.init();
        let asm = self
            .compiler
            .compile_program(ast, filepath, true, true, false, false)?;
        let mut dis = disassembler::new();
        dis.dis_asm(asm);
        Ok(())
    }
}
