use crate::hir::*;
use crate::mir::*;
use std::collections::HashMap;

pub struct MirBuilder<'a> {
    function: MirFunction,
    current_block: BasicBlockId,
    next_temp_id: usize,
    structs: &'a HashMap<String, Vec<StructFieldInfo>>,
}

impl<'a> MirBuilder<'a> {
    pub fn new(name: String, param_count: usize, structs: &'a HashMap<String, Vec<StructFieldInfo>>) -> Self {
        Self {
            function: MirFunction::new(name, param_count),
            current_block: BasicBlockId(0),
            next_temp_id: 1000,
            structs,
        }
    }

    pub fn build(mut self, hir_fn: HirStatement) -> Result<MirFunction, String> {
        self.function.blocks.push(BasicBlock {
            phi_nodes: Vec::new(),
            statements: Vec::new(),
            terminator: None,
        });

        match hir_fn {
            HirStatement::FunctionDeclaration { params, requires, ensures, body, .. } => {
                for (p_id, p_ty) in params {
                    self.function.locals.push(LocalDecl {
                        id: LocalId(p_id),
                        ty: p_ty,
                        ownership: Ownership::Owned,
                        is_mut: false, // params immutable by default in V0.1
                    });
                }
                
                for (i, req) in requires.into_iter().enumerate() {
                    let op = self.build_operand(req)?;
                    self.push_stmt(Statement::Assert(op, format!("pre-condition {} failed", i + 1)));
                }

                self.build_block(body)?;

                for (i, ens) in ensures.into_iter().enumerate() {
                    let op = self.build_operand(ens)?;
                    self.push_stmt(Statement::Assert(op, format!("post-condition {} failed", i + 1)));
                }

                if self.function.blocks[self.current_block.0].terminator.is_none() {
                    self.function.blocks[self.current_block.0].terminator = Some(Terminator::Return(None));
                }
                Ok(self.function)
            }
            HirStatement::StructDeclaration { .. } => {
                // Not lowered to MIR yet
                Ok(self.function)
            }
            _ => Err("Expected FunctionDeclaration or StructDeclaration".to_string()),
        }
    }

    fn new_temp(&mut self, ty: Type) -> LocalId {
        let id = LocalId(self.next_temp_id);
        self.next_temp_id += 1;
        let ownership = match ty {
            Type::Int | Type::Float | Type::Bool | Type::Char | Type::Unit => Ownership::Copied,
            Type::String => Ownership::Owned, 
            Type::Ref(_) => Ownership::Borrowed,
            Type::MutRef(_) => Ownership::MutBorrowed,
            _ => Ownership::Owned,
        };
        self.function.locals.push(LocalDecl { id, ty, ownership, is_mut: false });
        id
    }

    fn new_block(&mut self) -> BasicBlockId {
        let id = BasicBlockId(self.function.blocks.len());
        self.function.blocks.push(BasicBlock {
            phi_nodes: Vec::new(),
            statements: Vec::new(),
            terminator: None,
        });
        id
    }

    fn push_stmt(&mut self, stmt: Statement) {
        self.function.blocks[self.current_block.0].statements.push(stmt);
    }

    fn build_block(&mut self, block: HirBlock) -> Result<(), String> {
        for stmt in block.statements {
            self.build_statement(stmt)?;
        }
        Ok(())
    }

    fn build_statement(&mut self, stmt: HirStatement) -> Result<(), String> {
        if self.function.blocks[self.current_block.0].terminator.is_some() {
            return Ok(());
        }

        match stmt {
            HirStatement::LetDeclaration { id, value, .. } => {
                let local = LocalId(id);
                self.function.locals.push(LocalDecl { id: local, ty: value.ty().clone(), ownership: Ownership::Owned, is_mut: false });
                let rval = self.build_rvalue(value)?;
                self.push_stmt(Statement::Assign(local, rval));
                Ok(())
            }
            HirStatement::VarDeclaration { id, value, .. } => {
                let local = LocalId(id);
                self.function.locals.push(LocalDecl { id: local, ty: value.ty().clone(), ownership: Ownership::Owned, is_mut: true });
                let rval = self.build_rvalue(value)?;
                self.push_stmt(Statement::Assign(local, rval));
                Ok(())
            }
            HirStatement::Assignment { target_id, value } => {
                let rval = self.build_rvalue(value)?;
                self.push_stmt(Statement::Assign(LocalId(target_id), rval));
                Ok(())
            }
            HirStatement::ExpressionStatement(expr) => {
                let _ = self.build_rvalue(expr)?;
                Ok(())
            }
            HirStatement::IfStatement { condition, then_branch, else_branch } => {
                let cond_op = self.build_operand(condition)?;
                
                let then_bb = self.new_block();
                let else_bb = self.new_block();
                let merge_bb = self.new_block();

                self.function.blocks[self.current_block.0].terminator = Some(Terminator::If {
                    cond: cond_op,
                    then_target: then_bb,
                    else_target: else_bb,
                });

                self.current_block = then_bb;
                self.build_block(then_branch)?;
                if self.function.blocks[self.current_block.0].terminator.is_none() {
                    self.function.blocks[self.current_block.0].terminator = Some(Terminator::Goto { target: merge_bb });
                }

                self.current_block = else_bb;
                if let Some(eb) = else_branch {
                    self.build_block(eb)?;
                }
                if self.function.blocks[self.current_block.0].terminator.is_none() {
                    self.function.blocks[self.current_block.0].terminator = Some(Terminator::Goto { target: merge_bb });
                }

                self.current_block = merge_bb;
                Ok(())
            }
            HirStatement::WhileStatement { condition, body } => {
                let cond_bb = self.new_block();
                let body_bb = self.new_block();
                let exit_bb = self.new_block();

                self.function.blocks[self.current_block.0].terminator = Some(Terminator::Goto { target: cond_bb });
                
                self.current_block = cond_bb;
                let cond_op = self.build_operand(condition)?;
                
                self.function.blocks[self.current_block.0].terminator = Some(Terminator::If {
                    cond: cond_op,
                    then_target: body_bb,
                    else_target: exit_bb,
                });

                self.current_block = body_bb;
                self.build_block(body)?;
                if self.function.blocks[self.current_block.0].terminator.is_none() {
                    self.function.blocks[self.current_block.0].terminator = Some(Terminator::Goto { target: cond_bb });
                }

                self.current_block = exit_bb;
                Ok(())
            }
            HirStatement::ReturnStatement(opt_expr) => {
                let ret_op = if let Some(expr) = opt_expr {
                    Some(self.build_operand(expr)?)
                } else {
                    None
                };
                self.function.blocks[self.current_block.0].terminator = Some(Terminator::Return(ret_op));
                Ok(())
            }
            _ => Err("Statement not supported in MIR builder yet".to_string()),
        }
    }

    fn build_rvalue(&mut self, expr: HirExpression) -> Result<Rvalue, String> {
        match expr {
            HirExpression::Binary { left, operator, right, .. } => {
                let lhs = self.build_operand(*left)?;
                let rhs = self.build_operand(*right)?;
                Ok(Rvalue::BinaryOp(operator, lhs, rhs))
            }
            HirExpression::Call { callee, arguments, ty } => {
                let dest = self.new_temp(ty);
                let target_bb = self.new_block();
                
                let callee_name = match *callee {
                    HirExpression::BuiltinFunction(name, _) => name,
                    _ => return Err("Only builtin function calls supported in V0.1 basic slice MIR".to_string()),
                };

                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.build_operand(arg)?);
                }

                self.function.blocks[self.current_block.0].terminator = Some(Terminator::Call {
                    callee: callee_name,
                    args,
                    destination: dest,
                    target: target_bb,
                });
                
                self.current_block = target_bb;
                Ok(Rvalue::Use(Operand::Copy(dest)))
            }
            HirExpression::StructInstantiation { name, fields, .. } => {
                let size = if let Some(struct_fields) = self.structs.get(&name) {
                    struct_fields.len() * 8 // Assuming 8 bytes per field (i64 for now)
                } else {
                    0
                };
                
                let ptr = self.new_temp(Type::Struct(name.clone()));
                self.push_stmt(Statement::Assign(ptr, Rvalue::StructAlloc(size)));
                
                for (field_name, expr) in fields {
                    let mut offset = 0;
                    if let Some(struct_fields) = self.structs.get(&name) {
                        for sf in struct_fields {
                            if sf.name == *field_name {
                                break;
                            }
                            offset += 8;
                        }
                    }
                    let val = self.build_operand(expr)?;
                    self.push_stmt(Statement::Store(ptr, offset, val));
                }
                
                Ok(Rvalue::Use(Operand::Copy(ptr)))
            }
            HirExpression::MemberAccess { object, member, ty } => {
                let ptr_op = self.build_operand(*object.clone())?;
                let ptr_local = match ptr_op {
                    Operand::Move(l) | Operand::Copy(l) => l,
                    _ => return Err("Expected local for struct member access".to_string()),
                };
                
                let obj_ty = object.ty();
                let mut offset = 0;
                
                if let Type::Struct(struct_name) = obj_ty {
                    if let Some(struct_fields) = self.structs.get(struct_name) {
                        for sf in struct_fields {
                            if sf.name == *member {
                                break;
                            }
                            offset += 8;
                        }
                    }
                }
                
                let dest = self.new_temp(ty.clone());
                self.push_stmt(Statement::Assign(dest, Rvalue::FieldLoad(ptr_local, offset)));
                Ok(Rvalue::Use(Operand::Copy(dest)))
            }
            HirExpression::Borrow(inner, ty) => {
                let op = self.build_operand(*inner)?;
                let local = match op {
                    Operand::Copy(l) | Operand::Move(l) => l,
                    _ => return Err("Cannot borrow a constant".to_string()),
                };
                let dest = self.new_temp(ty.clone());
                self.push_stmt(Statement::Assign(dest, Rvalue::AddressOf(local)));
                Ok(Rvalue::Use(Operand::Copy(dest)))
            }
            HirExpression::MutBorrow(inner, ty) => {
                let op = self.build_operand(*inner)?;
                let local = match op {
                    Operand::Copy(l) | Operand::Move(l) => l,
                    _ => return Err("Cannot borrow a constant mutably".to_string()),
                };
                let dest = self.new_temp(ty.clone());
                self.push_stmt(Statement::Assign(dest, Rvalue::MutAddressOf(local)));
                Ok(Rvalue::Use(Operand::Copy(dest)))
            }
            HirExpression::Dereference(inner, ty) => {
                let op = self.build_operand(*inner)?;
                let local = match op {
                    Operand::Copy(l) | Operand::Move(l) => l,
                    _ => return Err("Cannot dereference a constant".to_string()),
                };
                let dest = self.new_temp(ty.clone());
                self.push_stmt(Statement::Assign(dest, Rvalue::Dereference(local)));
                Ok(Rvalue::Use(Operand::Copy(dest)))
            }
            _ => {
                let op = self.build_operand(expr)?;
                Ok(Rvalue::Use(op))
            }
        }
    }

    fn build_operand(&mut self, expr: HirExpression) -> Result<Operand, String> {
        match expr {
            HirExpression::Integer(val, _) => Ok(Operand::ConstantInt(val)),
            HirExpression::Boolean(val, _) => Ok(Operand::ConstantInt(if val { 1 } else { 0 })),
            HirExpression::StringLiteral(val, _) => Ok(Operand::ConstantString(val)),
            HirExpression::Variable(id, ty) => {
                if ty == Type::String {
                    Ok(Operand::Move(LocalId(id)))
                } else {
                    Ok(Operand::Copy(LocalId(id)))
                }
            }
            _ => {
                let rval = self.build_rvalue(expr.clone())?;
                let temp = self.new_temp(expr.ty().clone());
                self.push_stmt(Statement::Assign(temp, rval));
                Ok(Operand::Copy(temp))
            }
        }
    }
}
