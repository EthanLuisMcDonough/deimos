use super::{Identifier, Located};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Reg {
    A0,
    A1,
    A2,
    A3,
    V0,
    F0,
    F12,
}

impl Reg {
    pub fn str(&self) -> &'static str {
        match self {
            Self::A0 => "$a0",
            Self::A1 => "$a1",
            Self::A2 => "$a2",
            Self::A3 => "$a3",
            Self::V0 => "$v0",
            Self::F0 => "$f0",
            Self::F12 => "$f12",
        }
    }

    pub fn is_float(&self) -> bool {
        match self {
            Self::F0 | Self::F12 => true,
            _ => false,
        }
    }
}

pub type RegisterMap = HashMap<Reg, Identifier>;

#[derive(Debug, Default)]
pub struct RegVars {
    pub in_values: RegisterMap,
    pub out_values: RegisterMap,
}

#[derive(Debug)]
pub struct Syscall {
    pub syscall_id: Located<usize>,
    pub map: RegVars,
}

#[derive(Debug)]
pub struct AsmBlock {
    pub asm_strings: Vec<Located<usize>>,
    pub map: RegVars,
}
