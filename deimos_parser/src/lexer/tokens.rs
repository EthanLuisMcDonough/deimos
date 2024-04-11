use crate::keyword_map;
use deimos_ast::{PrimitiveType, Register};

pub fn test_primitive(s: &str) -> Option<PrimitiveType> {
    Some(match s {
        "i32" => PrimitiveType::I32,
        "u32" => PrimitiveType::U32,
        "f32" => PrimitiveType::F32,
        "u8" => PrimitiveType::U8,
        _ => return None,
    })
}

pub fn test_register(s: &str) -> Option<Register> {
    Some(match s {
        "a0" => Register::A0,
        "a1" => Register::A1,
        "a2" => Register::A2,
        "a3" => Register::A3,
        "v0" => Register::V0,
        "f0" => Register::F0,
        "f12" => Register::F12,
        _ => return None,
    })
}

keyword_map!(Keyword {
    Program -> "program",
    Fn -> "sub",
    Cast -> "as",
    Call -> "call",
    Let -> "let",
    If -> "if",
    Elif -> "elif",
    Else -> "else",
    And -> "and",
    Or -> "or",
    Record -> "record",
    Print -> "print",
    Break -> "break",
    Continue -> "continue",
    Return -> "return",
    Syscall -> "syscall",
    In -> "in",
    Out -> "out",
    Static -> "static",
    Mem -> "mem",
    While -> "while",
    Asm -> "asm",
});

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Grouper {
    Parenthesis,
    Bracket,
    Brace,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Lexeme {
    Keyword(Keyword),
    Register(Register),
    Primitive(PrimitiveType),

    Integer(i32),
    Unsigned(u32),
    Float(f32),
    String(usize),
    Identifier(usize),

    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Equals,

    Reference,
    Deref,

    Colon,
    Semicolon,
    Comma,
    Peroid,

    GreaterThan,
    LessThan,
    GreaterThanEq,
    LessThanEq,
    LogicEq,
    LogicNot,
    LogicNotEq,

    GroupBegin(Grouper),
    GroupEnd(Grouper),
}
