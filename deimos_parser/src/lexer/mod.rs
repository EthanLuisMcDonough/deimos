use deimos_ast::StringBank;

mod chiter;
mod error;
mod macros;
mod tokens;
mod util;

pub use error::*;
pub use tokens::*;

#[derive(Default, Debug)]
pub struct Tokens {
    pub lexemes: Vec<Lexeme>,
    pub bank: StringBank,
}

pub fn lex(s: &str) -> LexResult<Tokens> {
    let mut chars = chiter::ChIter::new(&s);
    let mut bank = util::TempStringBank::default();
    let mut lexemes = Vec::new();

    while let Some(c) = chars.next() {
        let token_loc = chars.get_loc();
        let token = match c {
            // Parse identifier
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::from(c);
                while let Some(c) = chars.next_if(util::is_varchar) {
                    ident.push(c);
                }

                if let Some(k) = Keyword::from_str(&ident) {
                    Token::Keyword(k)
                } else if let Some(p) = PrimitiveType::from_str(&ident) {
                    Token::Primitive(p)
                } else {
                    Token::Identifier(bank.get_ident(ident))
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
                    i32::from_str_radix(&num_buf, 16)
                        .map(Token::Integer)
                        .map_err(|_| LexErrorKind::InvalidNumber.with_loc(token_loc))?
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
                            .map(Token::Float)
                            .map_err(|_| LexErrorKind::InvalidNumber.with_loc(token_loc))?
                    } else {
                        i32::from_str_radix(&num_buf, 10)
                            .map(Token::Integer)
                            .map_err(|_| LexErrorKind::InvalidNumber.with_loc(token_loc))?
                    }
                }
            }

            // Parse operators
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Multiply,
            '/' => Token::Divide,
            '%' => Token::Modulo,
            '&' => Token::Reference,
            '@' => Token::Deref,

            // Parse comments
            '#' => {
                chars.pass_over(|c| c != '\n');
                continue;
            }

            // Parse N or Neq operators
            '>' | '=' | '<' | '!' => {
                let next_eq = chars.next_if_eq('=');
                match c {
                    '>' if next_eq => Token::GreaterThanEq,
                    '=' if next_eq => Token::LogicEq,
                    '<' if next_eq => Token::LessThanEq,
                    '!' if next_eq => Token::LogicNotEq,
                    '>' => Token::GreaterThan,
                    '=' => Token::Equals,
                    '<' => Token::LessThan,
                    '!' => Token::LogicNot,
                    _ => unreachable!(),
                }
            }

            /*'$' => {}*/
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
                Token::String(bank.get_string(s))
            }

            // Parse groupers [], (), and {}
            '(' => Token::GrouperBegin(Grouper::Parenthesis),
            ')' => Token::GrouperEnd(Grouper::Parenthesis),
            '{' => Token::GrouperBegin(Grouper::Brace),
            '}' => Token::GrouperEnd(Grouper::Brace),
            '[' => Token::GrouperBegin(Grouper::Bracket),
            ']' => Token::GrouperEnd(Grouper::Bracket),

            // Parse delimiters
            ':' => Token::Colon,
            ',' => Token::Comma,
            ';' => Token::Semicolon,
            '.' => Token::Peroid,

            // Skip whitespace
            ' ' | '\n' | '\t' | '\r' => continue,
            _ => return Err(LexErrorKind::UnexpectedChar(c).with_loc(token_loc)),
        };

        lexemes.push(Lexeme {
            token,
            loc: token_loc,
        });
    }
    Ok(Tokens {
        lexemes,
        bank: bank.into(),
    })
}
