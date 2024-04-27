use super::{ExprTemp, RegisterBank};
use crate::error::*;
use crate::scope::Scope;

use deimos_ast::*;
use mips_builder::MipsBuilder;

fn codegen_print_val(
    b: &mut MipsBuilder,
    expr_val: ExprTemp,
    reg_bank: &mut RegisterBank,
    s: &Scope,
) -> ValidationResult<()> {
    /*match expr_val.type_tuple() {
        //()
    }*/
    Ok(())
}
