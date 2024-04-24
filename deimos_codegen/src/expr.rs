use deimos_ast::*;
use mips_builder::{DataDef, MipsBuilder, Register};
use crate::scope::Scope;
use crate::error::*;

struct RegisterBank {
    
}

pub fn codegen_expr(b: &mut MipsBuilder, expr: &Expression, s: &Scope) -> ValidationResult<()> {

    Ok(())
}
