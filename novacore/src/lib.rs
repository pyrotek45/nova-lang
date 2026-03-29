use std::{
    path::{Path, PathBuf},
    rc::Rc,
};

use assembler::Assembler;
use common::{
    error::NovaResult,
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

    pub fn new(path: &Path) -> NovaResult<NovaCore> {
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
        function_pointer: fn(&mut State) -> NovaResult<()>,
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

                self.parser.typechecker.environment.insert_symbol(
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
                self.parser.typechecker.environment.insert_symbol(
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
        self.parser.modules.insert("String".into());
        self.parser.modules.insert("Char".into());
        self.parser.modules.insert("Float".into());
        self.parser
            .typechecker
            .environment
            .custom_types
            .insert("Sprite".into(), vec![]);
        // assert functions
        self.add_function(
            "assert",
            TType::Function {
                parameters: vec![TType::Bool],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::assert::assert_true,
        );
        self.add_function(
            "assert",
            TType::Function {
                parameters: vec![TType::Bool, TType::String],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::assert::assert_msg,
        );
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
        // is key released
        self.add_function(
            "raylib::isKeyReleased",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_is_key_released,
        );
        // is mouse button pressed which returns bool
        self.add_function(
            "raylib::isMousePressed",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_is_mouse_button_down,
        );
        // is mouse button released which returns bool
        self.add_function(
            "raylib::isMouseReleased",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_mouse_button_released,
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
        // build sprite function which takes width and height, and list of tuples (int, int, int)
        self.add_function(
            "raylib::buildSprite",
            TType::Function {
                parameters: vec![
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::List {
                        inner: Box::new(TType::Tuple {
                            elements: vec![TType::Int, TType::Int, TType::Int],
                        }),
                    },
                ],
                return_type: Box::new(TType::Custom {
                    name: "Sprite".into(),
                    type_params: vec![],
                }),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::create_sprite_from_array,
        );
        // raylib draw rectangle outline
        self.add_function(
            "raylib::drawRectangleLines",
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
            native::raylib::raylib_draw_rectangle_lines,
        );
        // raylib draw circle outline
        self.add_function(
            "raylib::drawCircleLines",
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
            native::raylib::raylib_draw_circle_lines,
        );
        // raylib draw filled triangle
        self.add_function(
            "raylib::drawTriangle",
            TType::Function {
                parameters: vec![
                    TType::Int,
                    TType::Int,
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
            native::raylib::raylib_draw_triangle,
        );
        // raylib draw thick line
        self.add_function(
            "raylib::drawLineThick",
            TType::Function {
                parameters: vec![
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Float,
                    TType::Tuple {
                        elements: vec![TType::Int, TType::Int, TType::Int],
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_line_thick,
        );
        // raylib draw rounded rectangle
        self.add_function(
            "raylib::drawRoundedRectangle",
            TType::Function {
                parameters: vec![
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Int,
                    TType::Float,
                    TType::Tuple {
                        elements: vec![TType::Int, TType::Int, TType::Int],
                    },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_draw_rounded_rectangle,
        );
        // raylib get screen width
        self.add_function(
            "raylib::getScreenWidth",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_screen_width,
        );
        // raylib get screen height
        self.add_function(
            "raylib::getScreenHeight",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_screen_height,
        );
        // raylib set target FPS
        self.add_function(
            "raylib::setTargetFPS",
            TType::Function {
                parameters: vec![TType::Int],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_set_target_fps,
        );
        // raylib get current FPS
        self.add_function(
            "raylib::getFPS",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_fps,
        );
        // raylib measure text width
        self.add_function(
            "raylib::measureText",
            TType::Function {
                parameters: vec![TType::String, TType::Int],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_measure_text,
        );
        // raylib get mouse wheel move
        self.add_function(
            "raylib::getMouseWheel",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_get_mouse_wheel,
        );
        // raylib is key up (not pressed)
        self.add_function(
            "raylib::isKeyUp",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::raylib::raylib_is_key_up,
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
        // add printf function (overload with list)
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
        self.add_function(
            "toString",
            TType::Function {
                parameters: vec![TType::Generic { name: "a".into() }],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::str::to_string,
        );
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
            native::list::pop_item,
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
        // ---------------------------------------------------------------
        // New string functions
        // ---------------------------------------------------------------
        self.add_function(
            "String::contains",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_contains,
        );
        self.add_function(
            "String::startsWith",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_starts_with,
        );
        self.add_function(
            "String::endsWith",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_ends_with,
        );
        self.add_function(
            "String::trim",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_trim,
        );
        self.add_function(
            "String::trimStart",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_trim_start,
        );
        self.add_function(
            "String::trimEnd",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_trim_end,
        );
        self.add_function(
            "String::toUpper",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_to_upper,
        );
        self.add_function(
            "String::toLower",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_to_lower,
        );
        self.add_function(
            "String::replace",
            TType::Function {
                parameters: vec![TType::String, TType::String, TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_replace,
        );
        self.add_function(
            "String::substring",
            TType::Function {
                parameters: vec![TType::String, TType::Int, TType::Int],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_substring,
        );
        self.add_function(
            "String::indexOf",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_index_of,
        );
        self.add_function(
            "String::repeat",
            TType::Function {
                parameters: vec![TType::String, TType::Int],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_repeat,
        );
        self.add_function(
            "String::reverse",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_reverse,
        );
        self.add_function(
            "String::isEmpty",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_is_empty,
        );
        self.add_function(
            "String::charAt",
            TType::Function {
                parameters: vec![TType::String, TType::Int],
                return_type: Box::new(TType::Option {
                    inner: Box::new(TType::Char),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_char_at,
        );
        self.add_function(
            "String::split",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::List {
                    inner: Box::new(TType::String),
                }),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_split,
        );
        self.add_function(
            "join",
            TType::Function {
                parameters: vec![
                    TType::List {
                        inner: Box::new(TType::String),
                    },
                    TType::String,
                ],
                return_type: Box::new(TType::String),
            },
            common::nodes::SymbolKind::Function,
            native::str::str_join,
        );
        self.add_function(
            "Cast::charToInt",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::str::char_to_int,
        );
        // ---------------------------------------------------------------
        // Char functions
        // ---------------------------------------------------------------
        self.add_function(
            "Char::isAlpha",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_is_alpha,
        );
        self.add_function(
            "Char::isDigit",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_is_digit,
        );
        self.add_function(
            "Char::isWhitespace",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_is_whitespace,
        );
        self.add_function(
            "Char::isAlphanumeric",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_is_alphanumeric,
        );
        self.add_function(
            "Char::toUpper",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Char),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_to_upper,
        );
        self.add_function(
            "Char::toLower",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Char),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_to_lower,
        );
        self.add_function(
            "Char::isUpper",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_is_upper,
        );
        self.add_function(
            "Char::isLower",
            TType::Function {
                parameters: vec![TType::Char],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::char::char_is_lower,
        );
        // ---------------------------------------------------------------
        // Float math functions
        // ---------------------------------------------------------------
        self.add_function(
            "Float::floor",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_floor,
        );
        self.add_function(
            "Float::ceil",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_ceil,
        );
        self.add_function(
            "Float::round",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_round,
        );
        self.add_function(
            "Float::abs",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_abs,
        );
        self.add_function(
            "Float::sqrt",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_sqrt,
        );
        self.add_function(
            "Float::sin",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_sin,
        );
        self.add_function(
            "Float::cos",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_cos,
        );
        self.add_function(
            "Float::tan",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_tan,
        );
        self.add_function(
            "Float::atan2",
            TType::Function {
                parameters: vec![TType::Float, TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_atan2,
        );
        self.add_function(
            "Float::log",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_log,
        );
        self.add_function(
            "Float::log10",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_log10,
        );
        self.add_function(
            "Float::pow",
            TType::Function {
                parameters: vec![TType::Float, TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_pow,
        );
        self.add_function(
            "Float::exp",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_exp,
        );
        self.add_function(
            "Float::min",
            TType::Function {
                parameters: vec![TType::Float, TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_min,
        );
        self.add_function(
            "Float::max",
            TType::Function {
                parameters: vec![TType::Float, TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_max,
        );
        self.add_function(
            "Float::clamp",
            TType::Function {
                parameters: vec![TType::Float, TType::Float, TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_clamp,
        );
        self.add_function(
            "Float::isNan",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_is_nan,
        );
        self.add_function(
            "Float::isInfinite",
            TType::Function {
                parameters: vec![TType::Float],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_is_infinite,
        );
        self.add_function(
            "Float::pi",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_pi,
        );
        self.add_function(
            "Float::e",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::float::float_e,
        );
        // ---------------------------------------------------------------
        // List functions
        // ---------------------------------------------------------------
        self.add_function(
            "List::insert",
            TType::Function {
                parameters: vec![
                    TType::List {
                        inner: Box::new(TType::Generic { name: "a".into() }),
                    },
                    TType::Int,
                    TType::Generic { name: "a".into() },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::insert,
        );
        self.add_function(
            "List::swap",
            TType::Function {
                parameters: vec![
                    TType::List {
                        inner: Box::new(TType::Generic { name: "a".into() }),
                    },
                    TType::Int,
                    TType::Int,
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::swap,
        );
        self.add_function(
            "List::clear",
            TType::Function {
                parameters: vec![TType::List {
                    inner: Box::new(TType::Generic { name: "a".into() }),
                }],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::clear,
        );
        self.add_function(
            "List::set",
            TType::Function {
                parameters: vec![
                    TType::List {
                        inner: Box::new(TType::Generic { name: "a".into() }),
                    },
                    TType::Int,
                    TType::Generic { name: "a".into() },
                ],
                return_type: Box::new(TType::Void),
            },
            common::nodes::SymbolKind::GenericFunction,
            native::list::set,
        );
        // ---------------------------------------------------------------
        // Random functions
        // ---------------------------------------------------------------
        self.add_function(
            "randomFloat",
            TType::Function {
                parameters: vec![TType::Float, TType::Float],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::random::random_float,
        );
        self.add_function(
            "randomBool",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::random::random_bool,
        );
        // ---------------------------------------------------------------
        // Time functions
        // ---------------------------------------------------------------
        self.add_function(
            "now",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Int),
            },
            common::nodes::SymbolKind::Function,
            native::time::now_ms,
        );
        self.add_function(
            "nowSec",
            TType::Function {
                parameters: vec![TType::None],
                return_type: Box::new(TType::Float),
            },
            common::nodes::SymbolKind::Function,
            native::time::now_sec,
        );
        // ---------------------------------------------------------------
        // IO functions
        // ---------------------------------------------------------------
        self.add_function(
            "writeFile",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::io::write_file,
        );
        self.add_function(
            "fileExists",
            TType::Function {
                parameters: vec![TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::io::file_exists,
        );
        self.add_function(
            "appendFile",
            TType::Function {
                parameters: vec![TType::String, TType::String],
                return_type: Box::new(TType::Bool),
            },
            common::nodes::SymbolKind::Function,
            native::io::append_file,
        );
    }

    fn process(&mut self) -> NovaResult<()> {
        self.initnova();
        let tokenlist = self.lexer.tokenize()?;
        self.parser.input = tokenlist;
        self.parser.parse()?;
        let ast = self.parser.ast.clone();
        //dbg!(&ast);
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

    pub fn run_line(&mut self, line: &str, store: bool) -> NovaResult<()> {
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

    pub fn run(mut self) -> NovaResult<()> {
        self.process()?;
        self.vm.run()?;
        Ok(())
    }

    pub fn check(mut self) -> NovaResult<()> {
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

    pub fn run_debug(mut self) -> NovaResult<()> {
        self.process()?;
        self.vm.run_debug()?;
        Ok(())
    }

    pub fn dis_file(mut self) -> NovaResult<()> {
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
