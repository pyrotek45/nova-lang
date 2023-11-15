use assembler::Assembler;
use common::{error::NovaError, tokens::TType};
use compiler::Compiler;
use lexer::Lexer;
use optimizer::Optimizer;
use parser::Parser;
use vm::Vm;

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

    pub fn run(mut self) -> Result<(), NovaError> {
        // adding native functions
        self.parser.environment.insert_symbol(
            "len",
            common::tokens::TType::Function(
                vec![TType::List(Box::new(TType::Generic("a".to_string())))],
                Box::new(TType::Int),
            ),
            None,
            common::nodes::SymbolKind::GenericFunction,
        );

        self.compiler.native_functions.insert("len".to_string());
        self.vm
            .native_functions
            .insert(self.vm.native_functions.len(), native::list::len);

        self.parser.environment.insert_symbol(
            "readline",
            common::tokens::TType::Function(vec![TType::None], Box::new(TType::String)),
            None,
            common::nodes::SymbolKind::GenericFunction,
        );
        self.compiler
            .native_functions
            .insert("readline".to_string());
        self.vm
            .native_functions
            .insert(self.vm.native_functions.len(), native::io::read_line);

        self.parser.environment.insert_symbol(
            "push",
            common::tokens::TType::Function(
                vec![
                    TType::List(Box::new(TType::Generic("a".to_string()))),
                    TType::Generic("a".to_string()),
                ],
                Box::new(TType::Void),
            ),
            None,
            common::nodes::SymbolKind::GenericFunction,
        );
        self.compiler.native_functions.insert("push".to_string());
        self.vm
            .native_functions
            .insert(self.vm.native_functions.len(), native::list::push);

        self.parser.environment.insert_symbol(
            "pop",
            common::tokens::TType::Function(
                vec![TType::List(Box::new(TType::Generic("a".to_string())))],
                Box::new(TType::Void),
            ),
            None,
            common::nodes::SymbolKind::GenericFunction,
        );
        self.compiler.native_functions.insert("pop".to_string());
        self.vm
            .native_functions
            .insert(self.vm.native_functions.len(), native::list::pop);

        self.parser.environment.insert_symbol(
            "randomInt",
            common::tokens::TType::Function(vec![TType::Int, TType::Int], Box::new(TType::Int)),
            None,
            common::nodes::SymbolKind::GenericFunction,
        );
        self.compiler
            .native_functions
            .insert("randomInt".to_string());
        self.vm
            .native_functions
            .insert(self.vm.native_functions.len(), native::rand::random_int);

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
        self.vm.run()?;
        Ok(())
    }
}
