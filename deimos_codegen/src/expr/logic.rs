use deimos_ast::*;
use mips_builder::{MipsBuilder, Register};

use super::temp::{AccessMode, ExprTemp, ExprType, RegisterBank};
use crate::error::{ValidationError, ValidationResult};
use crate::scope::{Scope, ValLocation};

pub fn codegen_logic_not(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    expr: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic ! not implemented")
}

pub fn codgen_logic_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic == not implemented")
}

pub fn codgen_logic_not_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic != not implemented")
}

pub fn codgen_logic_greater_than(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic > not implemented")
}

pub fn codgen_logic_greater_than_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic >= not implemented")
}

pub fn codgen_logic_less_than(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic < not implemented")
}

pub fn codgen_logic_less_than_eq(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic <= not implemented")
}

pub fn codgen_logic_and(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic and not implemented")
}

pub fn codgen_logic_or(
    b: &mut MipsBuilder,
    reg_bank: &mut RegisterBank,
    left: ExprTemp,
    right: ExprTemp,
    loc: Location,
) -> ValidationResult<ExprTemp> {
    unimplemented!("Logic or not implemented")
}
