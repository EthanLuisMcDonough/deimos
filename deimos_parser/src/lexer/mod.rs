use deimos_ast::{Located, StringBank};

mod chiter;
mod error;
mod macros;
mod tokens;
mod util;

pub use error::*;
pub use tokens::*;

#[derive(Default, Debug)]
pub struct Tokens {
    pub lexemes: Vec<Located<Lexeme>>,
    pub bank: StringBank,
}

pub fn lex(s: &str) -> LexResult<Tokens> {
    let mut chars = chiter::ChIter::new(&s);
    let mut bank = util::TempStringBank::default();
    let mut lexemes = Vec::new();

    while let Some(c) = chars.next() {
        let lexeme_loc = chars.get_loc();
        let lexeme = match c {
            // Parse identifier
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::from(c);
                while let Some(c) = chars.next_if(util::is_varchar) {
                    ident.push(c);
                }

                if let Some(k) = Keyword::from_str(&ident) {
                    Lexeme::Keyword(k)
                } else if let Some(p) = test_primitive(&ident) {
                    Lexeme::Primitive(p)
                } else {
                    Lexeme::Identifier(bank.get_ident(ident))
                }
            }

            // Parse number
            '0'..='9' => {
                let mut num_buf = String::new();
                // Parse hex integer
                if c == '0' && chars.next_if_eq('x') {
                    while let Some(c) = chars.next_if(|c| c.is_ascii_hexdigit()) {
                        num_buf.push(c);
                    }
                    if chars.next_if_eq('u') {
                        u32::from_str_radix(&num_buf, 16)
                            .map(Lexeme::Unsigned)
                            .map_err(|_| LexErrorKind::InvalidNumber.with_loc(lexeme_loc))?
                    } else {
                        i32::from_str_radix(&num_buf, 16)
                            .map(Lexeme::Integer)
                            .map_err(|_| LexErrorKind::InvalidNumber.with_loc(lexeme_loc))?
                    }
                } else {
                    // Parse decimal number
                    // We know its a float if we find ., e, or f
                    let mut is_float = false;

                    num_buf.push(c);
                    // Parse integer part
                    while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
                        num_buf.push(c);
                    }

                    // Parse rational part
                    if chars.next_if_eq('.') {
                        is_float = true;
                        num_buf.push('.');
                        while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
                            num_buf.push(c);
                        }
                    }

                    // Parse exponent part
                    if chars.next_if_eq('e') {
                        is_float = true;
                        let mut any_after_e = false;
                        num_buf.push('e');
                        if let Some(c) = chars.next_if(util::is_powsign) {
                            num_buf.push(c);
                        }
                        while let Some(c) = chars.next_if(|c| c.is_ascii_digit()) {
                            any_after_e = true;
                            num_buf.push(c);
                        }
                        // There needs to be something after e, otherwise its a syntax error
                        if !any_after_e {
                            return Err(chars.expect_any_err());
                        }
                    }

                    if chars.next_if_eq('f') {
                        is_float = true;
                    }

                    // Convert string to numeric value
                    if is_float {
                        num_buf
                            .parse::<f32>()
                            .map(Lexeme::Float)
                            .map_err(|_| LexErrorKind::InvalidNumber.with_loc(lexeme_loc))?
                    } else if chars.next_if_eq('u') {
                        u32::from_str_radix(&num_buf, 16)
                            .map(Lexeme::Unsigned)
                            .map_err(|_| LexErrorKind::InvalidNumber.with_loc(lexeme_loc))?
                    } else {
                        i32::from_str_radix(&num_buf, 10)
                            .map(Lexeme::Integer)
                            .map_err(|_| LexErrorKind::InvalidNumber.with_loc(lexeme_loc))?
                    }
                }
            }

            // Parse operators
            '+' => Lexeme::Plus,
            '-' => Lexeme::Minus,
            '*' => Lexeme::Multiply,
            '/' => Lexeme::Divide,
            '%' => Lexeme::Modulo,
            '&' => Lexeme::Reference,
            '@' => Lexeme::Deref,

            // Parse comments
            '#' => {
                chars.pass_over(|c| c != '\n');
                continue;
            }

            // Parse N or Neq operators
            '>' | '=' | '<' | '!' => {
                let next_eq = chars.next_if_eq('=');
                match c {
                    '>' if next_eq => Lexeme::GreaterThanEq,
                    '=' if next_eq => Lexeme::LogicEq,
                    '<' if next_eq => Lexeme::LessThanEq,
                    '!' if next_eq => Lexeme::LogicNotEq,
                    '>' => Lexeme::GreaterThan,
                    '=' => Lexeme::Equals,
                    '<' => Lexeme::LessThan,
                    '!' => Lexeme::LogicNot,
                    _ => unreachable!(),
                }
            }

            // Parse register name
            '$' => {
                let mut reg = String::new();
                while let Some(c) = chars.next_if(util::is_regchar) {
                    reg.push(c);
                }
                test_register(&reg)
                    .map(Lexeme::Register)
                    .ok_or(LexErrorKind::InvalidRegister.with_loc(lexeme_loc))?
            }

            // Parse string
            '"' => {
                let mut s = String::new();
                loop {
                    match chars.expect_any()? {
                        '"' => break,
                        '\\' => {
                            s.push('\\');
                            s.push(chars.expect_any()?);
                        }
                        c => s.push(c),
                    }
                }
                Lexeme::String(bank.get_string(s))
            }

            // Parse groupers [], (), and {}
            '(' => Lexeme::GroupBegin(Grouper::Parenthesis),
            ')' => Lexeme::GroupEnd(Grouper::Parenthesis),
            '{' => Lexeme::GroupBegin(Grouper::Brace),
            '}' => Lexeme::GroupEnd(Grouper::Brace),
            '[' => Lexeme::GroupBegin(Grouper::Bracket),
            ']' => Lexeme::GroupEnd(Grouper::Bracket),

            // Parse delimiters
            ':' => Lexeme::Colon,
            ',' => Lexeme::Comma,
            ';' => Lexeme::Semicolon,
            '.' => Lexeme::Peroid,

            // Skip whitespace
            ' ' | '\n' | '\t' | '\r' => continue,
            _ => return Err(LexErrorKind::UnexpectedChar(c).with_loc(lexeme_loc)),
        };

        lexemes.push(Located {
            data: lexeme,
            loc: lexeme_loc,
        });
    }
    Ok(Tokens {
        lexemes,
        bank: bank.into(),
    })
}
