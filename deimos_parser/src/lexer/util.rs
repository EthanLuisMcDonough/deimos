use deimos_ast::StringBank;
use std::collections::HashMap;

pub fn is_varchar(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn is_powsign(c: char) -> bool {
    c == '-' || c == '+'
}

pub fn is_regchar(c: char) -> bool {
    match c {
        '0'..='3' | 'a' | 'v' | 'f' => true,
        _ => false,
    }
}

#[derive(Default, Debug)]
pub struct TempStringBank {
    string_bank: HashMap<String, usize>,
    ident_bank: HashMap<String, usize>,
}

impl TempStringBank {
    pub fn get_ident(&mut self, s: String) -> usize {
        let new_index = self.ident_bank.len();
        *self.ident_bank.entry(s).or_insert(new_index)
    }

    pub fn get_string(&mut self, s: String) -> usize {
        let new_index = self.string_bank.len();
        *self.string_bank.entry(s).or_insert(new_index)
    }
}

impl From<TempStringBank> for StringBank {
    fn from(value: TempStringBank) -> Self {
        let ident_count = value.ident_bank.len();
        let mut identifiers = vec![String::new(); ident_count];
        for (s, index) in value.ident_bank {
            identifiers[index] = s;
        }

        let string_count = value.string_bank.len();
        let mut strings = vec![String::new(); string_count];
        for (s, index) in value.string_bank {
            strings[index] = s;
        }

        StringBank {
            identifiers,
            strings,
        }
    }
}
