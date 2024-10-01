use assembler::Assembler;
use common::{error::NovaError, nodes::SymbolKind, ttype::TType};
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
        self.parser
            .environment
            .insert_symbol(function_id, function_type, None, function_kind);

        self.compiler
            .native_functions
            .insert(function_id.to_string());
        self.vm
            .native_functions
            .insert(self.vm.native_functions.len(), function_pointer);
    }

    fn initnova(&mut self) {
        self.add_function(
            "toStr",
            TType::Function(
                vec![TType::Generic("a".to_string())],
                Box::new(TType::String),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::str::to_string,
        );
        self.add_function(
            "len",
            TType::Function(
                vec![TType::List(Box::new(TType::Generic("a".to_string())))],
                Box::new(TType::Int),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::list::len,
        );
        self.add_function(
            "sleep",
            TType::Function(vec![TType::Int], Box::new(TType::Void)),
            common::nodes::SymbolKind::GenericFunction,
            native::time::sleep,
        );
        self.add_function(
            "rawmode",
            TType::Function(vec![TType::Bool], Box::new(TType::Void)),
            common::nodes::SymbolKind::GenericFunction,
            native::terminal::rawmode,
        );
        self.add_function(
            "getch",
            TType::Function(
                vec![TType::None],
                Box::new(TType::Option(Box::new(TType::Char))),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::terminal::getch,
        );
        self.add_function(
            "rawread",
            TType::Function(
                vec![TType::Int],
                Box::new(TType::Option(Box::new(TType::Char))),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::terminal::rawread,
        );
        self.add_function(
            "readline",
            TType::Function(vec![TType::None], Box::new(TType::String)),
            common::nodes::SymbolKind::GenericFunction,
            native::io::read_line,
        );
        self.add_function(
            "clearscreen",
            TType::Function(vec![TType::None], Box::new(TType::Void)),
            common::nodes::SymbolKind::GenericFunction,
            native::terminal::clear_screen,
        );
        self.add_function(
            "push",
            TType::Function(
                vec![
                    TType::List(Box::new(TType::Generic("a".to_string()))),
                    TType::Generic("a".to_string()),
                ],
                Box::new(TType::Void),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::list::push,
        );
        self.add_function(
            "pop",
            TType::Function(
                vec![TType::List(Box::new(TType::Generic("a".to_string())))],
                Box::new(TType::Void),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::list::pop,
        );
        self.add_function(
            "randomInt",
            TType::Function(vec![TType::Int, TType::Int], Box::new(TType::Int)),
            common::nodes::SymbolKind::GenericFunction,
            native::rand::random_int,
        );
        self.add_function(
            "strlen",
            TType::Function(vec![TType::String], Box::new(TType::Int)),
            common::nodes::SymbolKind::GenericFunction,
            native::str::strlen,
        );
        self.add_function(
            "strToChars",
            TType::Function(
                vec![TType::String],
                Box::new(TType::List(Box::new(TType::Char))),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::str::str_to_chars,
        );
        self.add_function(
            "charsToStr",
            TType::Function(
                vec![TType::List(Box::new(TType::Char))],
                Box::new(TType::String),
            ),
            common::nodes::SymbolKind::GenericFunction,
            native::str::chars_to_str,
        );
        self.add_function(
            "chr",
            TType::Function(vec![TType::Int], Box::new(TType::Char)),
            common::nodes::SymbolKind::GenericFunction,
            native::char::chr,
        );
    }

    pub fn run(mut self) -> Result<(), NovaError> {
        self.initnova();
        let tokenlist = self.lexer.tokenize()?;
        self.parser.input = tokenlist;
        self.parser.parse()?;
        //dbg!(&self.parser.ast);
        let ast = self.parser.ast;
        let asm = self
            .compiler
            .compile_program(ast, self.filepath, true, true, false)?;
        self.assembler.input = asm;
        self.assembler.assemble();
        self.vm.runtime_errors_table = self.assembler.runtime_error_table.clone();
        self.vm.state.program = self.assembler.output;
        self.vm.run()?;
        Ok(())
    }

    pub fn run_time(mut self) -> Result<(), NovaError> {
        let start = std::time::Instant::now();
        self.initnova();
        println!("Initialize time: {}ms", start.elapsed().as_millis());

        let tokenlist = self.lexer.tokenize()?;
        println!("Lexing time: {}ms", start.elapsed().as_millis());

        self.parser.input = tokenlist;
        self.parser.parse()?;
        println!(
            "Parsing + Typechecking time: {}ms",
            start.elapsed().as_millis()
        );

        let ast = self.parser.ast;
        let asm = self
            .compiler
            .compile_program(ast, self.filepath, true, true, false)?;
        println!("Compile time: {}ms", start.elapsed().as_millis());

        self.assembler.input = asm;
        self.assembler.assemble();
        println!("Assembler time: {}ms", start.elapsed().as_millis());

        self.vm.runtime_errors_table = self.assembler.runtime_error_table.clone();
        self.vm.state.program = self.assembler.output;
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
        self.initnova();
        let tokenlist = self.lexer.tokenize()?;
        self.parser.input = tokenlist;
        self.parser.parse()?;
        let ast = self.parser.ast;
        let asm = self
            .compiler
            .compile_program(ast, self.filepath, true, true, false)?;
        self.assembler.input = asm;
        self.assembler.assemble();
        self.vm.state.program = self.assembler.output;
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

//                     "compile" => {
//                         if let Some(filepath) = std::env::args().nth(3) {
//                             let lexer = match Lexer::new(&filepath) {
//                                 Ok(lexer) => lexer,
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             };

//                             let tokenlist = match lexer.tokenize() {
//                                 Ok(tokenlist) => tokenlist,
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             };

//                             let mut parser = parser::new(&filepath);
//                             let mut compiler = compiler::new();
//                             let mut vm = vm::new();

//                             // adding native functions
//                             parser.environment.insert_symbol(
//                                 "len",
//                                 TType::Function(
//                                     vec![TType::List(Box::new(TType::Generic(
//                                         "a".to_string(),
//                                     )))],
//                                     Box::new(TType::Int),
//                                 ),
//                                 None,
//                                 common::nodes::SymbolKind::GenericFunction,
//                             );
//                             compiler.native_functions.insert("len".to_string());
//                             vm.native_functions.insert(0, native::list::len);

//                             parser.input = tokenlist;
//                             match parser.parse() {
//                                 Ok(()) => {
//                                     //dbg!(parser.ast.clone());
//                                 }
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             }

//                             let program = compiler
//                                 .compile_program(parser.ast, filepath, true, true, false);
//                             let asm = compiler.asm.clone();
//                             match program {
//                                 Ok(_) => {
//                                     let mut assembler = assembler::new(asm);
//                                     assembler.assemble();
//                                     let encoded: Vec<u8> =
//                                         bincode::serialize(&assembler.output.clone()).unwrap();
//                                     if let Some(outputname) = std::env::args().nth(4) {
//                                         std::fs::write(format!("{}.nvb", outputname), encoded)
//                                             .unwrap();
//                                     } else {
//                                         println!("Error: No output name specified");
//                                     }
//                                 }
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             }
//                         } else {
//                             println!("Error: No file path specified");
//                         }
//                     }
//                     _ => {
//                         println!("Error: Unrecognized option {}", option);
//                     }
//                 },
//                 None => todo!(),
//             }
//         }
//         "bin" => {
//             match std::env::args().nth(2) {
//                 Some(option) => match option.as_str() {
//                     "run" => {
//                         if let Some(filepath) = std::env::args().nth(3) {
//                             let encoded = std::fs::read(filepath).unwrap();
//                             let program = bincode::deserialize(&encoded).unwrap();

//                             let mut vm = vm::new();
//                             vm.native_functions.insert(0, native::list::len);
//                             vm.state.program(program);

//                             match vm.run() {
//                                 Ok(()) => {
//                                     //dbg!(vm.state.stack);
//                                 }
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             }
//                         } else {
//                             println!("Error: No file path specified");
//                         }
//                     }
//                     "dbg" => {
//                         if let Some(filepath) = std::env::args().nth(3) {
//                             let encoded = std::fs::read(filepath).unwrap();
//                             let program: Vec<u8> = bincode::deserialize(&encoded).unwrap();
//                             //println!("{}", rhexdump::hexdump(&program.clone()));
//                             let mut vm = vm::new();
//                             vm.native_functions.insert(0, native::list::len);
//                             vm.state.program(program);

//                             match vm.run_debug() {
//                                 Ok(()) => {
//                                     //dbg!(vm.state.stack);
//                                 }
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             }
//                         } else {
//                             println!("Error: No file path specified");
//                         }
//                     }
//                     _ => {}
//                 },
//                 None => todo!(),
//             }
//         }
//         "asm" => {
//             match std::env::args().nth(2) {
//                 Some(option) => match option.as_str() {
//                     "run" => {
//                         if let Some(filepath) = std::env::args().nth(3) {
//                             let lexer = match Lexer::new(&filepath) {
//                                 Ok(lexer) => lexer,
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             };

//                             let tokenlist = match lexer.tokenize() {
//                                 Ok(tokenlist) => tokenlist,
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             };

//                             let mut assembler = assembler::new_empty();
//                             assembler.assemble_from_nva(tokenlist);
//                             assembler.input = assembler.nva.clone();
//                             assembler.assemble();

//                             for o in assembler.nva {
//                                 println!("{:?}", o)
//                             }
//                             let mut vm = vm::new();
//                             vm.state.program(assembler.output);

//                             match vm.run() {
//                                 Ok(()) => {
//                                     //dbg!(vm.state.stack);
//                                 }
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             }
//                         } else {
//                             println!("Error: No file path specified");
//                         }
//                     }
//                     "compile" => {
//                         if let Some(filepath) = std::env::args().nth(3) {
//                             let lexer = match Lexer::new(&filepath) {
//                                 Ok(lexer) => lexer,
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             };

//                             let tokenlist = match lexer.tokenize() {
//                                 Ok(tokenlist) => tokenlist,
//                                 Err(error) => {
//                                     error.show();
//                                     exit(1)
//                                 }
//                             };

//                             let mut assembler = assembler::new_empty();
//                             assembler.assemble_from_nva(tokenlist);
//                             assembler.input = assembler.nva.clone();
//                             assembler.assemble();
//                             let encoded: Vec<u8> =
//                                 bincode::serialize(&assembler.output.clone()).unwrap();
//                             if let Some(outputname) = std::env::args().nth(4) {
//                                 std::fs::write(format!("{}.nvb", outputname), encoded).unwrap();
//                             } else {
//                                 println!("Error: No output name specified");
//                             }
//                         } else {
//                             println!("Error: No file path specified");
//                         }
//                     }
//                     _ => {}
//                 },
//                 None => todo!(),
//             }
//         }
//         _ => {}
//     },
//     None => todo!(),
// }
