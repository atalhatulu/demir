use crate::token::{Span, Token, TokenType};
use std::iter::Peekable;
use std::str::Chars;

pub struct Lexer<'a> {
    chars: Peekable<Chars<'a>>,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            chars: source.chars().peekable(),
            line: 1,
            column: 1,
        }
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    fn peek(&mut self) -> Option<&char> {
        self.chars.peek()
    }

    fn match_keyword_or_ident(s: &str) -> TokenType {
        match s {
            "fn" => TokenType::Fn,
            "let" => TokenType::Let,
            "var" => TokenType::Var,
            "mut" => TokenType::Mut,
            "ref" => TokenType::Ref,
            "if" => TokenType::If,
            "else" => TokenType::Else,
            "while" => TokenType::While,
            "match" => TokenType::Match,
            "return" => TokenType::Return,
            "unsafe" => TokenType::Unsafe,
            "module" => TokenType::Module,
            "import" => TokenType::Import,
            "export" => TokenType::Export,
            "trait" => TokenType::Trait,
            "impl" => TokenType::Impl,
            "struct" => TokenType::Struct,
            "requires" => TokenType::Requires,
            "ensures" => TokenType::Ensures,
            "intent" => TokenType::Intent,
            "select" => TokenType::Select,
            "where" => TokenType::Where,
            "order" => TokenType::Order,
            "agent" => TokenType::Agent,
            "capability" => TokenType::Capability,
            "state" => TokenType::State,
            "ask" => TokenType::Ask,
            _ => TokenType::Ident(s.to_string()),
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(&c) if c.is_whitespace() => {
                    self.advance();
                }
                Some(&'/') => {
                    // Check if it's a comment
                    let mut iter_clone = self.chars.clone();
                    iter_clone.next(); // Consume '/'
                    if let Some(&'/') = iter_clone.peek() {
                        // It's a line comment, consume until newline
                        while let Some(cc) = self.advance() {
                            if cc == '\n' {
                                break;
                            }
                        }
                    } else {
                        // Not a comment, just a slash
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();

        let start_line = self.line;
        let start_column = self.column;

        let kind = if let Some(c) = self.advance() {
            match c {
                '+' => TokenType::Plus,
                '*' => TokenType::Star,
                '/' => TokenType::Slash,
                '%' => TokenType::Percent,
                '(' => TokenType::OpenParen,
                ')' => TokenType::CloseParen,
                '{' => TokenType::OpenBrace,
                '}' => TokenType::CloseBrace,
                '[' => TokenType::OpenBracket,
                ']' => TokenType::CloseBracket,
                ',' => TokenType::Comma,
                ';' => TokenType::Semicolon,
                
                // Multi-character matchers
                '-' => {
                    if let Some(&'>') = self.peek() {
                        self.advance();
                        TokenType::Arrow
                    } else {
                        TokenType::Minus
                    }
                }
                '=' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        TokenType::Eq
                    } else if let Some(&'>') = self.peek() {
                        self.advance();
                        TokenType::FatArrow
                    } else {
                        TokenType::Assign
                    }
                }
                '!' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        TokenType::NotEq
                    } else {
                        TokenType::Bang
                    }
                }
                '<' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        TokenType::LessEq
                    } else {
                        TokenType::Less
                    }
                }
                '>' => {
                    if let Some(&'=') = self.peek() {
                        self.advance();
                        TokenType::GreaterEq
                    } else {
                        TokenType::Greater
                    }
                }
                ':' => {
                    if let Some(&':') = self.peek() {
                        self.advance();
                        TokenType::DoubleColon
                    } else {
                        TokenType::Colon
                    }
                }
                '.' => TokenType::Dot,
                '&' => {
                    if let Some(&'&') = self.peek() {
                        self.advance();
                        TokenType::And
                    } else {
                        TokenType::Ampersand
                    }
                }
                '|' => {
                    if let Some(&'|') = self.peek() {
                        self.advance();
                        TokenType::Or
                    } else {
                        TokenType::Error("Expected '||'".to_string())
                    }
                }
                '\'' => {
                    // Karakter literal: 'a' veya '\n' gibi tek karakter + escape
                    let char_val = match self.peek() {
                        Some(&'\\') => {
                            self.advance(); // consume backslash
                            match self.advance() {
                                Some('n') => '\n',
                                Some('t') => '\t',
                                Some('r') => '\r',
                                Some('\\') => '\\',
                                Some('\'') => '\'',
                                Some('\0') => '\0',
                                Some(c) => c,
                                None => {
                                    return Token {
                                        kind: TokenType::Error("unterminated char literal".to_string()),
                                        span: Span { start_line: self.line, start_col: self.column, end_line: self.line, end_col: self.column },
                                    };
                                }
                            }
                        }
                        Some(&c) => {
                            self.advance();
                            c
                        }
                        None => {
                            return Token {
                                kind: TokenType::Error("unterminated char literal".to_string()),
                                span: Span { start_line: self.line, start_col: self.column, end_line: self.line, end_col: self.column },
                            };
                        }
                    };
                    // Kapanış tırnağını doğrula
                    match self.advance() {
                        Some('\'') => TokenType::CharLiteral(char_val),
                        _ => TokenType::Error("unterminated char literal".to_string()),
                    }
                }
                '"' => {
                    let mut string_val = String::new();
                    let mut terminated = false;
                    while let Some(&next_c) = self.peek() {
                        if next_c == '"' {
                            self.advance();
                            terminated = true;
                            break;
                        }
                        string_val.push(next_c);
                        self.advance();
                    }
                    if terminated {
                        TokenType::StringLiteral(string_val)
                    } else {
                        TokenType::Error("unterminated string literal".to_string())
                    }
                }
                c if c.is_alphabetic() || c == '_' => {
                    let mut ident = String::from(c);
                    while let Some(&next_c) = self.peek() {
                        if next_c.is_alphanumeric() || next_c == '_' {
                            ident.push(next_c);
                            self.advance();
                        } else {
                            break;
                        }
                    }
                    Self::match_keyword_or_ident(&ident)
                }
                c if c.is_ascii_digit() => {
                    let mut num_str = String::from(c);
                    let mut is_float = false;

                    while let Some(&next_c) = self.peek() {
                        if next_c.is_ascii_digit() {
                            num_str.push(next_c);
                            self.advance();
                        } else if next_c == '.' {
                            // Check if next is also a dot (like .. range) or something else
                            // For V0.1, we assume . followed by digit is float.
                            let mut lookahead = self.chars.clone();
                            lookahead.next(); // consume '.'
                            if let Some(la_c) = lookahead.peek() {
                                if la_c.is_ascii_digit() {
                                    is_float = true;
                                    num_str.push(next_c);
                                    self.advance(); // consume '.'
                                    continue;
                                }
                            }
                            break;
                        } else {
                            break;
                        }
                    }

                    if is_float {
                        match num_str.parse::<f64>() {
                            Ok(val) => TokenType::FloatLiteral(val),
                            Err(_) => TokenType::Error("Invalid float format".to_string()),
                        }
                    } else {
                        match num_str.parse::<i64>() {
                            Ok(val) => TokenType::IntLiteral(val),
                            Err(_) => TokenType::Error("Invalid integer format".to_string()),
                        }
                    }
                }
                _ => TokenType::Error(format!("Unexpected character: '{}'", c)),
            }
        } else {
            TokenType::Eof
        };

        let end_line = self.line;
        let end_column = self.column;

        Token::new(
            kind,
            Span {
                start_line,
                start_col: start_column,
                end_line,
                end_col: end_column,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType::*;

    fn lex_all(input: &str) -> Vec<Token> {
        let mut lexer = Lexer::new(input);
        let mut tokens = Vec::new();
        loop {
            let tok = lexer.next_token();
            if tok.kind == Eof {
                break;
            }
            tokens.push(tok);
        }
        tokens
    }

    fn lex_kinds(input: &str) -> Vec<TokenType> {
        lex_all(input).into_iter().map(|t| t.kind).collect()
    }

    #[test]
    fn test_keywords_and_identifiers() {
        let tokens = lex_kinds("fn let var my_var mut ref if match module");
        assert_eq!(
            tokens,
            vec![
                Fn, Let, Var, Ident("my_var".to_string()), Mut, Ref, If, Match, Module
            ]
        );
    }

    #[test]
    fn test_numbers() {
        let tokens = lex_kinds("42 3.14 0");
        assert_eq!(
            tokens,
            vec![
                IntLiteral(42),
                FloatLiteral(3.14),
                IntLiteral(0),
            ]
        );
    }

    #[test]
    fn test_strings() {
        let tokens = lex_kinds(r#" "hello" "unterminated "#);
        assert_eq!(
            tokens,
            vec![
                StringLiteral("hello".to_string()),
                Error("unterminated string literal".to_string())
            ]
        );
    }

    #[test]
    fn test_operators() {
        let tokens = lex_kinds("= == != ! > >= < <= -> => :: . && ||");
        assert_eq!(
            tokens,
            vec![
                Assign, Eq, NotEq, Bang, Greater, GreaterEq, Less, LessEq, Arrow, FatArrow, DoubleColon, Dot, And, Or
            ]
        );
    }

    #[test]
    fn test_comments_and_whitespace() {
        let input = "
            // this is a comment
            let x = 10; // inline comment
        ";
        let tokens = lex_kinds(input);
        assert_eq!(
            tokens,
            vec![
                Let, Ident("x".to_string()), Assign, IntLiteral(10), Semicolon
            ]
        );
    }

    #[test]
    fn test_spans() {
        let input = "let x = 10;";
        let tokens = lex_all(input);
        
        // 'let'
        assert_eq!(tokens[0].span.start_line, 1);
        assert_eq!(tokens[0].span.start_col, 1);
        assert_eq!(tokens[0].span.end_line, 1);
        assert_eq!(tokens[0].span.end_col, 4); // After 'let'

        // 'x'
        assert_eq!(tokens[1].span.start_col, 5);
        assert_eq!(tokens[1].span.end_col, 6);
    }
}
