use deimos_ast::*;
use mips_builder::{FloatRegister, GenericRegister, MipsAddress, MipsBuilder, Register};

use crate::expr::print::codegen_print_val;
use crate::expr::temp::{AccessMode, ExprType};
use crate::expr::{self, codegen_expr, RegisterBank};
use crate::names::get_fn_name;
use crate::scope::{LocatedValue, ValLocation};

use super::error::{ValidationError, ValidationResult};
use super::expr::rvalue::codegen_assignment;
use super::scope::Scope;

pub fn codegen_stmt(
    b: &mut MipsBuilder,
    stmt: &Statement,
    s: &Scope,
    p: &Program,
) -> ValidationResult<()> {
    match stmt {
        Statement::Assignment(assignment) => codegen_assignment(b, s, assignment),
        Statement::Call(invoc) => codegen_fnc_call(b, invoc, s),
        Statement::Asm(asm) => codegen_asm(b, asm, s, &p.bank),
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
    codegen_regload_after(b, &syscall.map.out_values, s)
}

fn codegen_asm(
    b: &mut MipsBuilder,
    asm: &AsmBlock,
    s: &Scope,
    strs: &StringBank,
) -> ValidationResult<()> {
    codegen_regload_before(b, &asm.map.in_values, s)?;
    for str_ind in &asm.asm_strings {
        b.instr(strs.strings[str_ind.data].clone());
    }
    codegen_regload_after(b, &asm.map.out_values, s)
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
                    val:
                        DeclType::Param(Located {
                            data:
                                ParamType {
                                    param_type:
                                        Located {
                                            data: PrimitiveType::U8,
                                            ..
                                        },
                                    indirection: 0,
                                },
                            ..
                        }),
                },
            ) => {
                fnc_byte(b, r, addr.into());
            }
            (
                GenericRegister::Regular(r),
                LocatedValue {
                    loc: addr @ ValLocation::Stack(_),
                    val: DeclType::Param(_),
                },
            ) => fnc_word(b, r, addr.into()),
            _ => return Err(ValidationError::InvalidRegTransfer(*identifier, *register)),
        }
    }
    Ok(())
}

fn codegen_regload_before(
    b: &mut MipsBuilder,
    vars: &RegisterMap,
    s: &Scope,
) -> ValidationResult<()> {
    codegen_regload_apply(
        b,
        vars,
        s,
        |b, reg, addr| {
            b.load_word(reg, addr);
        },
        |b, reg, addr| {
            b.load_byte(reg, addr);
        },
        |b, reg, addr| {
            b.load_f32(reg, addr);
        },
    )
}

fn codegen_regload_after(
    b: &mut MipsBuilder,
    vars: &RegisterMap,
    s: &Scope,
) -> ValidationResult<()> {
    codegen_regload_apply(
        b,
        vars,
        s,
        |b, reg, addr| {
            b.save_word(reg, addr);
        },
        |b, reg, addr| {
            b.save_byte(reg, addr);
        },
        |b, reg, addr| {
            b.save_f32(reg, addr);
        },
    )
}

fn codegen_fnc_call(
    b: &mut MipsBuilder,
    invocation: &Invocation,
    caller_scope: &Scope,
) -> ValidationResult<()> {
    let fnc = caller_scope.get_fn(invocation.function)?;
    let invoc_loc = invocation.function.loc;
    if invocation.args.len() != fnc.len() {
        return Err(ValidationError::InvalidArgCount(invoc_loc));
    }

    let arg_stack_size = fnc.len() * 4;
    b.add_const_i32(
        Register::StackPtr,
        Register::StackPtr,
        (fnc.len() as i32) * -4,
    );

    let scope = caller_scope.shift_stack(arg_stack_size as u32);

    let mut bank = RegisterBank::default();
    for (index, (arg_expr, fnc_type)) in invocation.args.iter().zip(fnc.iter()).enumerate() {
        let expr = expr::codegen_expr(b, arg_expr, &scope, &mut bank)?;
        let arg_expr = ExprType::from(fnc_type.field_type.data.clone());
        if arg_expr != expr.computed_type {
            return Err(ValidationError::InvalidArgType(invoc_loc, index, arg_expr));
        }

        let stack_offset = fnc.len() - (index + 1);
        let addr = MipsAddress::RegisterOffset {
            register: Register::StackPtr,
            offset: stack_offset as i32,
        };

        if let ExprType {
            indirection: 0,
            base: PrimitiveType::F32,
        } = arg_expr
        {
            let f_reg = expr.register.get_float()?;
            f_reg.use_reg(b, 0, AccessMode::Read, |b, f| {
                b.save_f32(f, addr);
            });
        } else {
            let reg = expr.register.get_word()?;
            reg.use_reg(b, 0, AccessMode::Read, |b, r| {
                b.save_word(r, addr);
                //b.instr("# PLUH".into());
            });
        }

        bank.clear();
    }

    let fn_name = get_fn_name(invocation.function.data);
    b.jump_and_link(&fn_name);

    Ok(())
}
