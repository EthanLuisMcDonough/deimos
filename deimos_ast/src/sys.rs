use super::Identifier;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Register {
    A0,
    A1,
    A2,
    A3,
    V0,
    F0,
    F12,
}

impl Register {
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

#[derive(Debug)]
pub struct Syscall {
    pub syscall_id: usize,
    pub in_values: HashMap<Register, Identifier>,
    pub out_values: HashMap<Register, Identifier>,
}
