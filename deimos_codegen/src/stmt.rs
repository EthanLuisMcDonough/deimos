use deimos_ast::*;
use mips_builder::{FloatRegister, GenericRegister, MipsAddress, MipsBuilder, Register};

use crate::expr::print::codegen_print_val;
use crate::expr::temp::{AccessMode, ExprType};
use crate::expr::{self, codegen_expr, RegisterBank};
use crate::names::get_fn_name;
use crate::names::*;
use crate::scope::{LocatedValue, ValLocation};

use super::error::{ValidationError, ValidationResult};
use super::expr::rvalue::codegen_assignment;
use super::scope::{ConstructCounter, Scope};

/// Codegen for entire block
pub fn codegen_block(
    b: &mut MipsBuilder,
    block: &Block,
    scope: &Scope,
    p: &Program,
    c: &mut ConstructCounter,
) -> ValidationResult<()> {
    for stmt in block {
        codegen_stmt(b, &stmt.data, scope, p, c)?;
    }
    Ok(())
}

/// Codegen for generic statement
pub fn codegen_stmt(
    b: &mut MipsBuilder,
    stmt: &Statement,
    s: &Scope,
    p: &Program,
    c: &mut ConstructCounter,
) -> ValidationResult<()> {
    match stmt {
        Statement::Assignment(assignment) => codegen_assignment(b, s, assignment),
        Statement::Call(invoc) => codegen_fnc_call(b, invoc, s),
        Statement::Asm(asm) => codegen_asm(b, asm, s, &p.bank),
        Statement::ControlBreak(control) => codegen_control_flow(b, control, c),
        Statement::Syscall(syscall) => codegen_syscall(b, syscall, s),
        Statement::LogicChain(l) => codegen_logic_chain(b, l, s, p, c),
        Statement::While(w) => codegen_while(b, w, s, p, c),
        Statement::Print(p) => codegen_print(b, p, s),
    }
}

fn codegen_control_flow(
    b: &mut MipsBuilder,
    f: &Located<ControlBreak>,
    c: &mut ConstructCounter,
) -> ValidationResult<()> {
    let label = match &f.data {
        ControlBreak::Return => c.get_current_fn().map(get_fn_end),
        ControlBreak::Continue => c.get_current_loop().map(get_while_lbl),
        ControlBreak::Break => c.get_current_loop().map(get_while_end),
    };
    if let Some(l) = label {
        b.branch(&l);
        Ok(())
    } else {
        Err(ValidationError::InvalidControlFlow(f.loc, f.data))
    }
}

/// Codegen for print statement
fn codegen_print(b: &mut MipsBuilder, print: &Print, s: &Scope) -> ValidationResult<()> {
    let mut bank = RegisterBank::default();
    for p_expr in &print.args {
        let expr_val = codegen_expr(b, p_expr, s, &mut bank)?;
        codegen_print_val(b, expr_val)?;
        bank.clear();
    }
    Ok(())
}

/// Codegen for syscall statement
fn codegen_syscall(b: &mut MipsBuilder, syscall: &Syscall, s: &Scope) -> ValidationResult<()> {
    codegen_regload_before(b, &syscall.map.in_values, s)?;
    b.add_syscall(syscall.syscall_id.data as u8);
    codegen_regload_after(b, &syscall.map.out_values, s)
}

/// Codegen for ASM block
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

/// Map AST node registers to real codegen registers
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

/// Scaffold for regload operations.
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

/// Move user defined variables into specific registers.
/// This is called before asm and syscall
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

/// Move values in specific registers to user defined variables.
/// This is called after asm and syscall
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

/// Codegen for function calls
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

        let stack_offset = (fnc.len() - (index + 1)) * 4;
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
            });
        }

        bank.clear();
    }

    let fn_name = get_fn_name(invocation.function.data);
    b.jump_and_link(&fn_name);

    Ok(())
}

/// Generates the branch instructions for a given condition.
/// Branches to fail_branch if expression is equal to zero.
fn codegen_condition(
    b: &mut MipsBuilder,
    e: &Expression,
    s: &Scope,
    bank: &mut RegisterBank,
    fail_branch: &str,
) -> ValidationResult<()> {
    let expr = codegen_expr(b, e, s, bank)?;

    if let ExprType {
        indirection: 0,
        base: PrimitiveType::F32,
    } = expr.computed_type
    {
        return Err(ValidationError::FloatInCondition(e.get_loc()));
    }

    let reg = expr.register.get_word()?;
    reg.use_reg(b, 0, AccessMode::Read, |b, r| {
        b.branch_eq_zero(r, &fail_branch);
    });

    Ok(())
}

/// Codegen for while loops
fn codegen_while(
    b: &mut MipsBuilder,
    loop_block: &ConditionBody,
    s: &Scope,
    p: &Program,
    c: &mut ConstructCounter,
) -> ValidationResult<()> {
    let loop_id = c.start_loop();
    let loop_start_lbl = get_while_lbl(loop_id);
    let loop_end_lbl = get_while_end(loop_id);

    let mut bank = RegisterBank::default();

    b.new_block(loop_start_lbl.clone());

    codegen_condition(b, &loop_block.condition, s, &mut bank, &loop_end_lbl)?;

    codegen_block(b, &loop_block.body, s, p, c)?;
    b.branch(&loop_start_lbl);

    b.new_block(loop_end_lbl);

    let loop_id_check = c.end_loop();
    assert_eq!(loop_id, loop_id_check);

    Ok(())
}

/// Generates the code for an if/else logic chain
fn codegen_logic_chain(
    b: &mut MipsBuilder,
    l: &LogicChain,
    s: &Scope,
    p: &Program,
    c: &mut ConstructCounter,
) -> ValidationResult<()> {
    let if_id = c.new_if();
    let if_lbl = get_if_lbl(if_id);

    // Iterator for checking next label in logic chain
    let elif_lbls = (0..l.elifs.len())
        .map(|i| get_elif_lbl(if_id, i))
        .collect::<Vec<_>>();
    let end_lbl = get_if_end(if_id);
    let else_lbl = l.else_block.as_ref().map(|_| get_if_else(if_id));
    let mut lbl_iter = elif_lbls.iter().chain(else_lbl.iter());
    let mut next_lbl = || lbl_iter.next().unwrap_or(&end_lbl);

    let mut bank = RegisterBank::default();

    b.new_block(if_lbl);
    codegen_condition(b, &l.if_block.condition, s, &mut bank, next_lbl())?;

    codegen_block(b, &l.if_block.body, s, p, c)?;
    b.branch(&end_lbl); // Go to end if body was run

    // Elifs
    for (index, elif) in l.elifs.iter().enumerate() {
        b.new_block(get_elif_lbl(if_id, index));
        codegen_condition(b, &elif.condition, s, &mut bank, next_lbl())?;

        codegen_block(b, &elif.body, s, p, c)?;
        b.branch(&end_lbl); // Go to end if body was run
    }

    // Generate else block
    if let Some(else_block) = &l.else_block {
        b.new_block(get_if_else(if_id));
        codegen_block(b, else_block, s, p, c)?;
    }

    // End of if statement
    b.new_block(end_lbl);

    Ok(())
}
