use crate::hir::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BasicBlockId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub enum Ownership {
    Owned,
    Copied,
    Borrowed,
    MutBorrowed,
}

#[derive(Debug, Clone)]
pub struct LocalDecl {
    pub id: LocalId,
    pub ty: Type,
    pub ownership: Ownership,
    pub is_mut: bool,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Copy(LocalId),
    Move(LocalId),
    ConstantInt(i64),
    ConstantString(String),
}

#[derive(Debug, Clone)]
pub enum Rvalue {
    Use(Operand),
    BinaryOp(String, Operand, Operand),
    StructAlloc(usize), // size in bytes
    FieldLoad(LocalId, usize), // ptr local, byte offset
    AddressOf(LocalId),
    MutAddressOf(LocalId),
    Dereference(LocalId),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Assign(LocalId, Rvalue),
    Store(LocalId, usize, Operand), // ptr local, byte offset, value
    Assert(Operand, String), // condition, error_msg
}

#[derive(Debug, Clone)]
pub enum Terminator {
    Goto { target: BasicBlockId },
    If { cond: Operand, then_target: BasicBlockId, else_target: BasicBlockId },
    Return(Option<Operand>),
    Call { callee: String, args: Vec<Operand>, destination: LocalId, target: BasicBlockId },
}

#[derive(Debug, Clone)]
pub struct PhiNode {
    pub dest: LocalId,
    // (Gelen_Blok, Hangi_Deger)
    pub operands: Vec<(BasicBlockId, LocalId)>,
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub phi_nodes: Vec<PhiNode>, // SSA (M8) Phi Düğümleri eklendi
    pub statements: Vec<Statement>,
    pub terminator: Option<Terminator>,
}

#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub param_count: usize,
    pub locals: Vec<LocalDecl>,
    pub blocks: Vec<BasicBlock>,
}

impl MirFunction {
    pub fn new(name: String, param_count: usize) -> Self {
        Self {
            name,
            param_count,
            locals: Vec::new(),
            blocks: Vec::new(),
        }
    }

    pub fn start_block(&self) -> BasicBlockId {
        BasicBlockId(0)
    }
}

use std::fmt;

impl fmt::Display for LocalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "x{}", self.0)
    }
}

impl fmt::Display for BasicBlockId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "bb{}", self.0)
    }
}

impl fmt::Display for Operand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Operand::Copy(id) => write!(f, "{}", id),
            Operand::Move(id) => write!(f, "move {}", id),
            Operand::ConstantInt(v) => write!(f, "{}", v),
            Operand::ConstantString(s) => write!(f, "\"{}\"", s),
        }
    }
}

impl fmt::Display for Rvalue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rvalue::Use(op) => write!(f, "{}", op),
            Rvalue::BinaryOp(op_str, lhs, rhs) => write!(f, "{} {} {}", lhs, op_str, rhs),
            Rvalue::StructAlloc(size) => write!(f, "alloc({})", size),
            Rvalue::FieldLoad(ptr, off) => write!(f, "load({}, offset: {})", ptr, off),
            Rvalue::AddressOf(id) => write!(f, "&{}", id),
            Rvalue::MutAddressOf(id) => write!(f, "&mut {}", id),
            Rvalue::Dereference(id) => write!(f, "*{}", id),
        }
    }
}

impl fmt::Display for MirFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "fn {}({}) {{", self.name, self.param_count)?;
        for (i, block) in self.blocks.iter().enumerate() {
            writeln!(f, "bb{}:", i)?;
            for phi in &block.phi_nodes {
                write!(f, "    {} = phi(", phi.dest)?;
                let ops: Vec<String> = phi.operands.iter().map(|(b, l)| format!("{}: {}", b, l)).collect();
                writeln!(f, "{})", ops.join(", "))?;
            }
            for stmt in &block.statements {
                match stmt {
                    Statement::Assign(dest, rval) => writeln!(f, "    {} = {}", dest, rval)?,
                    Statement::Store(ptr, off, val) => writeln!(f, "    store({}, offset: {}) = {}", ptr, off, val)?,
                    Statement::Assert(cond, msg) => writeln!(f, "    assert({}, \"{}\")", cond, msg)?,
                }
            }
            if let Some(term) = &block.terminator {
                match term {
                    Terminator::Goto { target } => writeln!(f, "    goto {}", target)?,
                    Terminator::If { cond, then_target, else_target } => {
                        writeln!(f, "    if {} goto {} else goto {}", cond, then_target, else_target)?
                    }
                    Terminator::Return(Some(op)) => writeln!(f, "    return {}", op)?,
                    Terminator::Return(None) => writeln!(f, "    return")?,
                    Terminator::Call { callee, args, destination, target } => {
                        let arg_strs: Vec<String> = args.iter().map(|a| format!("{}", a)).collect();
                        writeln!(f, "    {} = call {}({}) -> {}", destination, callee, arg_strs.join(", "), target)?
                    }
                }
            } else {
                writeln!(f, "    <no terminator>")?;
            }
        }
        writeln!(f, "}}")
    }
}
