/// Module responsible for codegen for static expressions (local variable and
/// static var declaration)
use crate::error::*;
use crate::names::{get_static_name, get_str_name};
use deimos_ast::*;
use mips_builder::*;

/// Create directive for scalar static variable
fn init_static_param(
    bank: &StringBank,
    name: Identifier,
    param_type: &ParamType,
    init_val: &Option<InitValue>,
) -> ValidationResult<DataDirective> {
    Ok(
        match (param_type.param_type.data, param_type.indirection, init_val) {
            (PrimitiveType::F32, 0, None) => DataDirective::from(0.0),
            (PrimitiveType::F32, 0, Some(InitValue::Primitive(PrimitiveValue::Float(f)))) => {
                DataDirective::from(f.data)
            }
            (PrimitiveType::I32 | PrimitiveType::U32, 0, None) => DataDirective::from(0i32),
            (PrimitiveType::U8, 0, None) => DataDirective::from(0u8),
            (
                PrimitiveType::I32 | PrimitiveType::U32,
                0,
                Some(InitValue::Primitive(PrimitiveValue::Int(i))),
            ) => DataDirective::from(i.data),
            (
                PrimitiveType::I32 | PrimitiveType::U32,
                0,
                Some(InitValue::Primitive(PrimitiveValue::Unsigned(i))),
            ) => DataDirective::from(i.data),
            (PrimitiveType::U8, 0, Some(InitValue::Primitive(PrimitiveValue::Unsigned(i))))
                if i.data < 256 =>
            {
                DataDirective::from(i.data as u8)
            }
            (PrimitiveType::U8, 0, Some(InitValue::Primitive(PrimitiveValue::Int(i))))
                if i.data >= 0 && i.data < 256 =>
            {
                DataDirective::from(i.data as u8)
            }
            (PrimitiveType::U8, 1, Some(InitValue::Primitive(PrimitiveValue::String(s)))) => {
                DataDirective::Asciiz(bank.strings[s.data].clone())
            }
            _ => {
                return Err(ValidationError::new(
                    ValidationErrorKind::InvalidStaticVar,
                    name.loc,
                ));
            }
        },
    )
}

fn expect_f32(val: &PrimitiveValue) -> ValidationResult<f32> {
    Ok(match val {
        PrimitiveValue::Float(f) => f.data,
        PrimitiveValue::Int(i) => i.data as f32,
        PrimitiveValue::Unsigned(i) => i.data as f32,
        PrimitiveValue::String(s) => {
            return Err(ValidationError::new(
                ValidationErrorKind::MismatchedType,
                s.loc,
            ));
        }
    })
}

fn expect_word(val: &PrimitiveValue) -> ValidationResult<u32> {
    match val {
        PrimitiveValue::Int(i) => Ok(i.data as u32),
        PrimitiveValue::Unsigned(i) => Ok(i.data),
        PrimitiveValue::String(Located { data: _, loc })
        | PrimitiveValue::Float(Located { data: _, loc }) => Err(ValidationError::new(
            ValidationErrorKind::MismatchedType,
            *loc,
        )),
    }
}

fn expect_byte(val: &PrimitiveValue) -> ValidationResult<u8> {
    match val {
        PrimitiveValue::Int(i) if i.data >= 0 && i.data < 256 => Ok(i.data as u8),
        PrimitiveValue::Unsigned(i) if i.data < 256 => Ok(i.data as u8),
        PrimitiveValue::String(Located { loc, .. })
        | PrimitiveValue::Float(Located { loc, .. })
        | PrimitiveValue::Int(Located { loc, .. })
        | PrimitiveValue::Unsigned(Located { loc, .. }) => Err(ValidationError::new(
            ValidationErrorKind::MismatchedType,
            *loc,
        )),
    }
}

fn expect_string(val: &PrimitiveValue) -> ValidationResult<usize> {
    match val {
        PrimitiveValue::String(s) => Ok(s.data),
        PrimitiveValue::Float(Located { loc, .. })
        | PrimitiveValue::Int(Located { loc, .. })
        | PrimitiveValue::Unsigned(Located { loc, .. }) => Err(ValidationError::new(
            ValidationErrorKind::MismatchedType,
            *loc,
        )),
    }
}

/// Create directive for array static variable
fn init_static_array(
    bank: &StringBank,
    name: Identifier,
    array_type: &ParamType,
    array_size: u32,
    init_val: &Option<InitValue>,
) -> ValidationResult<DataDirective> {
    let array_size = array_size as usize;
    Ok(
        match (array_type.param_type.data, array_type.indirection, init_val) {
            (PrimitiveType::U8, 0, None) => DataDirective::ByteLen {
                len: array_size,
                default: 0,
            },
            (PrimitiveType::U32 | PrimitiveType::I32 | PrimitiveType::F32, 0, None)
            | (_, 1.., None) => DataDirective::WordLen {
                len: array_size,
                default: 0,
            },
            (PrimitiveType::F32, 0, Some(InitValue::List(list))) if list.len() == array_size => {
                let mut float_vals = Vec::with_capacity(list.len());
                for val in list {
                    float_vals.push(expect_f32(val)?);
                }
                DataDirective::from(float_vals)
            }
            (PrimitiveType::I32 | PrimitiveType::U32, 0, Some(InitValue::List(list)))
                if list.len() == array_size =>
            {
                let mut int_vals = Vec::with_capacity(list.len());
                for val in list {
                    int_vals.push(expect_word(val)?);
                }
                DataDirective::from(int_vals)
            }
            (PrimitiveType::U8, 0, Some(InitValue::List(list))) if list.len() == array_size => {
                let mut byte_vals = Vec::with_capacity(list.len());
                for val in list {
                    byte_vals.push(expect_byte(val)?);
                }
                DataDirective::from(byte_vals)
            }
            (PrimitiveType::U8, 0, Some(InitValue::Primitive(PrimitiveValue::String(s))))
                if bank.strings[s.data].len() + 1 == array_size =>
            {
                DataDirective::Asciiz(bank.strings[s.data].clone())
            }
            _ => {
                return Err(ValidationError::new(
                    ValidationErrorKind::InvalidStaticVar,
                    name.loc,
                ));
            }
        },
    )
}

/// Codegen for creating static variables (variables stored in .data section)
pub fn codegen_init_static(
    b: &mut MipsBuilder,
    bank: &StringBank,
    static_var: &VarDecl,
) -> ValidationResult<()> {
    let mut static_def = DataDef::new(get_static_name(static_var.name.data));
    let directive = match &static_var.variable {
        DeclType::Param(p) => init_static_param(bank, static_var.name, &p.data, &static_var.init)?,
        DeclType::Array { array_type, size } => init_static_array(
            bank,
            static_var.name,
            &array_type.data,
            size.data,
            &static_var.init,
        )?,
    };
    static_def.add_dir(directive);
    b.add_def(static_def);
    Ok(())
}

pub fn codegen_init_var(
    b: &mut MipsBuilder,
    name: Identifier,
    bank: &StringBank,
    var_type: DeclType,
    init: &Option<InitValue>,
    stack_offset: i32,
) -> ValidationResult<()> {
    match (var_type, init) {
        (DeclType::Param(p), Some(InitValue::Primitive(init))) => {
            stack_init_param(b, &p.data, Some(init), stack_offset)
        }
        (DeclType::Param(p), None) => stack_init_param(b, &p.data, None, stack_offset),
        (DeclType::Array { array_type, size }, Some(InitValue::List(list))) => {
            stack_init_array(b, &array_type.data, size.data, Some(list), stack_offset)
        }
        (DeclType::Array { array_type, size }, None) => {
            stack_init_array(b, &array_type.data, size.data, None, stack_offset)
        }
        _ => Err(ValidationError::new(
            ValidationErrorKind::InvalidLocalInit,
            name.loc,
        )),
    }
}

fn stack_init_param(
    b: &mut MipsBuilder,
    var_type: &ParamType,
    init: Option<&PrimitiveValue>,
    stack_offset: i32,
) -> ValidationResult<()> {
    let address = MipsAddress::RegisterOffset {
        register: Register::StackPtr,
        offset: stack_offset,
    };
    match var_type.param_type.data {
        PrimitiveType::F32 if var_type.indirection == 0 => {
            let val = init.map(expect_f32).transpose()?.unwrap_or_default();
            b.const_f32(val, FloatRegister::F4);
            b.save_f32(FloatRegister::F4, address);
        }
        PrimitiveType::I32 | PrimitiveType::U32 if var_type.indirection == 0 => {
            let val = init.map(expect_word).transpose()?.unwrap_or_default();
            b.const_word(val, Register::T0);
            b.save_word(Register::T0, address);
        }
        PrimitiveType::U8 if var_type.indirection == 0 => {
            let val = init.map(expect_byte).transpose()?.unwrap_or_default();
            b.const_word(val as u32, Register::T0);
            b.save_byte(Register::T0, address);
        }
        PrimitiveType::U8 if var_type.indirection == 1 => {
            if let Some(str_id) = init.map(expect_string).transpose()? {
                let str_name = get_str_name(str_id);
                b.load_addr(Register::T0, MipsAddress::Label(str_name.into()));
                b.save_word(Register::T0, address);
            } else {
                b.save_word(Register::Zero, address);
            }
        }
        _ => {
            /*if let Some(t) = init {
                return Err(ValidationError::new(
                    ValidationErrorKind::InvalidLocalInit,
                    name.loc,
                ));
            }*/
            unimplemented!()
        },//b.save_word(Register::Zero, address),
    };
    Ok(())
}

fn stack_init_array(
    b: &mut MipsBuilder,
    arr_type: &ParamType,
    arr_size: u32,
    init: Option<&Vec<PrimitiveValue>>,
    stack_offset: i32,
) -> ValidationResult<()> {
    Ok(())
}
