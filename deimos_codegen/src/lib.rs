use deimos_ast::*;
use mips_builder::{MipsBuilder, Register};

pub fn codegen(_p: &Program) -> String {
    let mut codegen = MipsBuilder::new();
    codegen.new_block("main");
    codegen.const_i32(10, Register::T0);
    codegen.const_i32(11, Register::T1);
    codegen.add_i32(Register::A0, Register::T0, Register::T1);
    codegen.add_syscall(1);
    codegen.add_syscall(10);
    codegen.codegen()
}
