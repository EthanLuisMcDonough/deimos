use super::Located;

pub type Identifier = Located<usize>;

#[derive(Debug)]
pub enum UnaryOp {
    Negation,
    LogicNot,
    Deref,
    Reference,
}

#[derive(Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    GreaterThan,
    LessThan,
    GreaterThanEq,
    LessThanEq,
    NotEq,
    Equal,
    And,
    Or,
}

#[derive(Debug)]
pub enum PrimitiveValue {
    Float(Located<f32>),
    Int(Located<i32>),
    String(Located<usize>),
}

#[derive(Debug)]
pub enum Expression {
    Binary {
        left: Box<Expression>,
        right: Box<Expression>,
        op: Located<BinaryOp>,
    },
    Unary {
        operand: Box<Expression>,
        op: Located<UnaryOp>,
    },
    Identifier(Identifier),
    Primitive(PrimitiveValue),
    IndexAccess {
        array: Box<Expression>,
        index: Box<Expression>,
    },
}
