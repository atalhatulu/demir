#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Fn, Let, Var, Mut, Ref, If, Else, While, Match, Return, Unsafe,
    Module, Import, Export, Trait, Impl, Requires, Ensures,
    Intent, Select, Where, Order, Agent, Capability, State, Ask,
    
    // Identifiers and Literals
    Ident(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    
    // Operators and Symbols
    Plus, Minus, Star, Slash, Percent,
    Assign, Eq, NotEq, Less, LessEq, Greater, GreaterEq,
    And, Or, Bang, Ampersand, // &&, ||, !, &
    Dot, DoubleColon, Arrow, FatArrow, // ., ::, ->, =>
    
    // Delimiters
    OpenParen, CloseParen, OpenBrace, CloseBrace, OpenBracket, CloseBracket,
    Comma, Colon, Semicolon,
    
    Struct,
    Eof,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Span {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenType,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenType, span: Span) -> Self {
        Self { kind, span }
    }
}
