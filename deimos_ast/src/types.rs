use super::Located;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PrimitiveType {
    I32,
    U32,
    F32,
    U8,
}

#[derive(Debug, Clone)]
pub struct ParamType {
    pub param_type: Located<PrimitiveType>,
    pub indirection: usize,
}

impl ParamType {
    pub fn type_eq(&self, other: &ParamType) -> bool {
        other.param_type.data == self.param_type.data && self.indirection == other.indirection
    }
}

#[derive(Debug, Clone)]
pub enum DeclType {
    Param(Located<ParamType>),
    Array {
        array_type: Located<ParamType>,
        size: Located<u32>,
    },
}

impl DeclType {
    pub fn get_param_type(&self) -> Located<ParamType> {
        match self {
            Self::Param(p) => p.clone(),
            Self::Array {
                array_type,
                size: _,
            } => {
                let mut param_type = array_type.clone();
                param_type.data.indirection += 1;
                param_type
            }
        }
    }
}

impl From<Located<ParamType>> for DeclType {
    fn from(value: Located<ParamType>) -> Self {
        DeclType::Param(value)
    }
}
