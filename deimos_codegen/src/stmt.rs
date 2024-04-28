use deimos_ast::*;
use mips_builder::{FloatRegister, GenericRegister, MipsAddress, MipsBuilder, Register};

use crate::expr::print::codegen_print_val;
use crate::expr::{codegen_expr, RegisterBank};
use crate::scope::{LocatedValue, ValLocation};

use super::error::ValidationResult;
use super::expr::rvalue::codegen_assignment;
use super::scope::{GlobalScope, LocalScope, Scope};

pub fn codegen_stmt(b: &mut MipsBuilder, stmt: &Statement, s: &Scope) -> ValidationResult<()> {
    match stmt {
        Statement::Assignment(assignment) => codegen_assignment(b, s, assignment),
        Statement::Call(invoc) => unimplemented!(),
        Statement::Asm(asm) => unimplemented!(),
        Statement::ControlBreak(_) => unimplemented!(),
        Statement::Syscall(syscall) => codegen_syscall(b, syscall, s),
        Statement::LogicChain(l) => unimplemented!(),
        Statement::While(w) => unimplemented!(),
        Statement::Print(p) => codegen_print(b, p, s),
    }
}

fn codegen_print(b: &mut MipsBuilder, print: &Print, s: &Scope) -> ValidationResult<()> {
    let mut bank = RegisterBank::default();
    for p_expr in &print.args {
        let expr_val = codegen_expr(b, p_expr, s, &mut bank)?;
        codegen_print_val(b, expr_val)?;
        bank.clear();
    }
    Ok(())
}

fn codegen_syscall(b: &mut MipsBuilder, syscall: &Syscall, s: &Scope) -> ValidationResult<()> {
    codegen_regload_before(b, &syscall.map.in_values, s)?;
    b.add_syscall(syscall.syscall_id.data as u8);
    codegen_regload_after(b, &syscall.map.in_values, s)
}

fn cvt_reg(value: Reg) -> GenericRegister {
    match value {
        Reg::A0 => GenericRegister::Regular(Register::A0),
        Reg::A1 => GenericRegister::Regular(Register::A1),
        Reg::A2 => GenericRegister::Regular(Register::A2),
        Reg::A3 => GenericRegister::Regular(Register::A3),
        Reg::F0 => GenericRegister::Float(FloatRegister::F0),
        Reg::F12 => GenericRegister::Float(FloatRegister::F12),
        Reg::V0 => GenericRegister::Regular(Register::V0),
    }
}

fn codegen_regload_apply(
    b: &mut MipsBuilder,
    vars: &RegisterMap,
    s: &Scope,
    fnc_word: impl Fn(&mut MipsBuilder, Register, MipsAddress),
    fnc_byte: impl Fn(&mut MipsBuilder, Register, MipsAddress),
    fnc_f32: impl Fn(&mut MipsBuilder, FloatRegister, MipsAddress),
) -> ValidationResult<()> {
    for (register, identifier) in vars {
        let val = s.get_var(*identifier)?;
        let reg = cvt_reg(*register);
        match (reg, val) {
            (
                GenericRegister::Float(f),
                LocatedValue {
                    loc: addr @ ValLocation::Stack(_),
                    val:
                        DeclType::Param(Located {
                            data:
                                ParamType {
                                    param_type:
                                        Located {
                                            data: PrimitiveType::F32,
                                            ..
                                        },
                                    indirection: 0,
                                },
                            ..
                        }),
                },
            ) => {
                fnc_f32(b, f, addr.into());
            }
            (
                GenericRegister::Regular(r),
                LocatedValue {
                    loc: addr @ ValLocation::Stack(_),
                    val: DeclType::Param(typ),
                },
            ) => fnc_word(b, r, addr.into()),
            _ => {}
        }
    }
    Ok(())
}

fn codegen_regload_before(
    b: &mut MipsBuilder,
    vars: &RegisterMap,
    s: &Scope,
) -> ValidationResult<()> {
    for (register, identifier) in vars {
        let val = s.get_var(*identifier)?;
        if let (DeclType::Param(typ), ValLocation::Stack(_)) = (val.val, val.loc) {
            match (typ.data.param_type.data, typ.data.indirection, register) {
                (PrimitiveType::F32, 0, Reg::F0 | Reg::F12) => {}
                _ => {}
            }
        }
        /*if let (DeclType::Param(Located {
            data:
                ParamType {
                    param_type:
                        Located {
                            data: PrimitiveType::F32,
                            ..
                        },
                    indirection: 0,
                },
            ..
        }),) = val.val
        {
            match val.loc {
                ValLocation::Stack()
            }
        }*/
    }
    unimplemented!()
}

fn codegen_regload_after(
    b: &mut MipsBuilder,
    vars: &RegisterMap,
    s: &Scope,
) -> ValidationResult<()> {
    unimplemented!()
}
