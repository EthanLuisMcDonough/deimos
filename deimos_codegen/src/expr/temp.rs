use std::{collections::HashSet, fmt::Debug};

use deimos_ast::*;
use mips_builder::{FloatRegister, GenericRegister, MipsAddress, MipsBuilder, Register};

use crate::expr::ValidationError;

use super::ValidationResult;

/// Represents an expression temp value that needed to be spilled
/// onto the stack. All virtual registers are word-sized and the
/// index is equal to (offset + 1) * -4
/// Virtual registers are zero-indexed but the top stack pointer is
/// never accessed this way. Virtual register offsets are always lookaheads
/// (negative)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VirtualRegister {
    offset: usize,
}

impl From<VirtualRegister> for MipsAddress<'_> {
    fn from(value: VirtualRegister) -> Self {
        MipsAddress::RegisterOffset {
            register: Register::StackPtr,
            offset: (value.offset + 1) as i32 * -4,
        }
    }
}

/// Represents a register that might be spilled onto the stack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrVirtual<T> {
    Virtual(VirtualRegister),
    Register(T),
}

impl<T> From<T> for OrVirtual<T> {
    fn from(value: T) -> Self {
        OrVirtual::Register(value)
    }
}

pub type ExprRegularRegister = OrVirtual<Register>;
pub type ExprFloatRegister = OrVirtual<FloatRegister>;
pub type ExprRegister = OrVirtual<GenericRegister>;

impl<T: Clone> OrVirtual<T> {
    /// Helper function for accessing a virtual register.
    /// Treat the temp register as the actual register and handle
    /// read/write operations before and after access
    fn use_val<R>(
        self,
        b: &mut MipsBuilder,
        temp_reg: T,
        access_mode: AccessMode,
        read_fnc: impl FnOnce(&mut MipsBuilder, T, VirtualRegister),
        fnc: impl FnOnce(&mut MipsBuilder, T) -> R,
        write_fnc: impl FnOnce(&mut MipsBuilder, T, VirtualRegister),
    ) -> R {
        match self {
            Self::Register(r) => fnc(b, r),
            Self::Virtual(v) => {
                if access_mode.read_mode() {
                    read_fnc(b, temp_reg.clone(), v);
                }
                let ret = fnc(b, temp_reg.clone());
                if access_mode.write_mode() {
                    write_fnc(b, temp_reg, v);
                }
                ret
            }
        }
    }
}

/// Describes the nature of a register access
/// This is used to decide whether a virtual register value is loaded
/// before the access and whether the temp register value is saved after
/// acess
pub enum AccessMode {
    Read,
    Write,
    ReadWrite,
}

impl AccessMode {
    pub fn read_mode(&self) -> bool {
        match self {
            Self::Read | Self::ReadWrite => true,
            Self::Write => false,
        }
    }

    pub fn write_mode(&self) -> bool {
        match self {
            Self::Write | Self::ReadWrite => true,
            Self::Read => false,
        }
    }
}

impl ExprRegularRegister {
    pub fn use_reg<R>(
        self,
        b: &mut MipsBuilder,
        temp_index: usize,
        access_mode: AccessMode,
        fnc: impl FnOnce(&mut MipsBuilder, Register) -> R,
    ) -> R {
        self.use_val(
            b,
            EXPR_TEMP[temp_index],
            access_mode,
            |b, temp_reg, v| {
                b.load_word(temp_reg, v);
            },
            fnc,
            |b, temp_reg, v| {
                b.save_word(temp_reg, v);
            },
        )
    }

    pub fn use_reg_byte<R>(
        self,
        b: &mut MipsBuilder,
        temp_index: usize,
        access_mode: AccessMode,
        fnc: impl FnOnce(&mut MipsBuilder, Register) -> R,
    ) -> R {
        self.use_val(
            b,
            EXPR_TEMP[temp_index],
            access_mode,
            |b, temp_reg, v| {
                b.load_byte(temp_reg, v);
            },
            fnc,
            |b, temp_reg, v| {
                b.save_byte(temp_reg, v);
            },
        )
    }

    pub fn load_to(&self, b: &mut MipsBuilder, r: Register) {
        match self {
            OrVirtual::Register(old_r) => b.mov(r, *old_r),
            OrVirtual::Virtual(v) => b.load_word(r, *v),
        }
    }

    pub fn store_val(&self, b: &mut MipsBuilder, old_r: Register) {
        match self {
            OrVirtual::Register(r) => b.mov(*r, old_r),
            OrVirtual::Virtual(v) => b.save_word(old_r, *v),
        }
    }

    pub fn load_byte_to(&self, b: &mut MipsBuilder, r: Register) {
        match self {
            OrVirtual::Register(old_r) => b.mov(r, *old_r),
            OrVirtual::Virtual(v) => b.load_byte(r, *v),
        }
    }
}

impl ExprFloatRegister {
    pub fn use_reg<R>(
        self,
        b: &mut MipsBuilder,
        temp_index: usize,
        access_mode: AccessMode,
        fnc: impl FnOnce(&mut MipsBuilder, FloatRegister) -> R,
    ) -> R {
        self.use_val(
            b,
            FLOAT_TEMP[temp_index],
            access_mode,
            |b, temp_reg, v| {
                b.load_f32(temp_reg, v);
            },
            fnc,
            |b, temp_reg, v| {
                b.save_f32(temp_reg, v);
            },
        )
    }

    pub fn load_to(&self, b: &mut MipsBuilder, r: FloatRegister) {
        match self {
            OrVirtual::Register(old_r) => b.mov_f32(r, *old_r),
            OrVirtual::Virtual(v) => b.load_f32(r, *v),
        }
    }
}

impl ExprRegister {
    pub fn get_word(self) -> ValidationResult<ExprRegularRegister> {
        match self {
            Self::Register(GenericRegister::Regular(r)) => Ok(OrVirtual::Register(r)),
            Self::Register(GenericRegister::Float(f)) => Err(ValidationError::InternalFloatReg(f)),
            Self::Virtual(v) => Ok(OrVirtual::Virtual(v)),
        }
    }

    pub fn get_float(self) -> ValidationResult<ExprFloatRegister> {
        match self {
            Self::Register(GenericRegister::Regular(r)) => Err(ValidationError::InternalIntReg(r)),
            Self::Register(GenericRegister::Float(f)) => Ok(OrVirtual::Register(f)),
            Self::Virtual(v) => Ok(OrVirtual::Virtual(v)),
        }
    }
}

impl From<ExprRegularRegister> for ExprRegister {
    fn from(value: ExprRegularRegister) -> Self {
        match value {
            ExprRegularRegister::Virtual(v) => ExprRegister::Virtual(v),
            ExprRegularRegister::Register(r) => ExprRegister::Register(r.into()),
        }
    }
}

impl From<ExprFloatRegister> for ExprRegister {
    fn from(value: ExprFloatRegister) -> Self {
        match value {
            ExprFloatRegister::Virtual(v) => ExprRegister::Virtual(v),
            ExprFloatRegister::Register(r) => ExprRegister::Register(r.into()),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub struct ExprType {
    pub base: PrimitiveType,
    pub indirection: usize,
}

impl ExprType {
    pub fn ref_type(self) -> Self {
        Self {
            base: self.base,
            indirection: self.indirection + 1,
        }
    }

    pub fn deref_type(self) -> Self {
        Self {
            base: self.base,
            indirection: self.indirection - 1,
        }
    }
}

impl From<ParamType> for ExprType {
    fn from(value: ParamType) -> Self {
        Self {
            base: value.param_type.data,
            indirection: value.indirection,
        }
    }
}

impl From<DeclType> for ExprType {
    fn from(value: DeclType) -> Self {
        match value {
            DeclType::Param(p) => ExprType::from(p.data),
            DeclType::Array {
                array_type,
                size: _,
            } => {
                let mut param_type = array_type.data.clone();
                param_type.indirection += 1;
                ExprType::from(param_type)
            }
        }
    }
}

impl From<PrimitiveType> for ExprType {
    fn from(value: PrimitiveType) -> Self {
        Self {
            base: value,
            indirection: 0,
        }
    }
}

/// Represents a temporary value living in a register.
#[derive(Debug)]
pub struct ExprTemp {
    pub register: ExprRegister,
    pub computed_type: ExprType,
}

impl ExprTemp {
    pub fn new(r: impl Into<ExprRegister>, typ: impl Into<ExprType>) -> Self {
        Self {
            register: r.into(),
            computed_type: typ.into(),
        }
    }

    pub fn type_tuple(&self) -> (PrimitiveType, usize) {
        (self.computed_type.base, self.computed_type.indirection)
    }
}

static EXPR_REGISTERS: &[Register] = &[
    Register::T0,
    Register::T1,
    Register::T2,
    Register::T3,
    Register::T4,
    Register::T5,
    Register::T6,
    Register::T7,
];

/// Registers used for handling virtual register values
pub static EXPR_TEMP: [Register; 2] = [Register::T8, Register::T9];

static EXPR_FLOAT_REGISTERS: &[FloatRegister] = &[
    FloatRegister::F4,
    FloatRegister::F5,
    FloatRegister::F6,
    FloatRegister::F7,
    FloatRegister::F8,
    FloatRegister::F9,
    FloatRegister::F10,
    FloatRegister::F11,
    FloatRegister::F16,
    FloatRegister::F17,
];

/// Registers used for handling floating virtual register values
pub static FLOAT_TEMP: [FloatRegister; 2] = [FloatRegister::F18, FloatRegister::F19];

#[derive(Default)]
pub struct RegisterBank {
    registers: HashSet<Register>,
    float_regs: HashSet<FloatRegister>,
    virtual_reg: HashSet<usize>,
}

impl RegisterBank {
    pub fn clear(&mut self) {
        self.registers.clear();
        self.float_regs.clear();
        self.virtual_reg.clear();
    }

    fn get_virtual(&mut self) -> VirtualRegister {
        let mut offset = 0usize;
        while self.virtual_reg.contains(&offset) {
            offset += 1;
        }
        self.virtual_reg.insert(offset);
        VirtualRegister { offset }
    }

    pub fn get_register(&mut self) -> ExprRegularRegister {
        EXPR_REGISTERS
            .iter()
            .find(|&&r| self.registers.insert(r))
            .map(|&r| OrVirtual::Register(r))
            .unwrap_or_else(|| OrVirtual::Virtual(self.get_virtual()))
    }

    pub fn get_float_reg(&mut self) -> ExprFloatRegister {
        EXPR_FLOAT_REGISTERS
            .iter()
            .find(|&&f| self.float_regs.insert(f))
            .map(|&f| OrVirtual::Register(f))
            .unwrap_or_else(|| OrVirtual::Virtual(self.get_virtual()))
    }

    pub fn free_reg(&mut self, reg: impl Into<ExprRegister>) {
        match reg.into() {
            OrVirtual::Register(GenericRegister::Float(f)) => {
                self.float_regs.remove(&f);
            }
            OrVirtual::Register(GenericRegister::Regular(r)) => {
                self.registers.remove(&r);
            }
            OrVirtual::Virtual(VirtualRegister { offset }) => {
                self.virtual_reg.remove(&offset);
            }
        }
    }
}
