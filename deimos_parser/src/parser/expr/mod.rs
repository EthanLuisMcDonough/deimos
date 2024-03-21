use std::collections::VecDeque;

use super::{Grouper, Lexeme, ParseResult};
use deimos_ast::*;

mod operators;
mod shunt;

#[derive(Clone, Copy)]
enum Operator {
    Binary(BinaryOp),
    Unary(UnaryOp),
    Cast,
}

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    Access,
    Cast,
    Unary,
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

enum OpStack {
    Grouper(Grouper),
    Op(Operator),
}

#[derive(Default)]
struct ShuntingStack {
    operands: VecDeque<Expression>,
    operators: VecDeque<OpStack>,
}

impl ShuntingStack {
    fn push_expr(&mut self, e: Expression) {
        self.operands.push_back(e);
    }

    fn push_op(&mut self, o: OpStack) {

    }
}

fn parse_expression(tokens: impl Iterator<Item = Located<Lexeme>>) -> ParseResult<Expression> {
    let mut stack = ShuntingStack::default();

    for token in tokens {
        match token.data {
            Lexeme::Integer(i) => {
                stack.push_expr(Expression::Primitive(PrimitiveValue::Int(Located::new(
                    i, token.loc,
                ))));
            }
            Lexeme::Float(f) => {
                stack.push_expr(Expression::Primitive(PrimitiveValue::Float(Located::new(
                    f, token.loc,
                ))));
            }
            Lexeme::String(s) => {
                stack.push_expr(Expression::Primitive(PrimitiveValue::String(Located::new(
                    s, token.loc,
                ))));
            }
            _ => unimplemented!(),
        }
    }

    unimplemented!()
}
