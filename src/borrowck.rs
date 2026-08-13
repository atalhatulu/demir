use crate::mir::*;
use crate::hir::Type;
use std::collections::HashMap;
use std::collections::HashSet;

pub struct BorrowChecker {
    moved_locals: HashSet<LocalId>,
    initialized_locals: HashSet<LocalId>,
    locals: HashMap<LocalId, LocalDecl>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            moved_locals: HashSet::new(),
            initialized_locals: HashSet::new(),
            locals: HashMap::new(),
        }
    }

    pub fn check(&mut self, function: &MirFunction) -> Result<(), String> {
        for local in &function.locals {
            self.locals.insert(local.id, local.clone());
        }

        for block in &function.blocks {
            for stmt in &block.statements {
                match stmt {
                    Statement::Assign(dest, rval) => {
                        self.check_rvalue(rval)?;
                        
                        if self.initialized_locals.contains(dest) {
                            if let Some(decl) = self.locals.get(dest) {
                                if !decl.is_mut {
                                    return Err(format!("Cannot reassign immutable variable {}", dest));
                                }
                            }
                        }
                        
                        self.initialized_locals.insert(*dest);
                        self.moved_locals.remove(dest);
                    }
                    Statement::Store(ptr, _, val) => {
                        self.check_operand(val)?;
                        if let Some(ptr_decl) = self.locals.get(ptr) {
                            let is_mut_ref = matches!(ptr_decl.ty, Type::MutRef(_));
                            if !ptr_decl.is_mut && !is_mut_ref && ptr.0 < 1000 {
                                return Err(format!("Cannot mutate field of immutable variable {}", ptr.0));
                            }
                        }
                    }
                    Statement::Assert(cond, _) => {
                        self.check_operand(cond)?;
                    }
                }
            }
            if let Some(terminator) = &block.terminator {
                self.check_terminator(terminator)?;
            }
        }
        Ok(())
    }

    fn check_rvalue(&mut self, rval: &Rvalue) -> Result<(), String> {
        match rval {
            Rvalue::Use(op) => self.check_operand(op),
            Rvalue::BinaryOp(_, left, right) => {
                self.check_operand(left)?;
                self.check_operand(right)
            }
            Rvalue::StructAlloc(_) => Ok(()),
            Rvalue::FieldLoad(ptr, _) => {
                self.check_operand(&Operand::Copy(*ptr))
            }
            Rvalue::AddressOf(ptr) => {
                self.check_operand(&Operand::Copy(*ptr))
            }
            Rvalue::MutAddressOf(ptr) => {
                if let Some(decl) = self.locals.get(ptr) {
                    if !decl.is_mut && ptr.0 < 1000 {
                        return Err(format!("Cannot borrow immutable variable {} as mutable", ptr.0));
                    }
                }
                self.check_operand(&Operand::Copy(*ptr))
            }
            Rvalue::Dereference(ptr) => {
                self.check_operand(&Operand::Copy(*ptr))
            }
        }
    }

    fn check_operand(&mut self, op: &Operand) -> Result<(), String> {
        match op {
            Operand::Move(local) => {
                if self.moved_locals.contains(local) {
                    return Err(format!("Use of moved value (Local {})! Ownership has already been transferred.", local.0));
                }
                // Değer move edildi, listeye ekle.
                self.moved_locals.insert(*local);
                Ok(())
            }
            Operand::Copy(local) => {
                if self.moved_locals.contains(local) {
                    return Err(format!("Use of moved value (Local {})! Cannot copy a value that has already been moved.", local.0));
                }
                Ok(())
            }
            _ => Ok(()), // Constant literal'ler sorunsuzdur
        }
    }

    fn check_terminator(&mut self, term: &Terminator) -> Result<(), String> {
        match term {
            Terminator::If { cond, .. } => self.check_operand(cond),
            Terminator::Call { args, destination, .. } => {
                for arg in args {
                    self.check_operand(arg)?;
                }
                
                if self.initialized_locals.contains(destination) {
                    if let Some(decl) = self.locals.get(destination) {
                        if !decl.is_mut {
                            return Err(format!("Cannot reassign immutable variable {}", destination));
                        }
                    }
                }
                self.initialized_locals.insert(*destination);
                self.moved_locals.remove(destination);
                Ok(())
            }
            _ => Ok(())
        }
    }
}
