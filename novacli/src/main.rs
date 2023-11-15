use std::{process::exit, time::Instant, f32::consts::E};

use common::tokens::TType;
use lexer::Lexer;

fn main() {
    match std::env::args().nth(1) {
        Some(option) => match option.as_str() {
            "file" => {
                match std::env::args().nth(2) {
                    Some(option) => match option.as_str() {
                        "time" => {}
                        "run" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let _ = match novacore::NovaCore::new(&filepath) {
                                    Ok(novacore) => match novacore.run() {
                                        Ok(()) => {}
                                        Err(error) => {
                                            error.show();
                                            exit(1)
                                        }
                                    },
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };
                            }
                        }
                        "dbg" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let lexer = match Lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let tokenlist = match lexer.tokenize() {
                                    Ok(tokenlist) => tokenlist,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let mut parser = parser::new(&filepath);
                                let mut compiler = compiler::new();
                                let mut vm = vm::new();

                                // adding native functions
                                parser.environment.insert_symbol(
                                    "len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );

                                compiler.native_functions.insert("len".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::len);

                                parser.environment.insert_symbol(
                                    "readline",
                                    common::tokens::TType::Function(
                                        vec![TType::None],
                                        Box::new(TType::String),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("readline".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::io::read_line);

                                parser.environment.insert_symbol(
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
                                compiler.native_functions.insert("push".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::push);

                                parser.environment.insert_symbol(
                                    "pop",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Void),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("pop".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::pop);

                                parser.input = tokenlist;
                                match parser.parse() {
                                    Ok(()) => {
                                        //dbg!(parser.ast.clone());
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }

                                let program = compiler
                                    .compile_program(parser.ast, filepath, true, true, false);
                                let asm = compiler.asm.clone();
                                match program {
                                    Ok(_) => {
                                        // println!("Before optimization");
                                        // dis.dis_asm(asm.clone());
                                        // println!();
                                        // let mut optimizer = optimizer::new();
                                        // let optimized = optimizer.Optimize(asm.clone());
                                        //println!("After optimization: {}", optimizer.optimizations);
                                        // dis.dis_asm(asm.clone());
                                        //println!("{}", rhexdump::hexdump(&assembler.output));
                                        let mut assembler = assembler::new(asm);
                                        assembler.assemble();
                                        vm.state.program(assembler.output)
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                                match vm.run_debug() {
                                    Ok(()) => {
                                        //dbg!(vm.state.stack);
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                            } else {
                                println!("Error: No file path specified");
                            }
                        }
                        "dis" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let lexer = match Lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let tokenlist = match lexer.tokenize() {
                                    Ok(tokenlist) => tokenlist,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let mut parser = parser::new(&filepath);
                                let mut compiler = compiler::new();
                                let mut vm = vm::new();

                                // adding native functions
                                parser.environment.insert_symbol(
                                    "len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );

                                compiler.native_functions.insert("len".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::len);

                                parser.environment.insert_symbol(
                                    "readline",
                                    common::tokens::TType::Function(
                                        vec![TType::None],
                                        Box::new(TType::String),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("readline".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::io::read_line);

                                parser.environment.insert_symbol(
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
                                compiler.native_functions.insert("push".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::push);

                                parser.environment.insert_symbol(
                                    "pop",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Void),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("pop".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::pop);

                                parser.input = tokenlist;
                                match parser.parse() {
                                    Ok(()) => {
                                        //dbg!(parser.ast.clone());
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }

                                let program = compiler
                                    .compile_program(parser.ast, filepath, true, true, false);
                                let asm = compiler.asm.clone();
                                match program {
                                    Ok(_) => {
                                        // println!("Before optimization");
                                        let mut dis = disassembler::new();
                                        // dis.dis_asm(asm.clone());
                                        // println!();
                                        // let mut optimizer = optimizer::new();
                                        // let optimized = optimizer.Optimize(asm.clone());
                                        //println!("After optimization: {}", optimizer.optimizations);
                                        dis.dis_asm(asm.clone());
                                        //println!("{}", rhexdump::hexdump(&assembler.output));
                                        let mut assembler = assembler::new(asm);
                                        assembler.assemble();
                                        vm.state.program(assembler.output)
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                            } else {
                                println!("Error: No file path specified");
                            }
                        }

                        "compile" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let lexer = match Lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let tokenlist = match lexer.tokenize() {
                                    Ok(tokenlist) => tokenlist,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let mut parser = parser::new(&filepath);
                                let mut compiler = compiler::new();
                                let mut vm = vm::new();

                                // adding native functions
                                parser.environment.insert_symbol(
                                    "len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("len".to_string());
                                vm.native_functions.insert(0, native::list::len);

                                parser.input = tokenlist;
                                match parser.parse() {
                                    Ok(()) => {
                                        //dbg!(parser.ast.clone());
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }

                                let program = compiler
                                    .compile_program(parser.ast, filepath, true, true, false);
                                let asm = compiler.asm.clone();
                                match program {
                                    Ok(_) => {
                                        let mut assembler = assembler::new(asm);
                                        assembler.assemble();
                                        let encoded: Vec<u8> =
                                            bincode::serialize(&assembler.output.clone()).unwrap();
                                        if let Some(outputname) = std::env::args().nth(4) {
                                            std::fs::write(format!("{}.nvb", outputname), encoded)
                                                .unwrap();
                                        } else {
                                            println!("Error: No output name specified");
                                        }
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                            } else {
                                println!("Error: No file path specified");
                            }
                        }
                        _ => {
                            println!("Error: Unrecognized option {}", option);
                        }
                    },
                    None => todo!(),
                }
            }
            "bin" => {
                match std::env::args().nth(2) {
                    Some(option) => match option.as_str() {
                        "run" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let encoded = std::fs::read(filepath).unwrap();
                                let program = bincode::deserialize(&encoded).unwrap();

                                let mut vm = vm::new();
                                vm.native_functions.insert(0, native::list::len);
                                vm.state.program(program);

                                match vm.run() {
                                    Ok(()) => {
                                        //dbg!(vm.state.stack);
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                            } else {
                                println!("Error: No file path specified");
                            }
                        }
                        "dbg" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let encoded = std::fs::read(filepath).unwrap();
                                let program: Vec<u8> = bincode::deserialize(&encoded).unwrap();
                                //println!("{}", rhexdump::hexdump(&program.clone()));
                                let mut vm = vm::new();
                                vm.native_functions.insert(0, native::list::len);
                                vm.state.program(program);

                                match vm.run_debug() {
                                    Ok(()) => {
                                        //dbg!(vm.state.stack);
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                            } else {
                                println!("Error: No file path specified");
                            }
                        }
                        _ => {}
                    },
                    None => todo!(),
                }
            }
            "asm" => {
                match std::env::args().nth(2) {
                    Some(option) => match option.as_str() {
                        "run" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let lexer = match Lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let tokenlist = match lexer.tokenize() {
                                    Ok(tokenlist) => tokenlist,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let mut assembler = assembler::new_empty();
                                assembler.assemble_from_nva(tokenlist);
                                assembler.input = assembler.nva.clone();
                                assembler.assemble();

                                for o in assembler.nva {
                                    println!("{:?}", o)
                                }
                                let mut vm = vm::new();
                                vm.state.program(assembler.output);

                                match vm.run() {
                                    Ok(()) => {
                                        //dbg!(vm.state.stack);
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                            } else {
                                println!("Error: No file path specified");
                            }
                        }
                        "compile" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let lexer = match Lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let tokenlist = match lexer.tokenize() {
                                    Ok(tokenlist) => tokenlist,
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let mut assembler = assembler::new_empty();
                                assembler.assemble_from_nva(tokenlist);
                                assembler.input = assembler.nva.clone();
                                assembler.assemble();
                                let encoded: Vec<u8> =
                                    bincode::serialize(&assembler.output.clone()).unwrap();
                                if let Some(outputname) = std::env::args().nth(4) {
                                    std::fs::write(format!("{}.nvb", outputname), encoded).unwrap();
                                } else {
                                    println!("Error: No output name specified");
                                }
                            } else {
                                println!("Error: No file path specified");
                            }
                        }
                        _ => {}
                    },
                    None => todo!(),
                }
            }
            _ => {}
        },
        None => todo!(),
    }
}
