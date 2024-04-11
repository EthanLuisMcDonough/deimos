use crate::{Block, DeclType, Identifier, Located, ParamType, PrimitiveValue};

#[derive(Debug)]
pub enum InitValue {
    Primitive(PrimitiveValue),
    List(Vec<InitValue>),
}

#[derive(Debug)]
pub struct TypedIdent {
    pub name: Identifier,
    pub field_type: Located<ParamType>,
}

#[derive(Debug)]
pub struct VarDecl {
    pub variable: DeclType,
    pub name: Identifier,
    pub init: Option<InitValue>,
}

#[derive(Debug, Default)]
pub struct FunctionBlock {
    pub vars: Vec<VarDecl>,
    pub block: Block,
}

pub type FunctionArgs = Vec<TypedIdent>;

#[derive(Debug)]
pub struct Function {
    pub name: Identifier,
    pub args: FunctionArgs,
    pub block: FunctionBlock,
}

#[derive(Debug)]
pub struct MemVar {
    pub var: TypedIdent,
    pub addr: Located<usize>,
}

#[derive(Debug)]
pub enum Definition {
    Function(Function),
    Static(VarDecl),
    MemVar(MemVar),
}

impl Definition {
    pub fn name(&self) -> usize {
        match self {
            Self::Function(f) => f.name.data,
            Self::Static(s) => s.name.data,
            Self::MemVar(m) => m.var.name.data,
        }
    }
}
