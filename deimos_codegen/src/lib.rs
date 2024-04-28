use deimos_ast::*;
use mips_builder::{DataDef, MipsBuilder, Register};

mod const_expr;
mod error;
mod expr;
mod names;
mod scope;
mod stmt;

use names::*;

use error::ValidationResult;
use scope::{GlobalScope, LocalScope, Scope};

fn codegen_sub(
    b: &mut MipsBuilder,
    sub: &Function,
    scope: &Scope,
    p: &Program,
) -> ValidationResult<()> {
    b.new_block(get_fn_name(sub.name.data));
    scope.init_stack(b)?;
    scope.init_stack_ptr(b);

    codegen_block(b, &sub.block.block, scope, p)?;

    b.new_block(get_fn_end(sub.name.data));
    scope.restore_ra(b);
    scope.cleanup_stack(b);
    b.jump_register(Register::ReturnAddr);

    Ok(())
}

fn codegen_block(
    b: &mut MipsBuilder,
    block: &Block,
    scope: &Scope,
    p: &Program,
) -> ValidationResult<()> {
    for stmt in block {
        stmt::codegen_stmt(b, &stmt.data, scope, p)?;
    }
    Ok(())
}

fn codegen_main(b: &mut MipsBuilder, global: &GlobalScope, p: &Program) -> ValidationResult<()> {
    let local = LocalScope::from_program(&p.body)?;
    let scope = Scope::new(&local, global);
    scope.init_stack(b)?;
    codegen_block(b, &p.body.block, &scope, p)
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
        fnc_scopes.push(LocalScope::from_fn(fnc)?);
        global.insert_fn(fnc);
    }

    let mut codegen = MipsBuilder::new();

    // Init static vars
    for static_var in &p.static_vars {
        const_expr::codegen_init_static(&mut codegen, &p.bank, static_var)?;
    }
    // Init string static vars
    for (str_id, str_val) in p.bank.strings.iter().enumerate() {
        let static_name = names::get_str_name(str_id);
        let mut str_def = DataDef::new(static_name);
        str_def.add_dir(format!("\"{}\"", str_val));
        codegen.add_def(str_def);
    }

    setup_main(&mut codegen);
    codegen_main(&mut codegen, &global, &p)?;
    teardown_main(&mut codegen);

    for (fnc, local) in p.fns.iter().zip(fnc_scopes.iter()) {
        let scope = Scope::new(&local, &global);
        codegen_sub(&mut codegen, fnc, &scope, p)?;
    }

    Ok(codegen.codegen())
}

fn setup_main(b: &mut MipsBuilder) {
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
