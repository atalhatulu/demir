use cranelift::prelude::*;
use cranelift_module::{Module, Linkage};
use cranelift_jit::{JITBuilder, JITModule};
use std::collections::HashMap;
use crate::mir::*;
use crate::hir::Type;

// --- M11 Runtime / C ABI ---
use std::io::Write;
// Bu gerçek C ABI üzerinden derlenen kodun çağıracağı host fonksiyonumuz.
unsafe extern "C" fn std_io_print(val: i64) {
    println!(">>> JIT OUTPUT: {}", val);
    std::io::stdout().flush().unwrap();
}

unsafe extern "C" fn std_io_print_str(ptr: i64) {
    let c_str = std::ffi::CStr::from_ptr(ptr as *const std::ffi::c_char);
    println!(">>> JIT OUTPUT: {}", c_str.to_str().unwrap());
    std::io::stdout().flush().unwrap();
}

// Contract assert'i ihlal edildiğinde anlamlı mesaj basar ve temiz çıkış yapar.
// Eski davranış (`trapz`) Cranelift'ta SIGILL (Illegal instruction) üretiyordu.
unsafe extern "C" fn __demir_assert_fail(ptr: i64) -> ! {
    let c_str = std::ffi::CStr::from_ptr(ptr as *const std::ffi::c_char);
    eprintln!("ASSERT FAILED: {}", c_str.to_str().unwrap_or("<invalid>"));
    std::process::exit(1);
}

pub struct JITCompiler {
    builder_context: FunctionBuilderContext,
    ctx: codegen::Context,
    module: JITModule,
}

impl JITCompiler {
    pub fn new() -> Self {
        let mut builder = JITBuilder::new(cranelift_module::default_libcall_names()).unwrap();
        
        // Host fonksiyonumuzu JIT Engine'e sembol olarak kaydediyoruz
        builder.symbol("std_io_print", std_io_print as *const u8);
        builder.symbol("std_io_print_str", std_io_print_str as *const u8);
        builder.symbol("__demir_assert_fail", __demir_assert_fail as *const u8);

        let module = JITModule::new(builder);
        
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            module,
        }
    }

    pub fn compile_and_run(&mut self, mir_funcs: &[MirFunction]) -> Result<i64, String> {
        let mut func_ids = HashMap::new();

        // 1. Declare all functions first (First Pass)
        for mir_func in mir_funcs {
            let mut sig = self.module.make_signature();
            for _ in 0..mir_func.param_count {
                sig.params.push(AbiParam::new(types::I64));
            }
            sig.returns.push(AbiParam::new(types::I64));
            
            let id = self.module.declare_function(&mir_func.name, Linkage::Export, &sig).unwrap();
            func_ids.insert(mir_func.name.clone(), id);
        }

        // 2. Define all function bodies (Second Pass)
        for mir_func in mir_funcs {
            self.ctx.func.signature = self.module.make_signature();
            for _ in 0..mir_func.param_count {
                self.ctx.func.signature.params.push(AbiParam::new(types::I64));
            }
            self.ctx.func.signature.returns.push(AbiParam::new(types::I64));

            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);

            // Borrow/pointer (stack-slot promotion): address-taken locals live in stack
            // slots so that &x / &mut x can yield a stable address.
            let addr_taken = collect_addr_taken(mir_func);
            let mut slot_map: HashMap<LocalId, cranelift::codegen::ir::StackSlot> = HashMap::new();
            for l in &addr_taken {
                let slot = builder.create_sized_stack_slot(
                    cranelift::prelude::StackSlotData::new(
                        cranelift::prelude::StackSlotKind::ExplicitSlot,
                        8,
                    ),
                );
                slot_map.insert(*l, slot);
            }

            let mut block_map = HashMap::new();
            for i in 0..mir_func.blocks.len() {
                block_map.insert(BasicBlockId(i), builder.create_block());
            }

            for local in &mir_func.locals {
                let cl_type = match local.ty {
                    Type::Int => types::I64,
                    Type::Float => types::F64,
                    Type::Bool => types::I64,
                    Type::String => types::I64,
                    _ => types::I64,
                };
                let var = Variable::new(local.id.0);
                builder.declare_var(var, cl_type);
            }

            let entry_block = block_map[&mir_func.start_block()];
            builder.append_block_params_for_function_params(entry_block);
            builder.switch_to_block(entry_block);
            
            let block_params = builder.block_params(entry_block).to_vec();
            for (i, &param_val) in block_params.iter().enumerate() {
                let local_id = mir_func.locals[i].id.0;
                let var = Variable::new(local_id);
                builder.def_var(var, param_val);
                // If this parameter is address-taken, spill its initial value into
                // its stack slot so borrows observe it immediately.
                let lid = mir_func.locals[i].id;
                if addr_taken.contains(&lid) {
                    if let Some(slot) = slot_map.get(&lid) {
                        let ptr = builder.ins().stack_addr(types::I64, *slot, 0);
                        builder.ins().store(cranelift::prelude::MemFlags::new(), param_val, ptr, 0);
                    }
                }
            }

            for (i, mir_block) in mir_func.blocks.iter().enumerate() {
                let cl_block = block_map[&BasicBlockId(i)];
                
                if i > 0 {
                    builder.switch_to_block(cl_block);
                }

                for stmt in &mir_block.statements {
                    match stmt {
                        Statement::Assign(dest, rval) => {
                            let dest_var = Variable::new(dest.0);
                            let value = match rval {
                                Rvalue::Use(op) => {
                                    compile_operand(&mut builder, op)
                                }
                                Rvalue::BinaryOp(op, left, right) => {
                                    let l = compile_operand(&mut builder, left);
                                    let r = compile_operand(&mut builder, right);
                                    match op.as_str() {
                                        "+" => builder.ins().iadd(l, r),
                                        "-" => builder.ins().isub(l, r),
                                        "*" => builder.ins().imul(l, r),
                                        "/" => builder.ins().sdiv(l, r),
                                        "<" => {
                                            let cmp = builder.ins().icmp(IntCC::SignedLessThan, l, r);
                                            builder.ins().uextend(types::I64, cmp)
                                        },
                                        ">" => {
                                            let cmp = builder.ins().icmp(IntCC::SignedGreaterThan, l, r);
                                            builder.ins().uextend(types::I64, cmp)
                                        },
                                        "==" => {
                                            let cmp = builder.ins().icmp(IntCC::Equal, l, r);
                                            builder.ins().uextend(types::I64, cmp)
                                        },
                                        "!=" => {
                                            let cmp = builder.ins().icmp(IntCC::NotEqual, l, r);
                                            builder.ins().uextend(types::I64, cmp)
                                        },
                                        _ => builder.ins().iconst(types::I64, 0),
                                    }
                                }
                                Rvalue::StructAlloc(size) => {
                                    let slot = builder.create_sized_stack_slot(cranelift::prelude::StackSlotData::new(cranelift::prelude::StackSlotKind::ExplicitSlot, *size as u32));
                                    builder.ins().stack_addr(types::I64, slot, 0)
                                }
                                Rvalue::FieldLoad(ptr_local, offset) => {
                                    let ptr_var = Variable::new(ptr_local.0);
                                    let ptr_val = builder.use_var(ptr_var);
                                    builder.ins().load(types::I64, cranelift::prelude::MemFlags::new(), ptr_val, *offset as i32)
                                }
                                Rvalue::AddressOf(l) | Rvalue::MutAddressOf(l) => {
                                    // Stack-slot promotion: &x / &mut x -> address of x's stack slot
                                    if let Some(slot) = slot_map.get(l) {
                                        builder.ins().stack_addr(types::I64, *slot, 0)
                                    } else {
                                        return Err(format!(
                                            "Compile error: borrow target local {} is not addressable (no stack slot)",
                                            l.0
                                        ));
                                    }
                                }
                                Rvalue::Dereference(local) => {
                                    // *ptr -> load I64 from address held by ptr
                                    let ptr_var = Variable::new(local.0);
                                    let ptr_val = builder.use_var(ptr_var);
                                    builder.ins().load(types::I64, cranelift::prelude::MemFlags::new(), ptr_val, 0)
                                }
                            };
                            builder.def_var(dest_var, value);
                            // Addr-taken local: value must also be spilled to its stack slot
                            // so that borrows of this local observe the written value.
                            if addr_taken.contains(dest) {
                                if let Some(slot) = slot_map.get(dest) {
                                    let ptr = builder.ins().stack_addr(types::I64, *slot, 0);
                                    builder.ins().store(cranelift::prelude::MemFlags::new(), value, ptr, 0);
                                }
                            }
                        }
                        Statement::Store(ptr_local, offset, op) => {
                            let ptr_var = Variable::new(ptr_local.0);
                            let ptr_val = builder.use_var(ptr_var);
                            let val = compile_operand(&mut builder, op);
                            builder.ins().store(cranelift::prelude::MemFlags::new(), val, ptr_val, *offset as i32);
                        }
                        Statement::Assert(cond, msg) => {
                            // cond == 0 ise fail runtime fonksiyonunu çağır (mesaj basar + exit).
                            // Eski `trapz` SIGILL üretiyordu ve mesaj basmıyordu.
                            let cond_val = compile_operand(&mut builder, cond);

                            // Mesajı C-string olarak heap'e kaçır (AOT kaynağında da aynı desen)
                            let mut c_msg = msg.clone();
                            c_msg.push('\0');
                            let leaked: &'static str = Box::leak(c_msg.into_boxed_str());
                            let msg_ptr = leaked.as_ptr() as i64;

                            let fail_block = builder.create_block();
                            let cont_block = builder.create_block();
                            // cond == 0 iken fail'e git
                            let cond_z = builder.ins().icmp_imm(IntCC::Equal, cond_val, 0);
                            builder.ins().brif(cond_z, fail_block, &[], cont_block, &[]);

                            // fail block: __demir_assert_fail(msg_ptr) çağır (geri dönmez, exit eder)
                            builder.switch_to_block(fail_block);
                            let mut sig = self.module.make_signature();
                            sig.params.push(AbiParam::new(types::I64));
                            let fn_id = self
                                .module
                                .declare_function("__demir_assert_fail", Linkage::Import, &sig)
                                .unwrap();
                            let lf = self.module.declare_func_in_func(fn_id, builder.func);
                            let arg = builder.ins().iconst(types::I64, msg_ptr);
                            builder.ins().call(lf, &[arg]);
                            // __demir_assert_fail -> ! döndürür; yine de konta fallback sağla
                            builder.ins().jump(cont_block, &[]);
                            builder.seal_block(fail_block);

                            // devam block'u
                            builder.switch_to_block(cont_block);
                            builder.seal_block(cont_block);
                        }
                    }
                }

                if let Some(term) = &mir_block.terminator {
                    // Inject phi node assignments for successors
                    let succs = match term {
                        Terminator::Goto { target } => vec![*target],
                        Terminator::If { then_target, else_target, .. } => vec![*then_target, *else_target],
                        Terminator::Call { target, .. } => vec![*target],
                        Terminator::Return(_) => vec![],
                    };

                    for succ in succs {
                        for phi in &mir_func.blocks[succ.0].phi_nodes {
                            for (pred_block, arg_local) in &phi.operands {
                                if pred_block.0 == i {
                                    let arg_var = Variable::new(arg_local.0);
                                    let dest_var = Variable::new(phi.dest.0);
                                    let val = builder.use_var(arg_var);
                                    builder.def_var(dest_var, val);
                                }
                            }
                        }
                    }

                    match term {
                        Terminator::Goto { target } => {
                            let t = block_map[target];
                            builder.ins().jump(t, &[]);
                        }
                        Terminator::If { cond, then_target, else_target } => {
                            let c = compile_operand(&mut builder, cond);
                            let c_i8 = builder.ins().icmp_imm(IntCC::NotEqual, c, 0);
                            let t = block_map[then_target];
                            let e = block_map[else_target];
                            
                            builder.ins().brif(c_i8, t, &[], e, &[]);
                        }
                        Terminator::Return(opt_val) => {
                            let ret_val = if let Some(op) = opt_val {
                                compile_operand(&mut builder, op)
                            } else {
                                builder.ins().iconst(types::I64, 0)
                            };
                            builder.ins().return_(&[ret_val]);
                        }
                        Terminator::Call { callee, args, target, destination } => {
                            let local_func = if callee == "std.io.print" {
                                let mut sig = self.module.make_signature();
                                sig.params.push(AbiParam::new(types::I64));
                                let func_id = self.module.declare_function("std_io_print", Linkage::Import, &sig).unwrap();
                                self.module.declare_func_in_func(func_id, builder.func)
                            } else if callee == "std.io.print_str" {
                                let mut sig = self.module.make_signature();
                                sig.params.push(AbiParam::new(types::I64));
                                let func_id = self.module.declare_function("std_io_print_str", Linkage::Import, &sig).unwrap();
                                self.module.declare_func_in_func(func_id, builder.func)
                            } else {
                                let func_id = func_ids.get(callee).expect(&format!("Function not found: {}", callee));
                                self.module.declare_func_in_func(*func_id, builder.func)
                            };
                            
                            let mut arg_vals = Vec::new();
                            for arg in args {
                                arg_vals.push(compile_operand(&mut builder, arg));
                            }
                            
                            let inst = builder.ins().call(local_func, &arg_vals);
                            if callee != "std.io.print" && callee != "std.io.print_str" {
                                let res = builder.inst_results(inst)[0];
                                let dest_var = Variable::new(destination.0);
                                builder.def_var(dest_var, res);
                            }
                            
                            let t = block_map[target];
                            builder.ins().jump(t, &[]);
                        }
                    }
                } else {
                     let zero = builder.ins().iconst(types::I64, 0);
                     builder.ins().return_(&[zero]);
                }
            }

            builder.seal_all_blocks();
            builder.finalize();
            
            let id = func_ids[&mir_func.name];
            self.module.define_function(id, &mut self.ctx).unwrap();
            self.module.clear_context(&mut self.ctx);
        }

        self.module.finalize_definitions().unwrap();
        
        // 'main' giriş noktası yoksa derleyici hatası ver (HashMap panik yerine)
        let main_id = match func_ids.get("main") {
            Some(id) => *id,
            None => return Err("Compile error: entry point 'main' not found".to_string()),
        };
        let code_ptr = self.module.get_finalized_function(main_id);
        
        println!("DEBUG: code_ptr obtained, transmuting...");
        let callable = unsafe { std::mem::transmute::<_, extern "C" fn() -> i64>(code_ptr) };
        println!("DEBUG: Calling callable()...");
        let result = callable();
        println!("DEBUG: callable() returned {}", result);
        
        Ok(result)
    }
}

fn compile_operand(builder: &mut FunctionBuilder, op: &Operand) -> Value {
    match op {
        Operand::ConstantInt(v) => builder.ins().iconst(types::I64, *v),
        Operand::ConstantString(s) => {
            let mut c_str = s.clone();
            c_str.push('\0');
            let leaked: &'static str = Box::leak(c_str.into_boxed_str());
            let ptr = leaked.as_ptr() as i64;
            builder.ins().iconst(types::I64, ptr)
        }
        Operand::Copy(loc) | Operand::Move(loc) => {
            let var = Variable::new(loc.0);
            builder.use_var(var)
        }
    }
}

/// Collects the set of locals that are taken by address (&x / &mut x). These
/// locals must be promoted to stack slots so the borrow yields a stable address.
fn collect_addr_taken(mir_func: &MirFunction) -> std::collections::HashSet<LocalId> {
    let mut set = std::collections::HashSet::new();
    for blk in &mir_func.blocks {
        for stmt in &blk.statements {
            if let Statement::Assign(_, Rvalue::AddressOf(l))
                | Statement::Assign(_, Rvalue::MutAddressOf(l)) = stmt
            {
                set.insert(*l);
            }
        }
    }
    set
}
