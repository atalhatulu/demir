// Compiler tortüre / smoke testleri.
// Proje bin-only olduğundan modüllere `#[path]` ile erişilir.
// Amacı: temel lexer/parser hattının canlı olduğunu ve
// örnek `.dmr` dosyalarının uçtan uca çalıştığını doğrulamak.

#[path = "../src/token.rs"]
mod token;

#[path = "../src/lexer.rs"]
mod lexer;

use lexer::Lexer;
use token::{Token, TokenType};

/// `next_token()`'ı Eof'a kadar çağırıp tüm tokenları toplar.
fn lex_all(src: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(src);
    let mut tokens = Vec::new();
    loop {
        let t = lexer.next_token();
        let is_eof = matches!(t.kind, TokenType::Eof) || matches!(t.kind, TokenType::Error(_));
        tokens.push(t);
        if is_eof {
            break;
        }
    }
    tokens
}

#[test]
fn lexer_basic_tokens() {
    let src = "let x = 42;";
    let tokens = lex_all(src);
    // let, x, =, 42, ;
    assert!(tokens.len() >= 5, "beklenen token sayısı, got {}", tokens.len());
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenType::Let)));
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenType::IntLiteral(42))));
}

#[test]
fn lexer_handles_arithmetic() {
    let tokens = lex_all("1 + 2 * 3;");
    assert!(tokens.len() >= 6);
}

#[test]
fn lexer_reports_span() {
    let tokens = lex_all("let x = 10;");
    let first = &tokens[0];
    assert_eq!(first.span.start_line, 1);
    assert_eq!(first.span.start_col, 1);
}

#[test]
fn lexer_char_literal() {
    let tokens = lex_all("'a'");
    assert!(tokens.iter().any(|t| matches!(t.kind, TokenType::CharLiteral('a'))));

    // Escape'li char literal
    let esc = lex_all("'\\n'");
    assert!(esc.iter().any(|t| matches!(t.kind, TokenType::CharLiteral('\n'))));

    // Terminatsız char literal hata üretmeli
    let bad = lex_all("'a");
    assert!(bad.iter().any(|t| matches!(t.kind, TokenType::Error(_))));
}

