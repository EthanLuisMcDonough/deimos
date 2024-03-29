use super::{Grouper, Keyword, Lexeme, ParseResult};
use deimos_ast::*;

use crate::{
    next_guard,
    parser::{expr::operators::Operator, ParseError},
};

mod operators;
mod shunt;

pub fn parse_param_type<T: Iterator<Item = Located<Lexeme>>>(
    tokens: &mut T,
) -> ParseResult<Located<ParamType>> {
    let mut indirection = 0usize;
    let mut loc = None;
    loop {
        next_guard!({ tokens.next() } (l) {
            Lexeme::Reference => {
                indirection += 1;
                loc.get_or_insert(l);
            },
            Lexeme::Primitive(p) => {
                let loc = loc.unwrap_or(l);
                return Ok(Located::new(
                    ParamType {
                        param_type: Located::new(p, l),
                        indirection,
                    },
                    loc,
                ));
            },
        });
    }
}

pub fn parse_expression(
    mut tokens: impl Iterator<Item = Located<Lexeme>>,
) -> ParseResult<Expression> {
    let mut stack = shunt::ShuntingStack::default();

    while let Some(token) = tokens.next() {
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
            Lexeme::Identifier(i) => {
                stack.push_expr(Expression::Identifier(Located::new(i, token.loc)));
            }
            Lexeme::Plus => stack.push_op(BinaryOp::Add, token.loc)?,
            Lexeme::Minus if stack.yield_unary() => stack.push_op(UnaryOp::Negation, token.loc)?,
            Lexeme::Minus => stack.push_op(BinaryOp::Sub, token.loc)?,
            Lexeme::Multiply => stack.push_op(BinaryOp::Mult, token.loc)?,
            Lexeme::Divide => stack.push_op(BinaryOp::Div, token.loc)?,
            Lexeme::Modulo => stack.push_op(BinaryOp::Mod, token.loc)?,
            Lexeme::LessThan => stack.push_op(BinaryOp::LessThan, token.loc)?,
            Lexeme::GreaterThan => stack.push_op(BinaryOp::GreaterThan, token.loc)?,
            Lexeme::LessThanEq => stack.push_op(BinaryOp::LessThanEq, token.loc)?,
            Lexeme::GreaterThanEq => stack.push_op(BinaryOp::GreaterThanEq, token.loc)?,
            Lexeme::LogicEq => stack.push_op(BinaryOp::Equal, token.loc)?,
            Lexeme::LogicNotEq => stack.push_op(BinaryOp::NotEq, token.loc)?,
            Lexeme::Keyword(Keyword::And) => stack.push_op(BinaryOp::And, token.loc)?,
            Lexeme::Keyword(Keyword::Or) => stack.push_op(BinaryOp::Or, token.loc)?,
            Lexeme::Keyword(Keyword::Cast) => {
                let cast_type = parse_param_type(&mut tokens)?;
                stack.push_op(Operator::Cast, token.loc)?;
                stack.push_cast_type(cast_type.data);
            }
            Lexeme::GroupBegin(Grouper::Brace) => {
                stack.push_op(BinaryOp::IndexAccess, token.loc)?;
                stack.push_open(Grouper::Brace, token.loc);
            }
            Lexeme::GroupBegin(Grouper::Parenthesis) => {
                stack.push_open(Grouper::Parenthesis, token.loc)
            }
            Lexeme::GroupEnd(g @ Grouper::Parenthesis | g @ Grouper::Brace) => {
                stack.push_close(g, token.loc)?;
            }
            _ => {
                return Err(ParseError::UnexpectedToken(token));
            }
        }
    }
    stack.get_val()
}
