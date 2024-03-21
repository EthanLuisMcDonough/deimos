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

#[derive(Debug)]
pub struct FunctionBlock {
    pub vars: Vec<VarDecl>,
    pub block: Block,
}

pub type FunctionArgs = Vec<TypedIdent>;

#[derive(Debug)]
pub struct Function {
    pub args: FunctionArgs,
    pub block: FunctionBlock,
}

#[derive(Debug)]
pub enum Definition {
    Function(Function),
    Static {
        variable: DeclType,
        init: InitValue,
    },
    MemVar {
        variable: DeclType,
        addr: Located<usize>,
    },
}
