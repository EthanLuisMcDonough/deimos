use crate::{Block, DeclType, Identifier, Located, ParamType, PrimitiveValue};

pub type InitList = Vec<Located<PrimitiveValue>>;

#[derive(Debug, Clone)]
pub enum InitValue {
    Primitive(PrimitiveValue),
    List(Vec<Located<PrimitiveValue>>),
}

#[derive(Debug, Clone)]
pub struct TypedIdent {
    pub name: Identifier,
    pub field_type: Located<ParamType>,
}

#[derive(Debug)]
pub struct VarDecl {
    pub variable: DeclType,
    pub name: Identifier,
    pub init: Option<Located<InitValue>>,
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
    pub addr: Located<u32>,
}

#[derive(Debug)]
pub enum Definition {
    Function(usize),
    Static(usize),
    MemVar(usize),
}
