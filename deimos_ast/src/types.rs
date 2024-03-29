use super::Located;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrimitiveType {
    I32,
    F32,
    U8,
}

#[derive(Debug)]
pub struct ParamType {
    pub param_type: Located<PrimitiveType>,
    pub indirection: usize,
}

#[derive(Debug)]
pub enum DeclType {
    Param(ParamType),
    Array {
        array_type: Located<PrimitiveType>,
        size: usize,
    },
}
