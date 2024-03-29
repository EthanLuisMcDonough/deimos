use super::operators::Operator;
use crate::{
    lexer::Lexeme,
    parser::{Grouper, ParseError, ParseResult},
};
use deimos_ast::{Expression, Located, Location, ParamType};
use std::collections::VecDeque;

/// Structs used internally by the shunting-yard struct
enum OpStack {
    Open(Grouper),
    Op(Operator),
}

enum ExprStack {
    Expression(Expression),
    Type(ParamType),
}

enum LastItem {
    Open,
    Close,
    Expr,
    BinOp,
    UnOp,
}

/// Struct implementing the shunting-yard algorithm for parsing expressions
#[derive(Default)]
pub struct ShuntingStack {
    operands: VecDeque<ExprStack>,
    operators: VecDeque<Located<OpStack>>,
    last_item: Option<LastItem>,
}

impl ShuntingStack {
    /// Pushes an expression node onto the stack
    pub fn push_expr(&mut self, e: Expression) {
        self.operands.push_back(ExprStack::Expression(e));
        self.last_item = Some(LastItem::Expr);
    }

    pub fn push_cast_type(&mut self, t: ParamType) {
        self.operands.push_back(ExprStack::Type(t));
        self.last_item = Some(LastItem::Expr);
    }

    fn apply_op(&mut self, o: Operator, loc: Location) -> bool {
        match o {
            Operator::Binary(b) => {
                let right = self.operands.pop_back();
                let left = self.operands.pop_back();
                if let (Some(ExprStack::Expression(r)), Some(ExprStack::Expression(l))) =
                    (left, right)
                {
                    self.operands
                        .push_back(ExprStack::Expression(Expression::Binary {
                            left: Box::new(l),
                            right: Box::new(r),
                            op: Located::new(b, loc),
                        }));
                    true
                } else {
                    false
                }
            }
            Operator::Unary(u) => {
                if let Some(ExprStack::Expression(e)) = self.operands.pop_back() {
                    self.operands
                        .push_back(ExprStack::Expression(Expression::Unary {
                            operand: Box::new(e),
                            op: Located::new(u, loc),
                        }));
                    true
                } else {
                    false
                }
            }
            Operator::Cast => {
                let right = self.operands.pop_back();
                let left = self.operands.pop_back();
                if let (Some(ExprStack::Expression(val)), Some(ExprStack::Type(p))) = (left, right)
                {
                    self.operands
                        .push_back(ExprStack::Expression(Expression::Cast {
                            value: Box::new(val),
                            cast_type: p,
                        }));
                    true
                } else {
                    false
                }
            }
        }
    }

    fn back_higher_prec(&self, o: &Operator) -> bool {
        self.operators
            .back()
            .filter(|b| match b.data {
                OpStack::Op(op) => op.precedence() < o.precedence(),
                _ => false,
            })
            .is_some()
    }

    pub fn push_op(&mut self, o: impl Into<Operator>, loc: Location) -> ParseResult<()> {
        let op = o.into();
        while self.back_higher_prec(&op) {
            if let Some(Located {
                data: OpStack::Op(o),
                ..
            }) = self.operators.pop_back()
            {
                if !self.apply_op(o, loc) {
                    return Err(ParseError::InvalidOperation(loc));
                }
            }
        }
        self.last_item = Some(match op {
            Operator::Binary(_) | Operator::Cast => LastItem::BinOp,
            Operator::Unary(_) => LastItem::UnOp,
        });
        self.operators.push_back(Located::new(OpStack::Op(op), loc));
        Ok(())
    }

    pub fn push_open(&mut self, g: Grouper, loc: Location) {
        self.operators
            .push_back(Located::new(OpStack::Open(g), loc));
        self.last_item = Some(LastItem::Open);
    }

    pub fn push_close(&mut self, g: Grouper, loc: Location) -> ParseResult<()> {
        loop {
            match self.operators.pop_back() {
                Some(Located {
                    data: OpStack::Open(b),
                    ..
                }) if b == g => break,
                Some(Located {
                    data: OpStack::Op(o),
                    ..
                }) => {
                    if !self.apply_op(o, loc) {
                        return Err(ParseError::InvalidOperation(loc));
                    }
                }
                _ => {
                    return Err(ParseError::UnexpectedToken(Located::new(
                        Lexeme::GroupEnd(g),
                        loc,
                    )))
                }
            }
        }
        self.last_item = Some(LastItem::Close);
        Ok(())
    }

    pub fn yield_unary(&self) -> bool {
        match self.last_item {
            None | Some(LastItem::Open | LastItem::BinOp | LastItem::Expr) => true,
            Some(LastItem::UnOp | LastItem::Close) => false,
        }
    }

    pub fn get_val(&mut self) -> ParseResult<Expression> {
        while let Some(op_item) = self.operators.pop_back() {
            let loc = op_item.loc;
            match op_item.data {
                OpStack::Open(_) => return Err(ParseError::UnexpectedEOF),
                OpStack::Op(op) if !self.apply_op(op, loc) => {}
                OpStack::Op(_) => return Err(ParseError::InvalidOperation(loc)),
            }
        }
        if self.operands.len() != 1 {
            return Err(ParseError::UnexpectedEOF);
        }
        if let Some(ExprStack::Expression(expr)) = self.operands.pop_back() {
            Ok(expr)
        } else {
            Err(ParseError::UnexpectedEOF)
        }
    }
}
