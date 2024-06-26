use deimos_ast::*;
use mips_builder::{DataDef, MipsBuilder, Register};

mod const_expr;
mod error;
mod expr;
mod internal;
mod names;
mod scope;
mod stmt;

use names::*;

use error::ValidationResult;
use scope::{ConstructCounter, GlobalScope, LocalScope, Scope};

fn codegen_sub(
    b: &mut MipsBuilder,
    sub: &Function,
    scope: &Scope,
    p: &Program,
    c: &mut ConstructCounter,
) -> ValidationResult<()> {
    c.enter_fn(sub.name.data);
    b.new_block(get_fn_name(sub.name.data));
    scope.init_stack(b)?;
    scope.init_stack_ptr(b);

    stmt::codegen_block(b, &sub.block.block, scope, p, c)?;

    b.new_block(get_fn_end(sub.name.data));
    scope.restore_ra(b);
    scope.cleanup_stack(b);
    b.jump_register(Register::ReturnAddr);

    c.clear_fn();
    Ok(())
}

fn codegen_main(
    b: &mut MipsBuilder,
    global: &GlobalScope,
    p: &Program,
    c: &mut ConstructCounter,
) -> ValidationResult<()> {
    let local = LocalScope::from_program(&p.body)?;
    let scope = Scope::new(&local, global);
    scope.init_stack(b)?;
    stmt::codegen_block(b, &p.body.block, &scope, p, c)
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
    let mut counter = ConstructCounter::default();

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

    internal::setup_main(&mut codegen);
    codegen_main(&mut codegen, &global, &p, &mut counter)?;
    internal::teardown_main(&mut codegen);

    for (fnc, local) in p.fns.iter().zip(fnc_scopes.iter()) {
        let scope = Scope::new(&local, &global);
        codegen_sub(&mut codegen, fnc, &scope, p, &mut counter)?;
    }

    Ok(codegen.codegen())
}
