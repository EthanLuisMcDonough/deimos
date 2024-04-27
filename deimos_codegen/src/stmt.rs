use deimos_ast::*;
use mips_builder::{DataDef, MipsAddress, MipsBuilder, Register};

use crate::expr::{codegen_expr, RegisterBank};

use super::error::ValidationResult;
use super::expr::rvalue::codegen_assignment;
use super::scope::{GlobalScope, LocalScope, Scope};

pub fn codegen_stmt(b: &mut MipsBuilder, stmt: &Statement, s: &Scope) -> ValidationResult<()> {
    match stmt {
        Statement::Assignment(assignment) => codegen_assignment(b, s, assignment),
        Statement::Call(invoc) => unimplemented!(),
        Statement::Asm(asm) => unimplemented!(),
        Statement::ControlBreak(_) => unimplemented!(),
        Statement::Syscall(_) => unimplemented!(),
        Statement::LogicChain(l) => unimplemented!(),
        Statement::While(w) => unimplemented!(),
        Statement::Print(p) => codegen_print(b, p, s),
    }
}

fn codegen_print(b: &mut MipsBuilder, print: &Print, s: &Scope) -> ValidationResult<()> {
    let mut bank = RegisterBank::default();
    for p_expr in &print.args {
        let expr_val = codegen_expr(b, p_expr, s, &mut bank)?;
        if let (PrimitiveType::F32, 0) = expr_val.type_tuple() {
            //expr_val.register.use_float(b, 0, , fnc)?
        } else {
        }
        //b.add_syscall(id)
    }
    Ok(())
}
