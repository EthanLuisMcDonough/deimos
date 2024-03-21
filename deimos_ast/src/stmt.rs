use crate::{Expression, Identifier, Located, Syscall};

#[derive(Debug)]
pub enum RValue {
    Identifier(Identifier),
    Index {
        array: Identifier,
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
pub enum Statement {
    LogicChain {
        if_block: ConditionBody,
        elifs: Vec<ConditionBody>,
        else_block: Option<Block>,
    },
    While(ConditionBody),
    Call {
        function: Identifier,
        args: Vec<Expression>,
    },
    Assignment {
        rvalue: RValue,
        lvalue: Expression,
    },
    Syscall(Syscall),
    ControlBreak(ControlBreak),
}

pub type Block = Vec<Located<Statement>>;
