use super::Located;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrimitiveType {
    I32,
    F32,
    U8,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BaseType {
    Primitive(PrimitiveType),
    Custom(usize),
}

#[derive(Debug)]
pub struct ParamType {
    pub param_type: Located<BaseType>,
    pub indirection: usize,
}

#[derive(Debug)]
pub enum DeclType {
    Param(ParamType),
    Array {
        array_type: Located<BaseType>,
        size: usize,
    },
}
