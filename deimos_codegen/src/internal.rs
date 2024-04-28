use super::names::*;
use mips_builder::{DataDef, MipsBuilder, Register};

pub fn setup_main(b: &mut MipsBuilder) {
    // Save CLI arguments
    let mut argc = DataDef::new(ARGC_GLOBAL);
    argc.add_dir(0);
    b.add_def(argc);

    let mut argv = DataDef::new(ARGV_GLOBAL);
    argv.add_dir(0);
    b.add_def(argv);

    b.new_block("main");
    b.save_word(Register::A0, ARGC_GLOBAL);
    b.save_word(Register::A1, ARGV_GLOBAL);
}

pub fn teardown_main(b: &mut MipsBuilder) {
    b.add_syscall(10);

    // Read floating point condition bit to register
    b.new_block(GET_FLOAT_BOOL);
    b.branch_float_false(GET_FLOAT_BOOL_FALSE);
    b.const_word(1, Register::V0);
    b.jump_register(Register::ReturnAddr);
    b.new_block(GET_FLOAT_BOOL_FALSE);
    b.const_word(0, Register::V0);
    b.jump_register(Register::ReturnAddr);

    // Inverse of above function
    b.new_block(GET_FLOAT_BOOL_INV);
    b.branch_float_false(GET_FLOAT_BOOL_INV_FALSE);
    b.const_word(0, Register::V0);
    b.jump_register(Register::ReturnAddr);
    b.new_block(GET_FLOAT_BOOL_INV_FALSE);
    b.const_word(1, Register::V0);
    b.jump_register(Register::ReturnAddr);
}
