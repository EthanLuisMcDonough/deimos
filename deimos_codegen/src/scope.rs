use super::{const_expr::codegen_init_var, error::*, names::get_static_name};
use deimos_ast::*;
use mips_builder::{MipsAddress, MipsBuilder, Register};
use std::collections::HashMap;

/// Calculate size of scalar type
fn get_ref_size(p: &ParamType) -> u32 {
    match p.param_type.data {
        PrimitiveType::U8 if p.indirection == 0 => 1,
        _ => 4,
    }
}

/// Calculate size of array type
fn get_def_size(d: &DeclType) -> u32 {
    match d {
        DeclType::Param(p) => get_ref_size(&p.data),
        DeclType::Array { array_type, size } => get_ref_size(&array_type.data) * size.data,
    }
}

/// Private type for representing local variables. The offset here
/// is the inverse of the working stack offset in that it represents
/// the running size of the stack as variables are inserted. Subtract
/// this value from the stack size and you'll get the actual offset.
struct LocalStackVal {
    val_type: DeclType,
    init_val: Option<Located<InitValue>>,
    offset: u32,
}

/// Minimal type that can represent addresses without any strings.
/// Can be converted into MipsAddress later
#[derive(Clone)]
pub enum ValLocation {
    Static(usize),
    RawAddr(u32),
    Stack(u32),
}

impl<'a> From<ValLocation> for MipsAddress<'a> {
    fn from(value: ValLocation) -> Self {
        match value {
            ValLocation::RawAddr(a) => MipsAddress::Addr(a),
            ValLocation::Static(i) => MipsAddress::Label(get_static_name(i).into()),
            ValLocation::Stack(offset) => MipsAddress::RegisterOffset {
                register: Register::StackPtr,
                offset: offset as i32,
            },
        }
    }
}

/// Copy safe located + typed value
#[derive(Clone)]
pub struct LocatedValue {
    pub loc: ValLocation,
    pub val: DeclType,
}

/// Represents a function scope
#[derive(Default)]
pub struct LocalScope {
    vars: HashMap<usize, LocalStackVal>,
    return_addr_offset: u32,
    stack_size: u32,
}

impl LocalScope {
    pub fn from_fn(fnc: &Function) -> ValidationResult<Self> {
        let mut local = Self::default();
        for param in &fnc.args {
            local.insert_arg(param.name, param.field_type.clone())?;
        }
        local.insert_ra();
        local.insert_fn_body(&fnc.block)?;
        Ok(local)
    }

    pub fn from_program(fnc: &FunctionBlock) -> ValidationResult<Self> {
        let mut local = Self::default();
        local.insert_fn_body(fnc)?;
        Ok(local)
    }

    fn insert_fn_body(&mut self, block: &FunctionBlock) -> ValidationResult<()> {
        for local_var in &block.vars {
            self.insert_local(
                local_var.name,
                local_var.variable.clone(),
                local_var.init.clone(),
            )?;
        }
        Ok(())
    }

    /// Insert the return address at current position
    fn insert_ra(&mut self) {
        self.stack_size += 4;
        self.return_addr_offset = self.stack_size;
    }

    /// Gets stack offset for return address variable
    fn get_ra_stack_offset(&self) -> u32 {
        self.calc_offset(self.return_addr_offset)
    }

    fn get_ra_stack_loc(&self) -> ValLocation {
        ValLocation::Stack(self.get_ra_stack_offset())
    }

    /// Insert argument into function scope
    fn insert_arg(&mut self, name: Identifier, typ: impl Into<DeclType>) -> ValidationResult<()> {
        self.insert_local(name, typ, None)
    }

    /// Insert local variable into funciton scope
    fn insert_local(
        &mut self,
        name: Identifier,
        typ: impl Into<DeclType>,
        init: impl Into<Option<Located<InitValue>>>,
    ) -> ValidationResult<()> {
        let typ = typ.into();
        self.stack_size += get_def_size(&typ);
        let ins_val = LocalStackVal {
            val_type: typ,
            offset: self.stack_size,
            init_val: init.into(),
        };
        if self.vars.insert(name.data, ins_val).is_some() {
            return Err(ValidationError::new(
                ValidationErrorKind::Redefinition,
                name.loc,
            ));
        }
        Ok(())
    }

    fn get_local_var(&self, name: Identifier) -> Option<LocatedValue> {
        self.vars.get(&name.data).map(|val| LocatedValue {
            loc: ValLocation::Stack(self.calc_offset(val.offset)),
            val: val.val_type.clone(),
        })
    }

    fn get_var(&self, name: Identifier, global: &GlobalScope) -> ValidationResult<LocatedValue> {
        self.get_local_var(name)
            .map(Ok)
            .unwrap_or_else(|| global.get_val(name).cloned())
    }

    /// Gets function from global scope. Checks local scope for any
    /// variables that might shadow the desired function.
    fn get_fn<'a>(
        &self,
        name: Identifier,
        global: &'a GlobalScope,
    ) -> ValidationResult<&'a FunctionArgs> {
        if self.vars.contains_key(&name.data) {
            return Err(ValidationError::new(
                ValidationErrorKind::ShadowedFuncCall,
                name.loc,
            ));
        }
        global.get_fn(name)
    }

    /// Padded size of the stack
    fn get_stack_size(&self) -> u32 {
        self.stack_size.div_ceil(4) * 4
    }

    fn calc_offset(&self, offset: u32) -> u32 {
        self.get_stack_size() - offset
    }
}

enum GlobalVal {
    Val(LocatedValue),
    Fnc(FunctionArgs),
}

#[derive(Default)]
pub struct GlobalScope {
    vars: HashMap<usize, GlobalVal>,
}

impl GlobalScope {
    fn get(&self, name: Identifier) -> ValidationResult<&GlobalVal> {
        self.vars.get(&name.data).ok_or(ValidationError::new(
            ValidationErrorKind::UndefinedIdent,
            name.loc,
        ))
    }

    fn get_fn(&self, name: Identifier) -> ValidationResult<&FunctionArgs> {
        match self.get(name)? {
            GlobalVal::Fnc(args) => Ok(args),
            GlobalVal::Val(_) => Err(ValidationError::new(
                ValidationErrorKind::NotAFunc,
                name.loc,
            )),
        }
    }

    fn get_val(&self, name: Identifier) -> ValidationResult<&LocatedValue> {
        match self.get(name)? {
            GlobalVal::Val(args) => Ok(args),
            GlobalVal::Fnc(_) => Err(ValidationError::new(
                ValidationErrorKind::FuncInExpr,
                name.loc,
            )),
        }
    }

    pub fn insert_mem(&mut self, mem: &MemVar) -> ValidationResult<()> {
        if mem.var.field_type.data.indirection == 0 {
            return Err(ValidationError::new(
                ValidationErrorKind::InvalidMemVarType,
                mem.var.field_type.loc,
            ));
        }
        self.vars.insert(
            mem.var.name.data,
            GlobalVal::Val(LocatedValue {
                loc: ValLocation::RawAddr(mem.addr.data),
                val: mem.var.field_type.clone().into(),
            }),
        );
        Ok(())
    }

    pub fn insert_static(&mut self, static_var: &VarDecl) {
        self.vars.insert(
            static_var.name.data,
            GlobalVal::Val(LocatedValue {
                loc: ValLocation::Static(static_var.name.data),
                val: static_var.variable.clone(),
            }),
        );
    }

    pub fn insert_fn(&mut self, fnc: &Function) {
        self.vars
            .insert(fnc.name.data, GlobalVal::Fnc(fnc.args.clone()));
    }
}

pub struct Scope<'a> {
    local: &'a LocalScope,
    global: &'a GlobalScope,
}

impl<'a> Scope<'a> {
    pub fn new(local: &'a LocalScope, global: &'a GlobalScope) -> Scope<'a> {
        Scope { local, global }
    }

    pub fn get_var(&self, name: Identifier) -> ValidationResult<LocatedValue> {
        self.local.get_var(name, self.global)
    }

    pub fn get_fn(&self, name: Identifier) -> ValidationResult<&'a FunctionArgs> {
        self.local.get_fn(name, &self.global)
    }

    pub fn init_stack(&self, b: &mut MipsBuilder) -> ValidationResult<()> {
        b.const_word(self.local.get_stack_size(), Register::T0);
        b.sub_i32(Register::StackPtr, Register::StackPtr, Register::T0);
        b.save_word(Register::ReturnAddr, self.local.get_ra_stack_loc());
        for val in self.local.vars.values() {
            codegen_init_var(
                b,
                val.val_type.clone(),
                &val.init_val,
                self.local.calc_offset(val.offset) as i32,
            )?;
        }
        Ok(())
    }

    pub fn cleanup_stack(&self, b: &mut MipsBuilder) {
        b.const_word(self.local.get_stack_size(), Register::T0);
        b.add_i32(Register::StackPtr, Register::StackPtr, Register::T0);
    }

    pub fn restore_ra(&self, b: &mut MipsBuilder) {
        b.load_word(Register::ReturnAddr, self.local.get_ra_stack_loc());
    }
}

#[derive(Clone, Copy)]
pub struct BlockMeta {
    pub construct_count: usize,
    pub loop_count: usize,
    pub in_func: Option<usize>,
}

impl BlockMeta {
    pub fn new_loop(&mut self) -> usize {
        let old = self.loop_count;
        self.loop_count += 1;
        old
    }

    pub fn new_construct(&mut self) -> usize {
        let old = self.construct_count;
        self.construct_count += 1;
        old
    }
}
