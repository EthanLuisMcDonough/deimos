use crate::{expect_semicolon, next_expect, next_guard};
use std::iter::Peekable;

use self::tokiter::TokenPeeker;

use super::lexer::*;
use deimos_ast::*;

mod error;
mod expr;
mod macros;
mod tokiter;
pub use error::*;

pub fn parse(Tokens { lexemes, bank }: Tokens) -> ParseResult<Program> {
    let mut tokens = tokiter::TokenPeeker::new(lexemes);
    let mut defs = Definitions::new();

    while let Some(token) = tokens.next() {
        match token.data {
            Lexeme::Keyword(Keyword::Fn) => {
                let fn_name = next_guard!({ tokens.next() } {
                    Lexeme::Identifier(ident) => ident,
                });
                next_expect!({ tokens.next() } {
                    Lexeme::GroupBegin(Grouper::Parenthesis)
                });
                let args = parse_fn_params(&mut tokens)?;
            }
            Lexeme::Keyword(Keyword::Record) => {
                let record_name = next_guard!({ tokens.next() } {
                    Lexeme::Identifier(ident) => ident,
                });
            }
            Lexeme::Keyword(Keyword::Mem) => unimplemented!(),
            Lexeme::Keyword(Keyword::Static) => unimplemented!(),
            _ => return Err(ParseError::UnexpectedToken(token)),
        }
    }

    unimplemented!()
}

fn parse_param_type(tokens: &mut TokenPeeker) -> ParseResult<Located<ParamType>> {
    let mut indirection = 0usize;
    let mut loc = None;

    while let Some(t) = tokens.next_if_eq(&Lexeme::Reference) {
        indirection += 1;
        loc.get_or_insert(t.loc);
    }
    let param_type = next_guard!({ tokens.next() } (loc) {
        Lexeme::Identifier(i) => Located::new(BaseType::Custom(i), loc),
        Lexeme::Primitive(p) => Located::new(BaseType::Primitive(p), loc),
    });
    let loc = loc.unwrap_or(param_type.loc);
    Ok(Located::new(
        ParamType {
            param_type,
            indirection,
        },
        loc,
    ))
}

fn parse_fn_params(tokens: &mut TokenPeeker) -> ParseResult<FunctionArgs> {
    let mut args = Vec::new();
    loop {
        next_guard!({ tokens.next() } (loc) {
            Lexeme::GroupEnd(Grouper::Parenthesis) => break,
            Lexeme::Identifier(ident) => {
                next_expect!({ tokens.next() } { Lexeme::Colon });
                let field_type = parse_param_type(tokens)?;
                args.push(TypedIdent {
                    name: Located::new(ident, loc),
                    field_type
                });
                next_guard!({ tokens.next() } {
                    Lexeme::Comma => {},
                    Lexeme::GroupEnd(Grouper::Parenthesis) => break,
                })
            },
        });
    }
    Ok(args)
}

fn parse_block_until_end(tokens: &mut TokenPeeker) -> ParseResult<Block> {
    let mut block = Block::new();
    loop {
        let token = tokens.next().ok_or(ParseError::UnexpectedEOF)?;
        let stmt = match token.data {
            Lexeme::GroupEnd(Grouper::Brace) => break,
            Lexeme::Keyword(Keyword::Call) => {
                unimplemented!()
            }
            Lexeme::Keyword(Keyword::Syscall) => unimplemented!(),
            Lexeme::Keyword(Keyword::If) => unimplemented!(),
            Lexeme::Keyword(Keyword::Break) => Statement::ControlBreak(ControlBreak::Break),
            Lexeme::Keyword(Keyword::Return) => Statement::ControlBreak(ControlBreak::Return),
            Lexeme::Keyword(Keyword::Continue) => Statement::ControlBreak(ControlBreak::Continue),
            _ => {
                return Err(ParseError::UnexpectedToken(token));
            }
        };
        block.push(Located::new(stmt, token.loc));
        expect_semicolon!(tokens.next());
    }
    Ok(block)
}

fn parse_fn_body(tokens: &mut TokenPeeker) -> ParseResult<FunctionBlock> {
    next_expect!({ tokens.next() } { Lexeme::GroupBegin(Grouper::Brace) });
    unimplemented!()
}
