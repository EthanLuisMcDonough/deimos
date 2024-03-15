use crate::keyword_map;

keyword_map!(Keyword {
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
});

keyword_map!(PrimitiveType {
    I32 -> "i32",
    F32 -> "f32",
    U8 -> "u8",
});

keyword_map!(Register {
    A0 -> "$a0",
    A1 -> "$a1",
    A2 -> "$a2",
    A3 -> "$a3",
    V0 -> "$v0",
    F0 -> "$f0",
    F12 -> "$f12",
});

#[derive(Debug)]
pub enum Grouper {
    Parenthesis,
    Bracket,
    Brace,
}

#[derive(Debug)]
pub enum Lexeme {
    Keyword(Keyword),
    Register(Register),
    Primitive(PrimitiveType),

    Integer(i32),
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
