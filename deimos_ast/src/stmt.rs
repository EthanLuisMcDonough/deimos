use crate::{AsmBlock, Expression, Identifier, Located, Syscall};

#[derive(Debug)]
pub enum RValue {
    Identifier(Identifier),
    Index {
        array: Expression,
        value: Expression,
    },
    Deref(Expression),
}

#[derive(Debug)]
pub struct ConditionBody {
    pub condition: Expression,
    pub body: Block,
}

#[derive(Debug)]
pub struct LogicChain {
    pub if_block: ConditionBody,
    pub elifs: Vec<ConditionBody>,
    pub else_block: Option<Block>,
}

#[derive(Debug)]
pub enum ControlBreak {
    Break,
    Continue,
    Return,
}

#[derive(Debug)]
pub struct Assignment {
    pub rvalue: RValue,
    pub lvalue: Expression,
}

#[derive(Debug)]
pub struct Print {
    pub args: Vec<Expression>,
}

#[derive(Debug)]
pub struct Invocation {
    pub function: Identifier,
    pub args: Vec<Expression>,
}

#[derive(Debug)]
pub enum Statement {
    LogicChain(LogicChain),
    While(ConditionBody),
    Call(Invocation),
    Assignment(Assignment),
    Syscall(Syscall),
    ControlBreak(ControlBreak),
    Print(Print),
    Asm(AsmBlock),
}

pub type Block = Vec<Located<Statement>>;
