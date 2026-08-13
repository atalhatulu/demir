use crate::ast::*;
use crate::token::{Token, TokenType};
use crate::lexer::Lexer;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Token,
    peek: Token,
    /// Names of structs declared so far. Used to disambiguate `Identifier { ... }`:
    /// a struct literal only starts with an identifier that is a known struct name,
    /// so a `while i < n {` block body (where the condition operand is a plain
    /// identifier) is not misread as a struct instantiation.
    known_structs: std::collections::HashSet<String>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current = lexer.next_token();
        let peek = lexer.next_token();
        Self {
            lexer,
            current,
            peek,
            known_structs: std::collections::HashSet::new(),
        }
    }

    fn advance(&mut self) {
        self.current = self.peek.clone();
        self.peek = self.lexer.next_token();
    }

    fn check(&self, kind: &TokenType) -> bool {
        &self.current.kind == kind
    }

    fn match_token(&mut self, kind: TokenType) -> bool {
        if self.check(&kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenType) -> Result<(), String> {
        if self.check(&kind) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, got {:?}", kind, self.current.kind))
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut statements = Vec::new();
        while self.current.kind != TokenType::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current.kind {
            TokenType::Fn => self.parse_function_declaration(),
            TokenType::Let => self.parse_let_declaration(),
            TokenType::Var => self.parse_var_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Struct => self.parse_struct_declaration(),
            TokenType::Import => self.parse_import_statement(),
            _ => self.parse_expression_or_assignment(),
        }
    }

    fn parse_type_info(&mut self) -> Result<TypeInfo, String> {
        if self.match_token(TokenType::Ampersand) {
            if self.match_token(TokenType::Mut) {
                let inner = self.parse_type_info()?;
                return Ok(TypeInfo::MutRef(Box::new(inner)));
            } else {
                let inner = self.parse_type_info()?;
                return Ok(TypeInfo::Ref(Box::new(inner)));
            }
        }

        match &self.current.kind {
            TokenType::Ident(name) => {
                let ty = match name.as_str() {
                    "Int" => TypeInfo::Int,
                    "Float" => TypeInfo::Float,
                    "String" => TypeInfo::String,
                    "Bool" => TypeInfo::Bool,
                    "Char" => TypeInfo::Char,
                    "Unit" => TypeInfo::Unit,
                    _ => TypeInfo::Custom(name.clone()),
                };
                self.advance();
                Ok(ty)
            }
            _ => Err(format!("Expected type name, got {:?}", self.current.kind)),
        }
    }

    fn parse_import_statement(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Import)?;
        let mut path = String::new();
        if let TokenType::StringLiteral(s) = &self.current.kind {
            path = s.clone();
            self.advance();
        } else {
            // handle dot-separated identifiers (e.g. std.io)
            loop {
                if let TokenType::Ident(s) = &self.current.kind {
                    path.push_str(s);
                    self.advance();
                } else {
                    return Err(format!("Expected identifier in import path, got {:?}", self.current.kind));
                }
                
                if self.match_token(TokenType::Dot) {
                    path.push('.');
                } else {
                    break;
                }
            }
        }
        self.expect(TokenType::Semicolon)?;
        Ok(Statement::Import(path))
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Fn)?;
        
        let name = match &self.current.kind {
            TokenType::Ident(n) => n.clone(),
            _ => return Err("Expected function name".to_string()),
        };
        self.advance();

        self.expect(TokenType::OpenParen)?;
        let mut params = Vec::new();
        if !self.check(&TokenType::CloseParen) {
            loop {
                let param_name = match &self.current.kind {
                    TokenType::Ident(n) => n.clone(),
                    _ => return Err("Expected parameter name".to_string()),
                };
                self.advance();
                self.expect(TokenType::Colon)?;
                let param_type = self.parse_type_info()?;
                params.push((param_name, param_type));

                if !self.match_token(TokenType::Comma) {
                    break;
                }
            }
        }
        self.expect(TokenType::CloseParen)?;

        let mut return_type = None;
        if self.match_token(TokenType::Arrow) {
            return_type = Some(self.parse_type_info()?);
        }

        let mut requires = Vec::new();
        while self.match_token(TokenType::Requires) {
            requires.push(self.parse_expression()?);
        }

        let mut ensures = Vec::new();
        while self.match_token(TokenType::Ensures) {
            ensures.push(self.parse_expression()?);
        }

        let body = self.parse_block()?;
        Ok(Statement::FunctionDeclaration { name, params, return_type, requires, ensures, body })
    }

    fn parse_struct_declaration(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Struct)?;
        
        let name = match &self.current.kind {
            TokenType::Ident(n) => n.clone(),
            _ => return Err("Expected struct name".to_string()),
        };
        self.advance();
        self.known_structs.insert(name.clone());

        self.expect(TokenType::OpenBrace)?;
        let mut fields = Vec::new();
        while !self.check(&TokenType::CloseBrace) && !self.check(&TokenType::Eof) {
            let field_name = match &self.current.kind {
                TokenType::Ident(n) => n.clone(),
                _ => return Err("Expected field name".to_string()),
            };
            self.advance();
            self.expect(TokenType::Colon)?;
            let field_type = self.parse_type_info()?;
            fields.push((field_name, field_type));

            if self.match_token(TokenType::Comma) {
                continue;
            } else if !self.check(&TokenType::CloseBrace) {
                // Semicolon or comma can separate fields, or just newline (in a real lexer). We'll allow comma or nothing before CloseBrace.
                // Or require comma:
            }
        }
        self.expect(TokenType::CloseBrace)?;

        Ok(Statement::StructDeclaration { name, fields })
    }

    fn parse_let_declaration(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Let)?;
        
        let name = match &self.current.kind {
            TokenType::Ident(n) => n.clone(),
            _ => return Err("Expected identifier in let".to_string()),
        };
        self.advance();

        let mut ty = None;
        if self.match_token(TokenType::Colon) {
            ty = Some(self.parse_type_info()?);
        }

        self.expect(TokenType::Assign)?;
        let value = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;

        Ok(Statement::LetDeclaration { name, ty, value })
    }

    fn parse_var_declaration(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Var)?;
        
        let name = match &self.current.kind {
            TokenType::Ident(n) => n.clone(),
            _ => return Err("Expected identifier in var".to_string()),
        };
        self.advance();

        let mut ty = None;
        if self.match_token(TokenType::Colon) {
            ty = Some(self.parse_type_info()?);
        }

        self.expect(TokenType::Assign)?;
        let value = self.parse_expression()?;
        self.expect(TokenType::Semicolon)?;

        Ok(Statement::VarDeclaration { name, ty, value })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::If)?;
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block()?;
        
        let mut else_branch = None;
        if self.match_token(TokenType::Else) {
            if self.check(&TokenType::If) {
                // else if
                let else_if = self.parse_if_statement()?;
                else_branch = Some(Block { statements: vec![else_if] });
            } else {
                else_branch = Some(self.parse_block()?);
            }
        }
        
        Ok(Statement::IfStatement { condition, then_branch, else_branch })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::While)?;
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        Ok(Statement::WhileStatement { condition, body })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, String> {
        self.expect(TokenType::Return)?;
        let mut value = None;
        if !self.check(&TokenType::Semicolon) {
            value = Some(self.parse_expression()?);
        }
        self.expect(TokenType::Semicolon)?;
        Ok(Statement::ReturnStatement(value))
    }

    fn parse_expression_or_assignment(&mut self) -> Result<Statement, String> {
        let target = self.parse_expression()?;
        
        if self.match_token(TokenType::Assign) {
            let value = self.parse_expression()?;
            self.expect(TokenType::Semicolon)?;
            Ok(Statement::Assignment { target, value })
        } else {
            self.expect(TokenType::Semicolon)?;
            Ok(Statement::ExpressionStatement(target))
        }
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        self.expect(TokenType::OpenBrace)?;
        let mut statements = Vec::new();

        while !self.check(&TokenType::CloseBrace) && !self.check(&TokenType::Eof) {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenType::CloseBrace)?;
        Ok(Block { statements })
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_logical_or()
    }

    fn parse_logical_or(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_logical_and()?;
        while self.check(&TokenType::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            expr = Expression::Binary { left: Box::new(expr), operator: "||".to_string(), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_equality()?;
        while self.check(&TokenType::And) {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expression::Binary { left: Box::new(expr), operator: "&&".to_string(), right: Box::new(right) };
        }
        Ok(expr)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_comparison()?;

        while self.check(&TokenType::Eq) || self.check(&TokenType::NotEq) {
            let operator = match self.current.kind {
                TokenType::Eq => "==",
                TokenType::NotEq => "!=",
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_comparison()?;
            expr = Expression::Binary { left: Box::new(expr), operator: operator.to_string(), right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_addition()?;

        while self.check(&TokenType::Less) || self.check(&TokenType::LessEq) ||
              self.check(&TokenType::Greater) || self.check(&TokenType::GreaterEq) {
            let operator = match self.current.kind {
                TokenType::Less => "<",
                TokenType::LessEq => "<=",
                TokenType::Greater => ">",
                TokenType::GreaterEq => ">=",
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_addition()?;
            expr = Expression::Binary { left: Box::new(expr), operator: operator.to_string(), right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_multiplication()?;

        while self.check(&TokenType::Plus) || self.check(&TokenType::Minus) {
            let operator = match self.current.kind {
                TokenType::Plus => "+",
                TokenType::Minus => "-",
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_multiplication()?;
            expr = Expression::Binary { left: Box::new(expr), operator: operator.to_string(), right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_unary()?;

        while self.check(&TokenType::Star) || self.check(&TokenType::Slash) || self.check(&TokenType::Percent) {
            let operator = match self.current.kind {
                TokenType::Star => "*",
                TokenType::Slash => "/",
                TokenType::Percent => "%",
                _ => unreachable!(),
            };
            self.advance();
            let right = self.parse_unary()?;
            expr = Expression::Binary { left: Box::new(expr), operator: operator.to_string(), right: Box::new(right) };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        if self.match_token(TokenType::Ampersand) {
            if self.match_token(TokenType::Mut) {
                let expr = self.parse_unary()?;
                return Ok(Expression::MutBorrow(Box::new(expr)));
            } else {
                let expr = self.parse_unary()?;
                return Ok(Expression::Borrow(Box::new(expr)));
            }
        }
        if self.match_token(TokenType::Star) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Dereference(Box::new(expr)));
        }
        self.parse_call()
    }

    fn parse_call(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            if self.match_token(TokenType::OpenParen) {
                let mut arguments = Vec::new();
                if !self.check(&TokenType::CloseParen) {
                    arguments.push(self.parse_expression()?);
                    while self.match_token(TokenType::Comma) {
                        arguments.push(self.parse_expression()?);
                    }
                }
                self.expect(TokenType::CloseParen)?;
                expr = Expression::Call { callee: Box::new(expr), arguments };
            } else if self.match_token(TokenType::Dot) {
                let member = match &self.current.kind {
                    TokenType::Ident(n) => n.clone(),
                    _ => return Err(format!("Expected identifier after '.', got {:?}", self.current.kind)),
                };
                self.advance();
                expr = Expression::MemberAccess { object: Box::new(expr), member };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match &self.current.kind {
            TokenType::IntLiteral(v) => {
                let val = *v;
                self.advance();
                Ok(Expression::Integer(val))
            }
            TokenType::FloatLiteral(v) => {
                let val = *v;
                self.advance();
                Ok(Expression::Float(val))
            }
            TokenType::StringLiteral(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expression::StringLiteral(val))
            }
            TokenType::Ident(n) => {
                let val = n.clone();
                if val == "true" {
                    self.advance();
                    Ok(Expression::Boolean(true))
                } else if val == "false" {
                    self.advance();
                    Ok(Expression::Boolean(false))
                } else {
                    self.advance();
                    // Struct instantiation only if this identifier is a known struct name.
                    // Otherwise `Identifier {` is most likely the start of an if/while
                    // block body following an identifier operand (e.g. `while i < n {`),
                    // not a struct literal — without this guard that body brace is
                    // misparsed as a struct field list and everything breaks.
                    if self.known_structs.contains(&val) && self.match_token(TokenType::OpenBrace) {
                        let mut fields = Vec::new();
                        while !self.check(&TokenType::CloseBrace) && !self.check(&TokenType::Eof) {
                            let field_name = match &self.current.kind {
                                TokenType::Ident(n) => n.clone(),
                                _ => return Err("Expected field name in struct instantiation".to_string()),
                            };
                            self.advance();
                            self.expect(TokenType::Colon)?;
                            let field_expr = self.parse_expression()?;
                            fields.push((field_name, field_expr));

                            if self.match_token(TokenType::Comma) {
                                continue;
                            }
                        }
                        self.expect(TokenType::CloseBrace)?;
                        Ok(Expression::StructInstantiation { name: val, fields })
                    } else {
                        Ok(Expression::Identifier(val))
                    }
                }
            }
            TokenType::OpenParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenType::CloseParen)?;
                Ok(expr)
            }
            _ => Err(format!("Unexpected token in expression: {:?}", self.current.kind)),
        }
    }
}
