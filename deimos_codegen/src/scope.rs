use super::{const_expr::codegen_init_var, error::*, names::get_static_name};
use deimos_ast::*;
use mips_builder::{MipsAddress, MipsBuilder, Register};
use std::collections::HashMap;

/// Calculate size of array type
fn get_def_size(d: &DeclType) -> u32 {
    match d {
        DeclType::Param(_) => 4,
        DeclType::Array {
            size,
            array_type:
                Located {
                    data:
                        ParamType {
                            param_type:
                                Located {
                                    data: PrimitiveType::U8,
                                    ..
                                },
                            ..
                        },
                    ..
                },
            ..
        } => size.data.div_ceil(4) * 4,
        DeclType::Array { size, .. } => 4 * size.data,
    }
}

/// Private type for representing local variables. The offset here
/// is the inverse of the working stack offset in that it represents
/// the running size of the stack as variables are inserted. Subtract
/// this value from the stack size and you'll get the actual offset.
#[derive(Debug)]
struct StackVal {
    val_type: DeclType,
    data: StackValType,
}

#[derive(Debug)]
enum StackValType {
    LocalVar {
        offset: u32,
        init_val: Option<Located<InitValue>>,
    },
    Argument {
        offset: u32,
    },
}

/// Minimal type that can represent addresses without any strings.
/// Can be converted into MipsAddress later
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct LocatedValue {
    pub loc: ValLocation,
    pub val: DeclType,
}

/// Represents a function scope
#[derive(Default)]
pub struct LocalScope {
    vars: HashMap<usize, StackVal>,
    return_addr_offset: u32,
    local_stack_size: u32,
    arg_stack_size: u32,
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
            self.insert_local(local_var)?;
        }
        Ok(())
    }

    /// Insert the return address at current position
    fn insert_ra(&mut self) {
        self.local_stack_size += 4;
        self.return_addr_offset = self.local_stack_size;
    }

    /// Gets stack offset for return address variable
    fn get_ra_stack_offset(&self) -> u32 {
        let stack_data = StackValType::LocalVar {
            offset: self.return_addr_offset,
            init_val: None,
        };
        self.calc_offset(&stack_data)
    }

    fn get_ra_stack_loc(&self) -> ValLocation {
        ValLocation::Stack(self.get_ra_stack_offset())
    }

    /// Insert argument into function scope
    fn insert_arg(&mut self, name: Identifier, typ: impl Into<DeclType>) -> ValidationResult<()> {
        let typ = typ.into();
        self.arg_stack_size += get_def_size(&typ);
        let ins_val = StackVal {
            val_type: typ,
            data: StackValType::Argument {
                offset: self.arg_stack_size,
            },
        };
        self.insert_val_internal(name, ins_val)
    }

    /// Insert local variable into funciton scope
    fn insert_local(&mut self, var: &VarDecl) -> ValidationResult<()> {
        self.local_stack_size += get_def_size(&var.variable);
        let ins_val = StackVal {
            val_type: var.variable.clone(),
            data: StackValType::LocalVar {
                offset: self.local_stack_size,
                init_val: var.init.clone(),
            },
        };
        self.insert_val_internal(var.name, ins_val)
    }

    fn insert_val_internal(&mut self, name: Identifier, val: StackVal) -> ValidationResult<()> {
        if self.vars.insert(name.data, val).is_some() {
            Err(ValidationError::Redefinition(name.loc))
        } else {
            Ok(())
        }
    }

    fn get_local_var(&self, name: Identifier, stack_shift: u32) -> Option<LocatedValue> {
        self.vars.get(&name.data).map(|val| LocatedValue {
            loc: ValLocation::Stack(self.calc_offset(&val.data) + stack_shift),
            val: val.val_type.clone(),
        })
    }

    fn get_var(
        &self,
        name: Identifier,
        global: &GlobalScope,
        stack_shift: u32,
    ) -> ValidationResult<LocatedValue> {
        self.get_local_var(name, stack_shift)
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
            return Err(ValidationError::ShadowedFuncCall(name.loc));
        }
        global.get_fn(name)
    }

    /// Padded size of the stack
    fn get_local_stack_size(&self) -> u32 {
        self.local_stack_size.div_ceil(4) * 4
    }

    /// Padded size of the args
    fn get_arg_stack_size(&self) -> u32 {
        self.arg_stack_size.div_ceil(4) * 4
    }

    fn get_stack_size(&self) -> u32 {
        self.get_arg_stack_size() + self.get_local_stack_size()
    }

    fn calc_offset_local(&self, offset: u32) -> u32 {
        self.get_local_stack_size() - offset
    }

    fn calc_offset_arg(&self, offset: u32) -> u32 {
        self.get_stack_size() - offset
    }

    fn calc_offset(&self, val: &StackValType) -> u32 {
        match val {
            StackValType::Argument { offset } => self.calc_offset_arg(*offset),
            StackValType::LocalVar { offset, .. } => self.calc_offset_local(*offset),
        }
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
        self.vars
            .get(&name.data)
            .ok_or(ValidationError::UndefinedIdent(name.loc))
    }

    fn get_fn(&self, name: Identifier) -> ValidationResult<&FunctionArgs> {
        match self.get(name)? {
            GlobalVal::Fnc(args) => Ok(args),
            GlobalVal::Val(_) => Err(ValidationError::NotAFunc(name.loc)),
        }
    }

    fn get_val(&self, name: Identifier) -> ValidationResult<&LocatedValue> {
        match self.get(name)? {
            GlobalVal::Val(args) => Ok(args),
            GlobalVal::Fnc(_) => Err(ValidationError::FuncInExpr(name.loc)),
        }
    }

    pub fn insert_mem(&mut self, mem: &MemVar) -> ValidationResult<()> {
        if mem.var.field_type.data.indirection == 0 {
            return Err(ValidationError::InvalidMemVarType(mem.var.field_type.loc));
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
    stack_shift: u32,
}

impl<'a> Scope<'a> {
    pub fn new(local: &'a LocalScope, global: &'a GlobalScope) -> Scope<'a> {
        Scope {
            local,
            global,
            stack_shift: 0,
        }
    }

    pub fn shift_stack(&self, shift: u32) -> Scope<'a> {
        Scope {
            local: self.local,
            global: self.global,
            stack_shift: self.stack_shift + shift,
        }
    }

    pub fn get_var(&self, name: Identifier) -> ValidationResult<LocatedValue> {
        self.local.get_var(name, self.global, self.stack_shift)
    }

    pub fn get_fn(&self, name: Identifier) -> ValidationResult<&'a FunctionArgs> {
        self.local.get_fn(name, &self.global)
    }

    /// Allocate enough space for the return address and local variables
    pub fn init_stack(&self, b: &mut MipsBuilder) -> ValidationResult<()> {
        let neg_stack = -(self.local.get_local_stack_size() as i32);
        b.add_const_i32(Register::StackPtr, Register::StackPtr, neg_stack);
        for val in self.local.vars.values() {
            if let StackValType::LocalVar {
                offset,
                init_val: Some(init),
            } = &val.data
            {
                let var_offset = self.local.calc_offset_local(*offset) as i32;
                codegen_init_var(b, val.val_type.clone(), init, var_offset)?;
            }
        }
        Ok(())
    }

    pub fn init_stack_ptr(&self, b: &mut MipsBuilder) {
        b.save_word(Register::ReturnAddr, self.local.get_ra_stack_loc());
    }

    pub fn cleanup_stack(&self, b: &mut MipsBuilder) {
        b.add_const_i32(
            Register::StackPtr,
            Register::StackPtr,
            self.local.get_stack_size() as i32,
        );
    }

    pub fn restore_ra(&self, b: &mut MipsBuilder) {
        b.load_word(Register::ReturnAddr, self.local.get_ra_stack_loc());
    }
}

#[derive(Default)]
pub struct ConstructCounter {
    if_count: usize,
    loop_count: usize,
    loop_stack: Vec<usize>,
    in_func: Option<usize>,
}

impl ConstructCounter {
    fn new_loop(&mut self) -> usize {
        let old = self.loop_count;
        self.loop_count += 1;
        old
    }

    pub fn start_loop(&mut self) -> usize {
        let ind = self.new_loop();
        self.loop_stack.push(ind);
        ind
    }

    pub fn end_loop(&mut self) -> usize {
        self.loop_stack.pop().expect("Loop end without start")
    }

    pub fn new_if(&mut self) -> usize {
        let old = self.if_count;
        self.if_count += 1;
        old
    }

    pub fn enter_fn(&mut self, fnc_id: usize) {
        self.in_func = fnc_id.into();
    }

    pub fn clear_fn(&mut self) {
        self.in_func = None;
    }

    pub fn get_current_loop(&self) -> Option<usize> {
        self.loop_stack.last().cloned()
    }

    pub fn get_current_fn(&self) -> Option<usize> {
        self.in_func
    }
}
