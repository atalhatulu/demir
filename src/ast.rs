#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Int,
    Float,
    String,
    Bool,
    Char,
    Unit,
    Ref(Box<TypeInfo>),
    MutRef(Box<TypeInfo>),
    Custom(String),
}

#[derive(Debug, Clone)]
pub enum Statement {
    Import(String),
    FunctionDeclaration {
        name: String,
        params: Vec<(String, TypeInfo)>,
        return_type: Option<TypeInfo>,
        requires: Vec<Expression>,
        ensures: Vec<Expression>,
        body: Block,
    },
    LetDeclaration { // Immutable
        name: String,
        ty: Option<TypeInfo>,
        value: Expression,
    },
    VarDeclaration { // Mutable
        name: String,
        ty: Option<TypeInfo>,
        value: Expression,
    },
    Assignment {
        target: Expression,
        value: Expression,
    },
    ExpressionStatement(Expression),
    IfStatement {
        condition: Expression,
        then_branch: Block,
        else_branch: Option<Block>,
    },
    WhileStatement {
        condition: Expression,
        body: Block,
    },
    ReturnStatement(Option<Expression>),
    StructDeclaration {
        name: String,
        fields: Vec<(String, TypeInfo)>,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(i64),
    Float(f64),
    StringLiteral(String),
    Boolean(bool),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: String,
        right: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    MemberAccess {
        object: Box<Expression>,
        member: String,
    },
    StructInstantiation {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    Borrow(Box<Expression>),
    MutBorrow(Box<Expression>),
    Dereference(Box<Expression>),
}
