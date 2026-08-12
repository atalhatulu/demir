mod lexer;
mod token;
mod ast;
mod parser;
mod hir;
mod analyzer;
mod mir;
mod mir_builder;
mod borrowck;
mod ssa;
mod codegen;
mod aot;

use std::env;
use std::fs;

use lexer::Lexer;
use parser::Parser;
use analyzer::Analyzer;
use mir_builder::MirBuilder;
use borrowck::BorrowChecker;
use ssa::{SsaAnalyzer, place_phi_nodes};
use hir::HirStatement;
use codegen::JITCompiler;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let source_code = if args.len() > 1 {
        let filename = &args[1];
        fs::read_to_string(filename).expect(&format!("Could not read file: {}", filename))
    } else {
        println!("Usage: cargo run <file.dmr>");
        println!("No file provided. Running default example...");
        String::from(r#"
            fn main() {
                var counter: Int = 0;
                let limit: Int = 3;
                
                while counter < limit {
                    std.io.print(counter);
                    counter = counter + 1;
                }
            }
        "#)
    };

    println!("--- SOURCE CODE ---");
    println!("{}", source_code);
    println!("-------------------");

    let lexer = Lexer::new(&source_code);
    let mut parser = Parser::new(lexer);
    
    match parser.parse_program() {
        Ok(mut ast) => {
            let base_path = if args.len() > 1 {
                std::path::Path::new(&args[1]).parent().unwrap_or(std::path::Path::new(""))
            } else {
                std::path::Path::new(".")
            };
            
            fn resolve_imports(ast: &mut ast::Program, base_path: &std::path::Path, visited: &mut std::collections::HashSet<String>) {
                let mut new_statements = Vec::new();
                for stmt in &ast.statements {
                    if let ast::Statement::Import(path) = stmt {
                        if !visited.insert(path.clone()) { continue; } // Prevent cycles
                        let mut file_path = base_path.join(path.replace(".", "/") + ".dmr");
                        if !file_path.exists() {
                            file_path = base_path.join(path.clone() + ".dmr"); // support string literal paths
                        }
                        
                        if let Ok(source) = std::fs::read_to_string(&file_path) {
                            let lexer = lexer::Lexer::new(&source);
                            let mut parser = parser::Parser::new(lexer);
                            if let Ok(mut sub_ast) = parser.parse_program() {
                                resolve_imports(&mut sub_ast, base_path, visited);
                                new_statements.extend(sub_ast.statements);
                            } else {
                                println!("WARNING: Failed to parse imported file: {}", file_path.display());
                            }
                        } else {
                            println!("WARNING: Could not resolve import {}", path);
                        }
                    } else {
                        new_statements.push(stmt.clone());
                    }
                }
                ast.statements = new_statements;
            }
            
            let mut visited = std::collections::HashSet::new();
            resolve_imports(&mut ast, base_path, &mut visited);

            println!("--- AST (Resolved) ---");
            println!("{:#?}", ast);

            let mut analyzer = Analyzer::new();
            let hir = analyzer.analyze_program(ast);
            
            if analyzer.diagnostics.is_empty() {
                println!("--- HIR ---");
                println!("{:#?}", hir);
                
                let mut mir_funcs = Vec::new();
                for stmt in hir.statements {
                    if let HirStatement::FunctionDeclaration { ref name, ref params, .. } = stmt {
                        let mut mir_builder = MirBuilder::new(name.clone(), params.len(), &analyzer.structs);
                        match mir_builder.build(stmt.clone()) {
                            Ok(mut mir_func) => {
                                let mut borrow_checker = BorrowChecker::new();
                                if let Err(e) = borrow_checker.check(&mir_func) {
                                    println!("\nBORROW CHECK ERROR: {}", e);
                                    return;
                                }

                                println!("--- MIR (pre-SSA) ---");
                                println!("{}", mir_func);

                                let ssa_analyzer = SsaAnalyzer::new(&mir_func);
                                
                                println!("--- CFG ---");
                                for i in 0..ssa_analyzer.cfg.num_blocks {
                                    println!("bb{}: preds: {:?}, succs: {:?}", i, ssa_analyzer.cfg.preds[i], ssa_analyzer.cfg.succs[i]);
                                }
                                
                                ssa::build_ssa(&mut mir_func);

                                println!("--- SSA (post-renaming) ---");
                                println!("{}", mir_func);
                                
                                mir_funcs.push(mir_func);
                            }
                            Err(e) => {
                                println!("MIR Build Error: {}", e);
                                return;
                            }
                        }
                    }
                }
                println!("\n--- M13: JIT NATIVE EXECUTION (CRANELIFT) ---");
                // println!("Note: Cranelift backend does not yet support explicit SSA Phi nodes.");
                // println!("JIT execution is temporarily disabled until custom backend or Cranelift Phi support is implemented.");
                
                let mut jit = JITCompiler::new();
                match jit.compile_and_run(&mir_funcs) {
                    Ok(res) => println!("JIT Execution completed successfully. Exit code: {}", res),
                    Err(e) => println!("JIT Error: {}", e),
                }

                println!("\n--- M14: AOT COMPILATION ---");
                let mut aot = aot::AOTCompiler::new();
                let obj_path = "output.o";
                match aot.compile_and_write_object(&mir_funcs, obj_path) {
                    Ok(_) => {
                        println!("AOT Object file generated: {}", obj_path);
                        let status = std::process::Command::new("gcc")
                            .arg(obj_path)
                            .arg("runtime.c")
                            .arg("-o")
                            .arg("output")
                            .status();
                        match status {
                            Ok(s) if s.success() => println!("Successfully linked into standalone executable './output'"),
                            _ => println!("Failed to link using gcc"),
                        }
                    }
                    Err(e) => println!("AOT Error: {}", e),
                }
                
            } else {
                for diag in &analyzer.diagnostics {
                    println!("ERROR: {}", diag.message);
                }
            }
        }
        Err(e) => println!("Parse Error: {}", e),
    }
}
