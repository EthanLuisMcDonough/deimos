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
pub struct Record {
    pub fields: Vec<TypedIdent>,
}

#[derive(Debug)]
pub struct VarDecl {
    pub variable: DeclType,
    pub init: Option<InitValue>,
}

#[derive(Debug)]
pub struct FunctionBlock {
    pub vars: Vec<VarDecl>,
    pub block: Block,
}

#[derive(Debug)]
pub struct Function {
    pub args: Vec<TypedIdent>,
    pub block: FunctionBlock,
}

#[derive(Debug)]
pub enum Definition {
    Record(Record),
    Function(Function),
    Static(InitValue),
    MemVar(PrimitiveValue),
}
