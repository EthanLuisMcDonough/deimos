use std::collections::HashMap;

mod decl;
mod expr;
mod loc;
mod stmt;
mod sys;
mod types;

pub use decl::*;
pub use expr::*;
pub use loc::*;
pub use stmt::*;
pub use sys::*;
pub use types::*;

#[derive(Debug, Default)]
pub struct StringBank {
    pub identifiers: Vec<String>,
    pub strings: Vec<String>,
}

pub type Definitions = HashMap<usize, Definition>;

#[derive(Debug)]
pub struct Program {
    pub bank: StringBank,
    pub fns: Vec<Function>,
    pub static_vars: Vec<VarDecl>,
    pub mem_vars: Vec<MemVar>,
    pub definitions: Definitions,
    pub body: FunctionBlock,
}
