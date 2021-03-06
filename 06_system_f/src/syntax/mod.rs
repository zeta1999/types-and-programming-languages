//! Lexical analysis and recursive descent parser for System F
pub mod lexer;
pub mod parser;
use util::span::Span;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub enum TokenKind {
    Uppercase(String),
    Lowercase(String),
    Nat(u32),
    TyNat,
    TyBool,
    TyArrow,
    TyUnit,
    Unit,
    True,
    False,
    Lambda,
    Forall,
    Exists,
    As,
    Pack,
    Unpack,
    Succ,
    Pred,
    If,
    Then,
    Else,
    Let,
    In,
    IsZero,
    Semicolon,
    Colon,
    Comma,
    Proj,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LSquare,
    RSquare,
    Equals,
    Bar,
    Wildcard,
    Gt,
    Case,
    Of,
    Fix,
    Fold,
    Unfold,
    Rec,
    Invalid(char),
    Dummy,
    Eof,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub const fn dummy() -> Token {
        Token {
            kind: TokenKind::Dummy,
            span: Span::zero(),
        }
    }

    pub const fn new(kind: TokenKind, span: Span) -> Token {
        Token { kind, span }
    }
}
