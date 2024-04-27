/// Module responsible for codegen for static expressions (local variable and
/// static var declaration)
use crate::error::*;
use crate::names::{get_static_name, get_str_name};
use deimos_ast::*;
use mips_builder::*;

/// Create directive for scalar static variable
fn init_static_param(
    bank: &StringBank,
    param_type: &ParamType,
    init_val: &Option<Located<InitValue>>,
) -> ValidationResult<DataDirective> {
    match (param_type.param_type.data, param_type.indirection, init_val) {
        (PrimitiveType::F32, 0, None) => Ok(DataDirective::from(0.0)),
        (
            PrimitiveType::F32,
            0,
            Some(Located {
                data: InitValue::Primitive(PrimitiveValue::Float(f)),
                ..
            }),
        ) => Ok(DataDirective::from(*f)),
        (PrimitiveType::I32 | PrimitiveType::U32, 0, None) | (_, 1.., None) => {
            Ok(DataDirective::from(0i32))
        }
        (PrimitiveType::U8, 0, None) => Ok(DataDirective::from(0u8)),
        (
            PrimitiveType::I32 | PrimitiveType::U32,
            0,
            Some(Located {
                data: InitValue::Primitive(PrimitiveValue::Int(i)),
                ..
            }),
        ) => Ok(DataDirective::from(*i)),
        (
            PrimitiveType::I32 | PrimitiveType::U32,
            0,
            Some(Located {
                data: InitValue::Primitive(PrimitiveValue::Unsigned(i)),
                ..
            }),
        ) => Ok(DataDirective::from(*i)),
        (
            PrimitiveType::U8,
            0,
            Some(Located {
                data: InitValue::Primitive(PrimitiveValue::Unsigned(i @ 0..=255)),
                ..
            }),
        ) => Ok(DataDirective::from(*i as u8)),
        (
            PrimitiveType::U8,
            0,
            Some(Located {
                data: InitValue::Primitive(PrimitiveValue::Int(i @ 0..=255)),
                ..
            }),
        ) => Ok(DataDirective::from(*i as u8)),
        (
            PrimitiveType::U8,
            1,
            Some(Located {
                data: InitValue::Primitive(PrimitiveValue::String(s)),
                ..
            }),
        ) => Ok(DataDirective::Asciiz(bank.strings[*s].clone())),
        (_, _, Some(Located { loc, .. })) => Err(ValidationError::InvalidStaticVar(*loc)),
    }
}

fn expect_f32(val: Located<PrimitiveValue>) -> ValidationResult<f32> {
    match val.data {
        PrimitiveValue::Float(f) => Ok(f),
        PrimitiveValue::Int(i) => Ok(i as f32),
        PrimitiveValue::Unsigned(i) => Ok(i as f32),
        _ => Err(ValidationError::MismatchedType(val.loc)),
    }
}

fn expect_word(val: Located<PrimitiveValue>) -> ValidationResult<u32> {
    match val.data {
        PrimitiveValue::Int(i) => Ok(i as u32),
        PrimitiveValue::Unsigned(i) => Ok(i),
        _ => Err(ValidationError::MismatchedType(val.loc)),
    }
}

fn expect_byte(val: Located<PrimitiveValue>) -> ValidationResult<u8> {
    match val.data {
        PrimitiveValue::Int(i) if i >= 0 && i < 256 => Ok(i as u8),
        PrimitiveValue::Unsigned(i) if i < 256 => Ok(i as u8),
        _ => Err(ValidationError::MismatchedType(val.loc)),
    }
}

fn expect_string(val: Located<PrimitiveValue>) -> ValidationResult<usize> {
    match val.data {
        PrimitiveValue::String(s) => Ok(s),
        _ => Err(ValidationError::MismatchedType(val.loc)),
    }
}

/// Create directive for array static variable
fn init_static_array(
    bank: &StringBank,
    array_type: &ParamType,
    array_size: u32,
    init_val: &Option<Located<InitValue>>,
) -> ValidationResult<DataDirective> {
    let array_size = array_size as usize;
    let directive = match (array_type.param_type.data, array_type.indirection, init_val) {
        (PrimitiveType::U8, 0, None) => DataDirective::ByteLen {
            len: array_size,
            default: 0,
        },
        (PrimitiveType::U32 | PrimitiveType::I32 | PrimitiveType::F32, 0, None)
        | (_, 1.., None) => DataDirective::WordLen {
            len: array_size,
            default: 0,
        },
        (
            PrimitiveType::F32,
            0,
            Some(Located {
                data: InitValue::List(list),
                ..
            }),
        ) if list.len() == array_size => {
            let mut float_vals = Vec::with_capacity(list.len());
            for &val in list {
                float_vals.push(expect_f32(val)?);
            }
            DataDirective::from(float_vals)
        }
        (
            PrimitiveType::I32 | PrimitiveType::U32,
            0,
            Some(Located {
                data: InitValue::List(list),
                ..
            }),
        ) if list.len() == array_size => {
            let mut int_vals = Vec::with_capacity(list.len());
            for &val in list {
                int_vals.push(expect_word(val)?);
            }
            DataDirective::from(int_vals)
        }
        (
            PrimitiveType::U8,
            0,
            Some(Located {
                data: InitValue::List(list),
                ..
            }),
        ) if list.len() == array_size => {
            let mut byte_vals = Vec::with_capacity(list.len());
            for &val in list {
                byte_vals.push(expect_byte(val)?);
            }
            DataDirective::from(byte_vals)
        }
        (
            PrimitiveType::U8,
            0,
            Some(Located {
                data: InitValue::Primitive(PrimitiveValue::String(s)),
                ..
            }),
        ) if bank.strings[*s].len() + 1 == array_size => {
            DataDirective::Asciiz(bank.strings[*s].clone())
        }
        (_, _, Some(Located { loc, .. })) => {
            return Err(ValidationError::InvalidStaticVar(*loc));
        }
    };
    Ok(directive)
}

/// Codegen for creating static variables (variables stored in .data section)
pub fn codegen_init_static(
    b: &mut MipsBuilder,
    bank: &StringBank,
    static_var: &VarDecl,
) -> ValidationResult<()> {
    let mut static_def = DataDef::new(get_static_name(static_var.name.data));
    let directive = match &static_var.variable {
        DeclType::Param(p) => init_static_param(bank, &p.data, &static_var.init)?,
        DeclType::Array { array_type, size } => {
            init_static_array(bank, &array_type.data, size.data, &static_var.init)?
        }
    };
    static_def.add_dir(directive);
    b.add_def(static_def);
    Ok(())
}

pub fn codegen_init_var(
    b: &mut MipsBuilder,
    var_type: DeclType,
    init: &Located<InitValue>,
    stack_offset: i32,
) -> ValidationResult<()> {
    match (var_type, &init.data) {
        (DeclType::Param(p), InitValue::Primitive(init_val)) => {
            stack_init_param(b, &p.data, Located::new(*init_val, init.loc), stack_offset)
        }
        (DeclType::Array { array_type, size }, InitValue::List(list)) => stack_init_array(
            b,
            &array_type.data,
            size.data,
            Some(Located::new(list, init.loc)),
            stack_offset,
        ),
        _ => Err(ValidationError::InvalidLocalInit(init.loc)),
    }
}

fn stack_init_param(
    b: &mut MipsBuilder,
    var_type: &ParamType,
    init: Located<PrimitiveValue>,
    stack_offset: i32,
) -> ValidationResult<()> {
    let address = MipsAddress::RegisterOffset {
        register: Register::StackPtr,
        offset: stack_offset,
    };
    match var_type.param_type.data {
        PrimitiveType::F32 if var_type.indirection == 0 => {
            let val = expect_f32(init)?;
            b.const_f32(val, FloatRegister::F4);
            b.save_f32(FloatRegister::F4, address);
        }
        PrimitiveType::I32 | PrimitiveType::U32 if var_type.indirection == 0 => {
            let val = expect_word(init)?;
            b.const_word(val, Register::T0);
            b.save_word(Register::T0, address);
        }
        PrimitiveType::U8 if var_type.indirection == 0 => {
            let val = expect_byte(init)?;
            b.const_word(val as u32, Register::T0);
            b.save_byte(Register::T0, address);
        }
        PrimitiveType::U8 if var_type.indirection == 1 => {
            let str_id = expect_string(init)?;
            let str_name = get_str_name(str_id);
            b.load_addr(Register::T0, MipsAddress::Label(str_name.into()));
            b.save_word(Register::T0, address);
        }
        _ => {
            return Err(ValidationError::InvalidLocalInit(init.loc));
        }
    };
    Ok(())
}

fn init_array_const(
    b: &mut MipsBuilder,
    mut offset: i32,
    slot_size: i32,
    data: &InitList,
    f: impl Fn(&mut MipsBuilder, MipsAddress, &Located<PrimitiveValue>) -> ValidationResult<()>,
) -> ValidationResult<()> {
    for val in data {
        let address = MipsAddress::RegisterOffset {
            register: Register::StackPtr,
            offset: offset,
        };
        f(b, address, val)?;
        offset += slot_size;
    }
    Ok(())
}

fn stack_init_array(
    b: &mut MipsBuilder,
    arr_type: &ParamType,
    arr_size: u32,
    init: Option<Located<&InitList>>,
    stack_offset: i32,
) -> ValidationResult<()> {
    let arr_size = arr_size as usize;
    match (arr_type.param_type.data, arr_type.indirection, init) {
        (_, _, None) => {}
        (PrimitiveType::I32 | PrimitiveType::U32, 0, Some(Located { data, .. }))
            if data.len() == arr_size =>
        {
            init_array_const(b, stack_offset, 4, data, |b, addr, val| {
                expect_word(*val).map(|word| {
                    b.const_word(word, Register::T0);
                    b.save_word(Register::T0, addr);
                })
            })?;
        }
        (PrimitiveType::U8, 0, Some(Located { data, .. })) if data.len() == arr_size => {
            init_array_const(b, stack_offset, 1, data, |b, addr, val| {
                expect_byte(*val).map(|byte| {
                    b.const_word(byte as u32, Register::T0);
                    b.save_byte(Register::T0, addr);
                })
            })?;
        }
        (PrimitiveType::U8, 0, Some(Located { data, .. })) if data.len() == arr_size => {
            init_array_const(b, stack_offset, 1, data, |b, addr, val| {
                expect_byte(*val).map(|byte| {
                    b.const_word(byte as u32, Register::T0);
                    b.save_byte(Register::T0, addr);
                })
            })?;
        }
        (PrimitiveType::F32, 0, Some(Located { data, .. })) if data.len() == arr_size => {
            init_array_const(b, stack_offset, 4, data, |b, addr, val| {
                expect_f32(*val).map(|float| {
                    b.const_f32(float, FloatRegister::F4);
                    b.save_f32(FloatRegister::F4, addr);
                })
            })?;
        }
        (PrimitiveType::U8, 1, Some(Located { data, .. })) if data.len() == arr_size => {
            init_array_const(b, stack_offset, 4, data, |b, addr, val| {
                expect_string(*val).map(|str_id| {
                    let str_name = get_str_name(str_id);
                    b.load_addr(Register::T0, MipsAddress::Label(str_name.into()));
                    b.save_word(Register::T0, addr);
                })
            })?;
        }
        (_, _, Some(Located { loc, .. })) => {
            return Err(ValidationError::InvalidLocalInit(loc));
        }
    }
    Ok(())
}
