use deimos_ast::*;
use mips_builder::{DataDef, MipsBuilder, Register};

mod const_expr;
mod error;
mod names;
mod scope;
mod expr;

use names::*;

use error::ValidationResult;
use scope::{GlobalScope, LocalScope, Scope};

use crate::error::ValidationError;

fn codegen_sub(b: &mut MipsBuilder, sub: &Function, scope: &Scope) -> ValidationResult<()> {
    b.new_block(get_fn_name(sub.name.data));
    scope.init_stack(b)?;

    codegen_block(b, &sub.block.block, scope)?;

    b.new_block(get_fn_end(sub.name.data));
    scope.restore_ra(b);
    scope.cleanup_stack(b);
    b.jump_register(Register::ReturnAddr);

    Ok(())
}

fn codegen_block(b: &mut MipsBuilder, block: &Block, scope: &Scope) -> ValidationResult<()> {
    for stmt in block {
        codegen_stmt(b, &stmt.data, scope)?;
    }
    Ok(())
}

fn codegen_stmt(b: &mut MipsBuilder, stmt: &Statement, s: &Scope) -> ValidationResult<()> {
    match stmt {
        _ => {}
    }
    Ok(())
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
        fnc_scopes.push((fnc.name.data, LocalScope::from_fn(fnc)?));
        global.insert_fn(fnc);
    }

    let mut codegen = MipsBuilder::new();
    /*codegen.new_block("main");
    codegen.const_word(10, Register::T0);
    codegen.const_word(11, Register::T1);
    codegen.add_i32(Register::A0, Register::T0, Register::T1);
    codegen.add_syscall(1);
    codegen.add_syscall(10);*/
    for (id, local) in fnc_scopes {
        let scope = Scope::new(&local, &global);
        codegen_sub(&mut codegen, &p.fns[id], &scope)?;
    }
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
