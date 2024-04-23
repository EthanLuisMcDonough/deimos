use deimos_ast::*;
use mips_builder::{DataDef, DataDirective, MipsAddress, MipsBuilder, Register};

mod const_expr;
mod error;
mod names;
mod scope;
use names::*;

use error::ValidationResult;
use scope::{GlobalScope, LocalScope, ValLocation};

use crate::error::ValidationError;

fn codegen_sub(b: &mut MipsBuilder, p: &Program, sub: &Function) {
    b.new_block(format!("{}{}", FN_PREFIX, sub.name.data));
}

fn codegen_stmt(b: &mut MipsBuilder, stmt: &Statement) {
    match stmt {
        _ => {}
    }
}

fn codegen_init_fn_stack(b: &mut MipsBuilder, vars: &Vec<VarDecl>, local: &LocalScope) {
    b.const_word(local.get_stack_size(), Register::T0);
    b.sub_i32(Register::StackPtr, Register::StackPtr, Register::T0);
    b.save_word(Register::ReturnAddr, local.get_ra_stack_loc());
    for var in vars {
        let val = local
            .get_local_var(var.name)
            .expect("Incorrect scope fed to codegen_init_fn_stack");
        unimplemented!()
    }
}

fn codegen_main(b: &mut MipsBuilder, global: &GlobalScope, p: &Program) -> ValidationResult<()> {
    let mut local = LocalScope::default();
    for var in &p.body.vars {
        //local.insert(var.name, var.variable)
    }
    unimplemented!()
}

pub fn codegen(p: &Program) -> ValidationResult<String> {
    let mut global = GlobalScope::default();
    let mut fnc_scopes = Vec::new();
    for static_var in &p.static_vars {
        global.insert_static(static_var);
    }
    for mem_var in &p.mem_vars {
        global.insert_mem(mem_var)?;
    }
    for fnc in &p.fns {
        let mut local = LocalScope::default();
        for param in &fnc.args {
            local.insert(param.name, param.field_type.clone())?;
        }
        local.insert_ra();
        for local_var in &fnc.block.vars {
            local.insert(local_var.name, local_var.variable.clone())?;
        }
        fnc_scopes.push((fnc.name.data, local));
        global.insert_fn(fnc);
    }

    let mut codegen = MipsBuilder::new();
    codegen.new_block("main");
    codegen.const_word(10, Register::T0);
    codegen.const_word(11, Register::T1);
    codegen.add_i32(Register::A0, Register::T0, Register::T1);
    codegen.add_syscall(1);
    codegen.add_syscall(10);
    Ok(codegen.codegen())
}

fn setup_main(b: &mut MipsBuilder) {
    // Save CLI arguments
    let mut argc = DataDef::new(ARGC_GLOBAL);
    argc.add_dir(0);
    b.add_def(argc);

    let mut argv = DataDef::new(ARGC_GLOBAL);
    argv.add_dir(0);
    b.add_def(argv);

    b.new_block("main");
    b.save_word(Register::A0, ARGC_GLOBAL);
    b.save_word(Register::A1, ARGV_GLOBAL);
}

fn teardown_main(b: &mut MipsBuilder) {
    b.add_syscall(10);

    b.new_block(GET_FLOAT_BOOL);
    b.branch_float_false(GET_FLOAT_BOOL_FALSE);
    b.const_word(1, Register::V0);
    b.jump_register(Register::ReturnAddr);
    b.new_block(GET_FLOAT_BOOL_FALSE);
    b.const_word(0, Register::V0);
    b.jump_register(Register::ReturnAddr);
}
