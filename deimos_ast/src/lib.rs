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

#[derive(Debug)]
pub struct ProgramArgs {
    pub argc: Identifier,
    pub argv: Identifier,
}

#[derive(Debug)]
pub struct ProgramBody {
    pub args: Option<ProgramArgs>,
    pub block: FunctionBlock,
}

pub type Definitions = HashMap<usize, Definition>;

#[derive(Debug)]
pub struct Program {
    pub bank: StringBank,
    pub definitions: Definitions,
    pub body: ProgramBody,
}
