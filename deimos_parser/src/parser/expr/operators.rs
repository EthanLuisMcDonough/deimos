use deimos_ast::{BinaryOp, UnaryOp};

use crate::lexer::Lexeme;

#[derive(Clone, Copy)]
pub enum Operator {
    Binary(BinaryOp),
    Unary(UnaryOp),
    Cast,
}

impl From<BinaryOp> for Operator {
    fn from(b: BinaryOp) -> Self {
        Self::Binary(b)
    }
}

impl From<UnaryOp> for Operator {
    fn from(u: UnaryOp) -> Self {
        Self::Unary(u)
    }
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
pub enum Precedence {
    Access,
    Unary,
    Cast,
    MulDiv,
    AddSub,
    Cmp,
    Eq,
    And,
    Or,
}

impl Operator {
    pub fn precedence(&self) -> Precedence {
        match self {
            Operator::Binary(BinaryOp::IndexAccess) => Precedence::Access,
            Operator::Cast => Precedence::Cast,
            Operator::Unary(_) => Precedence::Unary,
            Operator::Binary(BinaryOp::Mult | BinaryOp::Div | BinaryOp::Mod) => Precedence::MulDiv,
            Operator::Binary(BinaryOp::Add | BinaryOp::Sub) => Precedence::AddSub,
            Operator::Binary(
                BinaryOp::GreaterThan
                | BinaryOp::GreaterThanEq
                | BinaryOp::LessThan
                | BinaryOp::LessThanEq,
            ) => Precedence::Cmp,
            Operator::Binary(BinaryOp::Equal | BinaryOp::NotEq) => Precedence::Eq,
            Operator::Binary(BinaryOp::And) => Precedence::And,
            Operator::Binary(BinaryOp::Or) => Precedence::Or,
        }
    }
}
