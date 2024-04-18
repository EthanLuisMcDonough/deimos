pub trait GenericRegister: std::fmt::Display {
    fn str(&self) -> &'static str;
}

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum Register {
    Zero,
    V0,
    V1,
    A0,
    A1,
    A2,
    A3,
    T0,
    T1,
    T2,
    T3,
    T4,
    T5,
    T6,
    T7,
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    T8,
    T9,
    GlobalPtr,
    StackPtr,
    FramePtr,
    ReturnAddr,
}

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl GenericRegister for Register {
    fn str(&self) -> &'static str {
        match self {
            Self::Zero => "$zero",
            Self::V0 => "$v0",
            Self::V1 => "$v1",
            Self::A0 => "$a0",
            Self::A1 => "$a1",
            Self::A2 => "$a2",
            Self::A3 => "$a3",
            Self::T0 => "$t0",
            Self::T1 => "$t1",
            Self::T2 => "$t2",
            Self::T3 => "$t3",
            Self::T4 => "$t4",
            Self::T5 => "$t5",
            Self::T6 => "$t6",
            Self::T7 => "$t7",
            Self::S0 => "$s0",
            Self::S1 => "$s1",
            Self::S2 => "$s2",
            Self::S3 => "$s3",
            Self::S4 => "$s4",
            Self::S5 => "$s5",
            Self::S6 => "$s6",
            Self::S7 => "$s7",
            Self::T8 => "$t8",
            Self::T9 => "$t9",
            Self::GlobalPtr => "$gp",
            Self::StackPtr => "$sp",
            Self::FramePtr => "$fp",
            Self::ReturnAddr => "$ra",
        }
    }
}

pub enum FloatRegister {
    F0,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    F26,
    F27,
    F28,
    F29,
    F30,
    F31,
}

impl std::fmt::Display for FloatRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.str())
    }
}

impl GenericRegister for FloatRegister {
    fn str(&self) -> &'static str {
        match self {
            Self::F0 => "$f0",
            Self::F1 => "$f1",
            Self::F2 => "$f2",
            Self::F3 => "$f3",
            Self::F4 => "$f4",
            Self::F5 => "$f5",
            Self::F6 => "$f6",
            Self::F7 => "$f7",
            Self::F8 => "$f8",
            Self::F9 => "$f9",
            Self::F10 => "$f10",
            Self::F11 => "$f11",
            Self::F12 => "$f12",
            Self::F13 => "$f13",
            Self::F14 => "$f14",
            Self::F15 => "$f15",
            Self::F16 => "$f16",
            Self::F17 => "$f17",
            Self::F18 => "$f18",
            Self::F19 => "$f19",
            Self::F20 => "$f20",
            Self::F21 => "$f21",
            Self::F22 => "$f22",
            Self::F23 => "$f23",
            Self::F24 => "$f24",
            Self::F25 => "$f25",
            Self::F26 => "$f26",
            Self::F27 => "$f27",
            Self::F28 => "$f28",
            Self::F29 => "$f29",
            Self::F30 => "$f30",
            Self::F31 => "$f31",
        }
    }
}
