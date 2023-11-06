use std::{process::exit, time::Instant};

use common::tokens::TType;

fn main() {
    match std::env::args().nth(1) {
        Some(option) => match option.as_str() {
            "file" => {
                match std::env::args().nth(2) {
                    Some(option) => match option.as_str() {
                        "time" => {
                            let start = Instant::now();
                            if let Some(filepath) = std::env::args().nth(3) {
                                let mut lexer = match lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        println!("{}", error);
                                        exit(1)
                                    }
                                };

                                let lexer_output = match lexer.tokenize() {
                                    Ok(output) => {
                                        // for t in output.clone().iter() {
                                        //     println!("{:?}", t)
                                        // }
                                        output
                                    }
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
                                    "super::len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("super::len".to_string());
                                vm.native_functions.insert(0, native::list::len);

                                parser.input = lexer_output.clone();
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
                                match vm.run() {
                                    Ok(()) => {
                                        //dbg!(vm.state.stack);
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                }
                                let duration = start.elapsed();
                                println!("Lexer & Parser Execution & Runtime >> {:?}", duration);
                            } else {
                                println!("Error: No file path specified");
                            }
                        }
                        "run" => {
                            if let Some(filepath) = std::env::args().nth(3) {
                                let mut lexer = match lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        println!("{}", error);
                                        exit(1)
                                    }
                                };
                                let lexer_output = match lexer.tokenize() {
                                    Ok(output) => {
                                        // for t in output.clone().iter() {
                                        //     println!("{:?}", t)
                                        // }
                                        output
                                    }
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
                                    "super::len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );

                                compiler.native_functions.insert("super::len".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::len);

                                parser.environment.insert_symbol(
                                    "super::readline",
                                    common::tokens::TType::Function(
                                        vec![TType::None],
                                        Box::new(TType::Str),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler
                                    .native_functions
                                    .insert("super::readline".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::io::read_line);

                                parser.environment.insert_symbol(
                                    "super::push",
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
                                compiler.native_functions.insert("super::push".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::push);

                                parser.environment.insert_symbol(
                                    "super::pop",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Void),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("super::pop".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::pop);

                                parser.input = lexer_output.clone();
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
                                let mut lexer = match lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        println!("{}", error);
                                        exit(1)
                                    }
                                };
                                let lexer_output = match lexer.tokenize() {
                                    Ok(output) => {
                                        // for t in output.clone().iter() {
                                        //     println!("{:?}", t)
                                        // }
                                        output
                                    }
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
                                    "super::len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );

                                compiler.native_functions.insert("super::len".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::len);

                                parser.environment.insert_symbol(
                                    "super::readline",
                                    common::tokens::TType::Function(
                                        vec![TType::None],
                                        Box::new(TType::Str),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler
                                    .native_functions
                                    .insert("super::readline".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::io::read_line);

                                parser.environment.insert_symbol(
                                    "super::push",
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
                                compiler.native_functions.insert("super::push".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::push);

                                parser.environment.insert_symbol(
                                    "super::pop",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Void),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("super::pop".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::pop);

                                parser.input = lexer_output.clone();
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
                                let mut lexer = match lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        println!("{}", error);
                                        exit(1)
                                    }
                                };
                                let lexer_output = match lexer.tokenize() {
                                    Ok(output) => {
                                        // for t in output.clone().iter() {
                                        //     println!("{:?}", t)
                                        // }
                                        output
                                    }
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
                                    "super::len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );

                                compiler.native_functions.insert("super::len".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::len);

                                parser.environment.insert_symbol(
                                    "super::readline",
                                    common::tokens::TType::Function(
                                        vec![TType::None],
                                        Box::new(TType::Str),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler
                                    .native_functions
                                    .insert("super::readline".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::io::read_line);

                                parser.environment.insert_symbol(
                                    "super::push",
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
                                compiler.native_functions.insert("super::push".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::push);

                                parser.environment.insert_symbol(
                                    "super::pop",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Void),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("super::pop".to_string());
                                vm.native_functions
                                    .insert(vm.native_functions.len(), native::list::pop);

                                parser.input = lexer_output.clone();
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
                                let mut lexer = match lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        println!("{}", error);
                                        exit(1)
                                    }
                                };
                                let lexer_output = match lexer.tokenize() {
                                    Ok(output) => {
                                        // for t in output.clone().iter() {
                                        //     println!("{:?}", t)
                                        // }
                                        output
                                    }
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
                                    "super::len",
                                    common::tokens::TType::Function(
                                        vec![TType::List(Box::new(TType::Generic(
                                            "a".to_string(),
                                        )))],
                                        Box::new(TType::Int),
                                    ),
                                    None,
                                    common::nodes::SymbolKind::GenericFunction,
                                );
                                compiler.native_functions.insert("super::len".to_string());
                                vm.native_functions.insert(0, native::list::len);

                                parser.input = lexer_output.clone();
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
                                println!("{}", rhexdump::hexdump(&program.clone()));
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
                                let mut lexer = match lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        println!("{}", error);
                                        exit(1)
                                    }
                                };
                                let lexer_output = match lexer.tokenize() {
                                    Ok(output) => {
                                        // for t in output.clone().iter() {
                                        //     println!("{:?}", t)
                                        // }

                                        output
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let mut assembler = assembler::new_empty();
                                assembler.assemble_from_nva(lexer_output.clone());
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
                                let mut lexer = match lexer::new(&filepath) {
                                    Ok(lexer) => lexer,
                                    Err(error) => {
                                        println!("{}", error);
                                        exit(1)
                                    }
                                };
                                let lexer_output = match lexer.tokenize() {
                                    Ok(output) => {
                                        // for t in output.clone().iter() {
                                        //     println!("{:?}", t)
                                        // }

                                        output
                                    }
                                    Err(error) => {
                                        error.show();
                                        exit(1)
                                    }
                                };

                                let mut assembler = assembler::new_empty();
                                assembler.assemble_from_nva(lexer_output.clone());
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
