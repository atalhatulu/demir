// HIR, ilerideki tip sistemi (Struct/Trait) ve AI-first özelliklerin (Agent/Intent)
// iskelesi olarak kullanılacak alanlar içerir. Henüz uçtan uca tüketilmeyen alanlar
// bilinçli olarak korunur — silmek mimari iskeleyi bozar. Bu yüzden dead-code uyarıları
// bastırılır.
#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    String,
    Bool,
    Char,
    Unit,
    Ref(Box<Type>),
    MutRef(Box<Type>),
    Struct(String),
    Error,
}

#[derive(Debug, Clone)]
pub struct StructFieldInfo {
    pub name: String,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub struct HirProgram {
    pub statements: Vec<HirStatement>,
}

#[derive(Debug, Clone)]
pub enum HirStatement {
    FunctionDeclaration {
        id: usize,
        name: String,
        params: Vec<(usize, Type)>, // param_id, param_type
        return_type: Type,
        requires: Vec<HirExpression>,
        ensures: Vec<HirExpression>,
        body: HirBlock,
    },
    LetDeclaration {
        id: usize,
        name: String,
        value: HirExpression,
    },
    VarDeclaration { // Mutable değişken
        id: usize,
        name: String,
        value: HirExpression,
    },
    Assignment {
        target_id: usize,
        value: HirExpression,
    },
    DerefStore {
        ptr_id: usize,
        value: HirExpression,
    },
    ExpressionStatement(HirExpression),
    IfStatement {
        condition: HirExpression,
        then_branch: HirBlock,
        else_branch: Option<HirBlock>,
    },
    WhileStatement {
        condition: HirExpression,
        body: HirBlock,
    },
    ReturnStatement(Option<HirExpression>),
    StructDeclaration {
        name: String,
        fields: Vec<StructFieldInfo>,
    },
}

#[derive(Debug, Clone)]
pub struct HirBlock {
    pub statements: Vec<HirStatement>,
    pub ty: Type,
}

#[derive(Debug, Clone)]
pub enum HirExpression {
    Integer(i64, Type),
    Float(f64, Type),
    StringLiteral(String, Type),
    Boolean(bool, Type),
    Variable(usize, Type),
    BuiltinFunction(String, Type),
    Binary {
        left: Box<HirExpression>,
        operator: String,
        right: Box<HirExpression>,
        ty: Type,
    },
    Call {
        callee: Box<HirExpression>,
        arguments: Vec<HirExpression>,
        ty: Type,
    },
    MemberAccess {
        object: Box<HirExpression>,
        member: String,
        ty: Type,
    },
    StructInstantiation {
        name: String,
        fields: Vec<(String, HirExpression)>,
        ty: Type,
    },
    Borrow(Box<HirExpression>, Type),
    MutBorrow(Box<HirExpression>, Type),
    Dereference(Box<HirExpression>, Type),
}

impl HirExpression {
    pub fn ty(&self) -> &Type {
        match self {
            HirExpression::Integer(_, t) => t,
            HirExpression::Float(_, t) => t,
            HirExpression::StringLiteral(_, t) => t,
            HirExpression::Boolean(_, t) => t,
            HirExpression::Variable(_, t) => t,
            HirExpression::BuiltinFunction(_, t) => t,
            HirExpression::Binary { ty, .. } => ty,
            HirExpression::Call { ty, .. } => ty,
            HirExpression::MemberAccess { ty, .. } => ty,
            HirExpression::StructInstantiation { ty, .. } => ty,
            HirExpression::Borrow(_, t) => t,
            HirExpression::MutBorrow(_, t) => t,
            HirExpression::Dereference(_, t) => t,
        }
    }
}
