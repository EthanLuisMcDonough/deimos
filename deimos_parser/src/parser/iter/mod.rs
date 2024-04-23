use super::{Grouper, ParseError, ParseResult};
use crate::lexer::{Keyword, Lexeme};
use deimos_ast::{Identifier, Located, Location};
use std::collections::VecDeque;

pub mod macros;

/// Iterator wrapper over vector of lexemes designed to logically
/// handle nested structures
#[derive(Clone)]
pub struct TokenIter<'a> {
    pub tokens: &'a Vec<Located<Lexeme>>,
    pub index: usize,
    end: usize,
}

impl<'a> Iterator for TokenIter<'a> {
    type Item = Located<Lexeme>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.end {
            None
        } else {
            let ind = self.index;
            self.index += 1;
            self.tokens.get(ind).cloned()
        }
    }
}

impl<'a> TokenIter<'a> {
    pub fn new(tokens: &'a Vec<Located<Lexeme>>) -> Self {
        Self {
            tokens,
            end: tokens.len(),
            index: 0,
        }
    }

    /// Returns next value in sequence without consuming it.
    /// Identical to Peekable .peek() method
    pub fn peek(&mut self) -> Option<&Located<Lexeme>> {
        if self.index < self.end {
            self.tokens.get(self.index)
        } else {
            None
        }
    }

    /// Advances the iterator if the next value meets the provided predicate
    pub fn next_if(&mut self, f: impl FnOnce(&Lexeme) -> bool) -> Option<Located<Lexeme>> {
        match self.peek() {
            Some(t) if f(&t.data) => self.next(),
            _ => None,
        }
    }

    pub fn prev(&mut self) -> Option<Located<Lexeme>> {
        if self.index == 0 {
            None
        } else {
            self.index -= 1;
            self.tokens.get(self.index).cloned()
        }
    }

    /// Advances if the next token is equal to the lexeme provided. Usually used
    /// for checking if keywords and symbols are present
    pub fn next_if_eq(&mut self, lex: Lexeme) -> Option<Location> {
        self.next_if(|t| *t == lex).map(|t| t.loc)
    }

    pub fn next_if_key(&mut self, key: Keyword) -> Option<Location> {
        self.next_if_eq(Lexeme::Keyword(key))
    }

    pub fn expect_next(&mut self, f: impl FnOnce(&Lexeme) -> bool) -> ParseResult<Located<Lexeme>> {
        match self.next() {
            Some(t) if f(&t.data) => Ok(t),
            Some(t) => Err(ParseError::UnexpectedToken(t)),
            None => Err(self.eof_err()),
        }
    }

    pub fn expect_next_eq(&mut self, lex: Lexeme) -> ParseResult<Location> {
        self.expect_next(|t| *t == lex).map(|t| t.loc)
    }

    pub fn expect_semicolon(&mut self) -> ParseResult<()> {
        self.expect_next_eq(Lexeme::Semicolon).map(std::mem::drop)
    }

    pub fn expect_colon(&mut self) -> ParseResult<()> {
        self.expect_next_eq(Lexeme::Colon).map(std::mem::drop)
    }

    pub fn expect_string(&mut self) -> ParseResult<Located<usize>> {
        match self.next() {
            Some(Located {
                data: Lexeme::String(s),
                loc,
            }) => Ok(Located::new(s, loc)),
            Some(t) => Err(ParseError::UnexpectedToken(t)),
            None => Err(self.eof_err()),
        }
    }

    pub fn expect_ident(&mut self) -> ParseResult<Identifier> {
        match self.next() {
            Some(Located {
                data: Lexeme::Identifier(i),
                loc,
            }) => Ok(Located::new(i, loc)),
            Some(t) => Err(ParseError::UnexpectedToken(t)),
            None => Err(self.eof_err()),
        }
    }

    pub fn expect_int(&mut self) -> ParseResult<Located<u32>> {
        match self.next() {
            Some(Located {
                data: Lexeme::Integer(i),
                loc,
            }) => Ok(Located::new(i as u32, loc)),
            Some(Located {
                data: Lexeme::Unsigned(i),
                loc,
            }) => Ok(Located::new(i as u32, loc)),
            Some(t) => Err(ParseError::UnexpectedToken(t)),
            None => Err(self.eof_err()),
        }
    }

    pub fn expect_begin(&mut self, grouper: Grouper) -> ParseResult<()> {
        self.expect_next_eq(Lexeme::GroupBegin(grouper))
            .map(std::mem::drop)
    }

    pub fn expect_end(&mut self, grouper: Grouper) -> ParseResult<()> {
        self.expect_next_eq(Lexeme::GroupEnd(grouper))
            .map(std::mem::drop)
    }

    pub fn expect_group<T>(
        &mut self,
        grouper: Grouper,
        f: impl FnOnce(&mut Self) -> ParseResult<T>,
    ) -> ParseResult<T> {
        self.expect_begin(grouper)?;
        let ret = f(self)?;
        self.expect_end(grouper)?;
        Ok(ret)
    }

    /// Gets the lexeme that terminates the sequence. Not to be confused
    /// with the last lexeme in a sequence. This will be EOF (None) in the
    /// base lex stream and will often be a delimiter (e.g. ',') or a group
    /// end (e.g. ')', ']') in lexeme subsequences.
    pub fn get_end(&self) -> Option<&Located<Lexeme>> {
        self.tokens.get(self.end)
    }

    /// Checks if iterator has no values left to iterate over
    pub fn is_empty(&self) -> bool {
        self.index == self.end
    }

    /// Returns proper EOF error. This will be UnexpectedEOF in the base iterator
    /// and a terminator (',', ')', '}') in cases of a token slice
    pub fn eof_err(&self) -> ParseError {
        match self.get_end() {
            Some(t) => ParseError::UnexpectedToken(t.clone()),
            None => ParseError::UnexpectedEOF,
        }
    }

    pub fn until_level(
        &mut self,
        predicate: impl Fn(&Lexeme) -> bool,
    ) -> ParseResult<TokenIter<'a>> {
        let mut stack = VecDeque::new();

        let start = self.index;
        let mut end = self.index;

        loop {
            match self.next() {
                Some(Located {
                    data: Lexeme::GroupBegin(g),
                    ..
                }) => {
                    stack.push_back(g);
                }
                Some(t) if stack.is_empty() && predicate(&t.data) => {
                    return Ok(Self {
                        tokens: self.tokens,
                        index: start,
                        end,
                    })
                }
                Some(Located {
                    data: Lexeme::GroupEnd(g),
                    loc,
                }) if !stack.is_empty() => {
                    if stack.pop_back() != Some(g) {
                        return Err(ParseError::UnexpectedToken(Located {
                            data: Lexeme::GroupEnd(g),
                            loc,
                        }));
                    }
                }
                Some(_) => {}
                None => return Err(self.eof_err()),
            }
            end += 1;
        }
    }

    pub fn until_level_eq(&mut self, lex: Lexeme) -> ParseResult<TokenIter<'a>> {
        self.until_level(|t| *t == lex)
    }

    pub fn take_group(&mut self, grouper: Grouper) -> ParseResult<TokenIter<'a>> {
        self.until_level_eq(Lexeme::GroupEnd(grouper))
    }

    pub fn level_split_comma(&mut self, grouper: Grouper) -> ParseResult<Vec<TokenIter<'a>>> {
        let mut groups = Vec::new();
        loop {
            let tokens =
                self.until_level(|t| *t == Lexeme::Comma || *t == Lexeme::GroupEnd(grouper))?;
            match tokens.get_end() {
                Some(Located {
                    data: Lexeme::Comma,
                    ..
                }) => {
                    groups.push(tokens);
                }
                Some(Located {
                    data: Lexeme::GroupEnd(_),
                    ..
                }) => {
                    // Allow for a single trailing comma
                    if !tokens.is_empty() {
                        groups.push(tokens);
                    }
                    break;
                }
                _ => unreachable!(),
            }
        }
        Ok(groups)
    }
}
