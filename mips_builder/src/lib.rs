use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;

mod registers;
pub use registers::*;

const FMT_ITEMS_PER_LINE: usize = 10;
const WORD_CONSTS_LBL: &'static str = "WORD_CONST";

pub struct MipsBlock {
    label: String,
    instructions: Vec<String>,
}

impl MipsBlock {
    fn append(&self, s: &mut String) {
        s.push_str(&self.label);
        s.push_str(":\n");
        for instr in &self.instructions {
            s.push('\t');
            s.push_str(instr);
            s.push('\n');
        }
    }
}

pub enum MipsAddress<'a> {
    Register(Register),
    Label(Cow<'a, str>),
    RegisterOffset {
        register: Register,
        offset: i32,
    },
    RegisterLabel {
        register: Register,
        label: Cow<'a, str>,
    },
    LabelOffset {
        label: Cow<'a, str>,
        offset: i32,
    },
    Full {
        label: Cow<'a, str>,
        offset: i32,
        register: Register,
    },
    Addr(u32),
    RegisterAddr {
        register: Register,
        addr: u32,
    },
}

/// Add or subtract to mips address offset
impl<'a> MipsAddress<'a> {
    pub fn add(self, offset: i32) -> Self {
        match self {
            Self::Addr(u) => Self::Addr(u.wrapping_add_signed(offset)),
            Self::RegisterAddr { register, addr } => Self::RegisterAddr {
                register,
                addr: addr.wrapping_add_signed(offset),
            },
            Self::Register(register) => Self::RegisterOffset { register, offset },
            Self::RegisterOffset {
                register,
                offset: o,
            } => Self::RegisterOffset {
                register,
                offset: o + offset,
            },
            Self::Label(label) => Self::LabelOffset { label, offset },
            Self::LabelOffset { label, offset: o } => Self::LabelOffset {
                label,
                offset: o + offset,
            },
            Self::RegisterLabel { register, label } => Self::Full {
                label,
                offset,
                register,
            },
            Self::Full {
                label,
                offset: o,
                register,
            } => Self::Full {
                label,
                offset: o + offset,
                register,
            },
        }
    }
}

impl<'a> From<&'a str> for MipsAddress<'a> {
    fn from(value: &'a str) -> Self {
        Self::Label(Cow::Borrowed(value))
    }
}

impl<'a> From<String> for MipsAddress<'a> {
    fn from(value: String) -> Self {
        Self::Label(Cow::Owned(value))
    }
}

impl<'a> From<Register> for MipsAddress<'a> {
    fn from(value: Register) -> Self {
        Self::Register(value)
    }
}

impl std::fmt::Display for MipsAddress<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Register(r) => write!(f, "({})", r),
            Self::Label(l) => write!(f, "{}", l),
            Self::RegisterOffset { register, offset } => write!(f, "{}({})", offset, register),
            Self::RegisterLabel { register, label } => write!(f, "{}({})", label, register),
            Self::LabelOffset { label, offset } => write!(f, "{}+{}", label, offset),
            Self::Full {
                label,
                offset,
                register,
            } => write!(f, "{}+{}({})", label, offset, register),
            Self::Addr(a) => write!(f, "{:#010X}", a),
            Self::RegisterAddr { register, addr } => write!(f, "{:#010X}({})", addr, register),
        }
    }
}

fn write_group_directive<T: Display>(s: &mut String, directive: &str, i: &[T]) {
    for line in i.chunks(FMT_ITEMS_PER_LINE) {
        s.push_str(directive);
        for item in line {
            s.push(' ');
            s.push_str(&item.to_string());
        }
        s.push('\n');
    }
}

fn write_len_directive(s: &mut String, directive: &str, len: usize, default: impl Display) {
    s.push_str(directive);
    s.push(' ');
    s.push_str(&default.to_string());
    s.push_str(" : ");
    s.push_str(&len.to_string());
    s.push('\n');
}

pub enum DataDirective {
    Word(Vec<u32>),
    WordLen { len: usize, default: u32 },
    Asciiz(String),
    Byte(Vec<u8>),
    ByteLen { len: usize, default: u8 },
    Float(Vec<f32>),
    FloatLen { len: usize, default: f32 },
}

impl From<Vec<u32>> for DataDirective {
    fn from(value: Vec<u32>) -> Self {
        Self::Word(value)
    }
}

impl From<u32> for DataDirective {
    fn from(value: u32) -> Self {
        Self::WordLen {
            len: 1,
            default: value,
        }
    }
}

impl From<i32> for DataDirective {
    fn from(value: i32) -> Self {
        Self::WordLen {
            len: 1,
            default: value as u32,
        }
    }
}

impl From<Vec<u8>> for DataDirective {
    fn from(value: Vec<u8>) -> Self {
        Self::Byte(value)
    }
}

impl From<u8> for DataDirective {
    fn from(value: u8) -> Self {
        Self::ByteLen {
            len: 1,
            default: value,
        }
    }
}

impl From<f32> for DataDirective {
    fn from(value: f32) -> Self {
        Self::FloatLen {
            len: 1,
            default: value,
        }
    }
}

impl From<Vec<f32>> for DataDirective {
    fn from(value: Vec<f32>) -> Self {
        Self::Float(value)
    }
}

impl From<String> for DataDirective {
    fn from(value: String) -> Self {
        Self::Asciiz(value)
    }
}

impl DataDirective {
    fn append(&self, s: &mut String) {
        match self {
            Self::Word(u) => write_group_directive(s, ".word", u),
            Self::WordLen { len, default } => write_len_directive(s, ".word", *len, default),
            Self::Asciiz(txt) => {
                s.push_str(".asciiz ");
                s.push_str(txt);
                s.push('\n');
            }
            Self::Byte(b) => write_group_directive(s, ".byte", b),
            Self::ByteLen { len, default } => write_len_directive(s, ".byte", *len, default),
            Self::Float(f) => write_group_directive(s, ".float", f),
            Self::FloatLen { len, default } => write_len_directive(s, ".float", *len, default),
        }
    }
}

pub struct DataDef {
    name: Cow<'static, str>,
    vals: Vec<DataDirective>,
}

impl DataDef {
    pub fn new(name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            name: name.into(),
            vals: Vec::new(),
        }
    }

    pub fn add_dir(&mut self, dir: impl Into<DataDirective>) {
        self.vals.push(dir.into());
    }

    fn append(&self, s: &mut String) {
        s.push_str(&self.name);
        s.push_str(": ");
        for dir in &self.vals {
            dir.append(s);
        }
    }
}

#[derive(Default)]
pub struct MipsBuilder {
    word_const_ind: HashMap<u32, usize>,
    word_consts: Vec<u32>,
    data_vars: Vec<DataDef>,
    blocks: Vec<MipsBlock>,
}

impl MipsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn new_block(&mut self, blockname: impl Into<Cow<'static, str>>) {
        let label = blockname.into().into_owned();
        self.blocks.push(MipsBlock {
            label,
            instructions: Vec::new(),
        });
    }

    fn instr(&mut self, text: String) {
        if let Some(block) = self.blocks.last_mut() {
            block.instructions.push(text);
        }
    }
    fn instr2(&mut self, t: &str, a: impl Into<GenericRegister>, b: impl Into<GenericRegister>) {
        self.instr(format!("{} {}, {}", t, a.into(), b.into()));
    }
    fn instr2_const(
        &mut self,
        t: &str,
        a: impl Into<GenericRegister>,
        b: impl Into<GenericRegister>,
        val: impl Display,
    ) {
        self.instr(format!("{} {}, {}, {}", t, a.into(), b.into(), val));
    }
    fn instr3(
        &mut self,
        t: &str,
        a: impl Into<GenericRegister>,
        b: impl Into<GenericRegister>,
        c: impl Into<GenericRegister>,
    ) {
        self.instr(format!("{} {}, {}, {}", t, a.into(), b.into(), c.into()));
    }
    fn addr_instr(&mut self, t: &str, reg: impl Into<GenericRegister>, loc: MipsAddress) {
        self.instr(format!("{} {}, {}", t, reg.into(), loc));
    }
    fn branch_instr(&mut self, instr: &str, reg1: Register, reg2: Register, label: &str) {
        self.instr(format!("{} {}, {}, {}", instr, reg1, reg2, label));
    }

    pub fn mov(&mut self, dest: Register, source: Register) {
        self.instr2("move", dest, source);
    }
    pub fn move_from_hi(&mut self, dest: Register) {
        self.instr(format!("mfhi {}", dest));
    }
    pub fn move_from_lo(&mut self, dest: Register) {
        self.instr(format!("mflo {}", dest));
    }

    pub fn load_word<'a>(&mut self, dest: Register, loc: impl Into<MipsAddress<'a>>) {
        self.addr_instr("lw", dest, loc.into());
    }
    pub fn save_word<'a>(&mut self, source: Register, loc: impl Into<MipsAddress<'a>>) {
        self.addr_instr("sw", source, loc.into());
    }
    pub fn load_byte<'a>(&mut self, dest: Register, loc: impl Into<MipsAddress<'a>>) {
        self.addr_instr("lb", dest, loc.into());
    }
    pub fn save_byte<'a>(&mut self, source: Register, loc: impl Into<MipsAddress<'a>>) {
        self.addr_instr("sb", source, loc.into());
    }
    pub fn load_f32<'a>(&mut self, dest: FloatRegister, loc: impl Into<MipsAddress<'a>>) {
        self.addr_instr("l.s", dest, loc.into());
    }
    pub fn save_f32<'a>(&mut self, source: FloatRegister, loc: impl Into<MipsAddress<'a>>) {
        self.addr_instr("s.s", source, loc.into());
    }

    pub fn load_addr<'a>(&mut self, dest: Register, addr: impl Into<MipsAddress<'a>>) {
        self.instr(format!("la {}, {}", dest, addr.into()));
    }

    fn ins_word(&mut self, val: u32) -> usize {
        let word_count = self.word_const_ind.len();
        self.word_consts.push(val);
        *self.word_const_ind.entry(val).or_insert(word_count)
    }
    pub fn const_word(&mut self, val: u32, dest: Register) {
        if val == 0 {
            self.mov(dest, Register::Zero);
        } else {
            self.instr(format!("li {}, {}", dest, val));
        }
    }
    pub fn const_f32(&mut self, val: f32, dest: FloatRegister) {
        let index = self.ins_word(val.to_bits()) as i32;
        self.load_f32(
            dest,
            MipsAddress::LabelOffset {
                label: WORD_CONSTS_LBL.into(),
                offset: index * 4,
            },
        );
    }

    pub fn add_i32(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("add", dest, source1, source2);
    }
    pub fn add_u32(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("addu", dest, source1, source2);
    }
    pub fn sub_i32(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("sub", dest, source1, source2);
    }
    pub fn sub_u32(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("subu", dest, source1, source2);
    }
    pub fn mul_i32(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("mul", dest, source1, source2);
    }
    pub fn div_i32(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr2("div", source1, source2);
        self.move_from_lo(dest);
    }
    pub fn mod_i32(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr2("div", source1, source2);
        self.move_from_hi(dest);
    }

    pub fn shift_logical_left(&mut self, dest: Register, source: Register, value: u32) {
        self.instr2_const("sll", dest, source, value);
    }
    pub fn add_const_i32(&mut self, dest: Register, source: Register, val: i32) {
        self.instr2_const("addi", dest, source, val);
    }

    pub fn set_eq(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("seq", dest, source1, source2);
    }
    pub fn set_neq(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("sne", dest, source1, source2);
    }
    pub fn set_gt(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("sgt", dest, source1, source2);
    }
    pub fn set_ge(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("sge", dest, source1, source2);
    }
    pub fn set_lt(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("slt", dest, source1, source2);
    }
    pub fn set_le(&mut self, dest: Register, source1: Register, source2: Register) {
        self.instr3("sle", dest, source1, source2);
    }

    pub fn jump_register(&mut self, dest: Register) {
        self.instr(format!("jr {}", dest));
    }
    pub fn jump_and_link(&mut self, fn_name: &str) {
        self.instr(format!("jal {}", fn_name));
    }

    pub fn branch_eq(&mut self, reg1: Register, reg2: Register, lbl: &str) {
        self.branch_instr("beq", reg1, reg2, lbl);
    }
    pub fn branch_not_eq(&mut self, reg1: Register, reg2: Register, lbl: &str) {
        self.branch_instr("bne", reg1, reg2, lbl);
    }
    pub fn branch_eq_zero(&mut self, reg1: Register, lbl: &str) {
        self.branch_eq(reg1, Register::Zero, lbl);
    }
    pub fn branch_not_eq_zero(&mut self, reg1: Register, lbl: &str) {
        self.branch_not_eq(reg1, Register::Zero, lbl);
    }
    pub fn branch(&mut self, lbl: &str) {
        self.instr(format!("b {}", lbl));
    }

    pub fn branch_float_true(&mut self, lbl: &str) {
        self.instr(format!("bc1t {}", lbl));
    }
    pub fn branch_float_false(&mut self, lbl: &str) {
        self.instr(format!("bc1f {}", lbl));
    }

    pub fn mov_f32(&mut self, dest: FloatRegister, source: FloatRegister) {
        self.instr2("mov.s", dest, source);
    }
    pub fn mov_from_f32(&mut self, dest: Register, source: FloatRegister) {
        self.instr2("mfc1", dest, source);
    }
    pub fn mov_to_f32(&mut self, dest: FloatRegister, source: Register) {
        self.instr2("mtc1", source, dest);
    }
    pub fn cast_to_f32(&mut self, int: FloatRegister, float: FloatRegister) {
        self.instr2("cvt.w.s", int, float);
    }
    pub fn cast_from_f32(&mut self, float: FloatRegister, int: FloatRegister) {
        self.instr2("cvt.s.w", float, int);
    }

    pub fn add_f32(&mut self, dest: FloatRegister, f1: FloatRegister, f2: FloatRegister) {
        self.instr3("add.s", dest, f1, f2);
    }
    pub fn sub_f32(&mut self, dest: FloatRegister, f1: FloatRegister, f2: FloatRegister) {
        self.instr3("sub.s", dest, f1, f2);
    }
    pub fn mul_f32(&mut self, dest: FloatRegister, f1: FloatRegister, f2: FloatRegister) {
        self.instr3("mul.s", dest, f1, f2);
    }
    pub fn div_f32(&mut self, dest: FloatRegister, f1: FloatRegister, f2: FloatRegister) {
        self.instr3("div.s", dest, f1, f2);
    }
    pub fn neg_f32(&mut self, dest: FloatRegister, val: FloatRegister) {
        self.instr2("neg.s", dest, val);
    }

    pub fn equals_f32(&mut self, val1: FloatRegister, val2: FloatRegister) {
        self.instr2("c.eq.s", val1, val2);
    }
    pub fn less_than_or_eq_f32(&mut self, val1: FloatRegister, val2: FloatRegister) {
        self.instr2("c.le.s", val1, val2);
    }
    pub fn less_than_f32(&mut self, val1: FloatRegister, val2: FloatRegister) {
        self.instr2("c.lt.s", val1, val2);
    }

    pub fn add_def(&mut self, d: DataDef) {
        self.data_vars.push(d);
    }
    pub fn add_syscall(&mut self, id: u8) {
        self.const_word(id as u32, Register::V0);
        self.instr("syscall".to_string());
    }

    pub fn codegen(self) -> String {
        let mut buf = String::new();
        buf.push_str("\t.data\n");

        // Write word constants
        if !self.word_consts.is_empty() {
            let mut word_bank = DataDef::new(WORD_CONSTS_LBL);
            word_bank.add_dir(self.word_consts);
            word_bank.append(&mut buf);
        }

        // Write data defs
        for val in self.data_vars {
            val.append(&mut buf);
        }

        buf.push_str("\n\t.text\n");

        // Write instructions
        for block in self.blocks {
            block.append(&mut buf);
        }

        buf
    }
}
