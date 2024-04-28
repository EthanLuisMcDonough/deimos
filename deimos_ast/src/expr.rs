use super::{Located, Location, ParamType};

pub type Identifier = Located<usize>;

#[derive(Debug, Clone, Copy)]
pub enum UnaryOp {
    Negation,
    LogicNot,
    Deref,
    Reference,
}

#[derive(Debug, Clone, Copy)]
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
    IndexAccess,
}

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveValue {
    Float(f32),
    Int(i32),
    Unsigned(u32),
    String(usize),
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
    Cast {
        value: Box<Expression>,
        cast_type: ParamType,
    },
    Identifier(Identifier),
    Primitive(Located<PrimitiveValue>),
}

impl Expression {
    /// Extracts location from expression
    pub fn get_loc(&self) -> Location {
        match self {
            Self::Cast { value, .. } | Self::Binary { left: value, .. } => value.get_loc(),
            Self::Unary { op, .. } => op.loc,
            Self::Identifier(ident) => ident.loc,
            Self::Primitive(p) => p.loc,
        }
    }
}
