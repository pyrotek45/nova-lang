use std::{process::exit, time::Instant};

fn main() {
    match std::env::args().nth(1) {
        Some(option) => match option.as_str() {
            "time" => {
                let start = Instant::now();
                if let Some(filepath) = std::env::args().nth(2) {
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
                    let mut compiler = compiler::new();
                    let mut vm = vm::new();
                    let program = compiler.compile_program(parser.ast, filepath, true, true, false);
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
                if let Some(filepath) = std::env::args().nth(2) {
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
                    let mut compiler = compiler::new();
                    let mut vm = vm::new();
                    let program = compiler.compile_program(parser.ast, filepath, true, true, false);
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
            "dis" => {
                if let Some(filepath) = std::env::args().nth(2) {
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
                    let mut compiler = compiler::new();
                    let mut dis = disassembler::new();
                    let program = compiler.compile_program(parser.ast, filepath, true, true, false);
                    let asm = compiler.asm.clone();
                    match program {
                        Ok(_) => {
                            dis.dis_asm(asm.clone());
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
        None => {
            let mut input = String::new();
            loop {
                print!("Nova $ ");
                std::io::Write::flush(&mut std::io::stdout()).unwrap();
                input.clear();
                std::io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                match input.to_ascii_lowercase().as_str() {
                    "exit" => std::process::exit(0),
                    _ => {
                        if !input.is_empty() {
                            // match nova.eval(input, true) {
                            //     Ok(_) => {}
                            //     Err(error) => {error.show(); exit(1)},
                            // }
                        }
                    }
                }
            }
        }
    }
}
