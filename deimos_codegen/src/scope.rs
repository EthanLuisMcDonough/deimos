use super::{error::*, names::get_static_name};
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

#[derive(Clone)]
pub struct LocatedValue {
    pub loc: ValLocation,
    pub val: DeclType,
}

#[derive(Default)]
pub struct LocalScope {
    vars: HashMap<usize, LocalStackVal>,
    return_addr_offset: u32,
    stack_size: u32,
}

impl LocalScope {
    pub fn insert_ra(&mut self) {
        self.stack_size += 4;
        self.return_addr_offset = self.stack_size;
    }

    pub fn get_ra_stack_offset(&self) -> u32 {
        self.stack_size - self.return_addr_offset
    }

    pub fn get_ra_stack_loc(&self) -> ValLocation {
        ValLocation::Stack(self.get_ra_stack_offset())
    }

    pub fn insert(&mut self, name: Identifier, typ: impl Into<DeclType>) -> ValidationResult<()> {
        let typ = typ.into();
        self.stack_size += get_def_size(&typ);
        let ins_val = LocalStackVal {
            val_type: typ,
            offset: self.stack_size,
        };
        if self.vars.insert(name.data, ins_val).is_some() {
            return Err(ValidationError::new(
                ValidationErrorKind::Redefinition,
                name.loc,
            ));
        }
        Ok(())
    }

    pub fn get_local_var(&self, name: Identifier) -> Option<LocatedValue> {
        self.vars.get(&name.data).map(|val| LocatedValue {
            loc: ValLocation::Stack(self.stack_size - val.offset),
            val: val.val_type.clone(),
        })
    }

    pub fn get_var(
        &self,
        name: Identifier,
        global: &GlobalScope,
    ) -> ValidationResult<LocatedValue> {
        self.get_local_var(name)
            .map(Ok)
            .unwrap_or_else(|| global.get_val(name).cloned())
    }

    pub fn get_fn<'a>(
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

    pub fn get_stack_size(&self) -> u32 {
        self.stack_size
    }

    pub fn init_stack(&self, b: &mut MipsBuilder, vars: &Vec<VarDecl>) -> ValidationResult<()> {
        b.const_word(self.get_stack_size(), Register::T0);
        b.sub_i32(Register::StackPtr, Register::StackPtr, Register::T0);
        b.save_word(Register::ReturnAddr, self.get_ra_stack_loc());
        for var in vars {
            //var.init
        }
        Ok(())
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

#[derive(Clone, Copy)]
pub struct BlockMeta {
    pub in_loop: bool,
    pub in_func: Option<usize>,
}

impl BlockMeta {
    pub fn new(f: impl Into<Option<usize>>) -> Self {
        Self {
            in_loop: false,
            in_func: f.into(),
        }
    }

    pub fn with_loop(&self) -> Self {
        Self {
            in_loop: true,
            in_func: self.in_func.clone(),
        }
    }
}
