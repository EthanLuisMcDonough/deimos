use super::{Grouper, ParseError, ParseResult};
use crate::lexer::Lexeme;
use deimos_ast::Located;
use std::{collections::VecDeque, iter::Peekable};

pub struct TokenPeeker {
    iter: Peekable<std::vec::IntoIter<Located<Lexeme>>>,
}

impl Iterator for TokenPeeker {
    type Item = Located<Lexeme>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

impl TokenPeeker {
    pub fn new(v: Vec<Located<Lexeme>>) -> Self {
        Self {
            iter: v.into_iter().peekable(),
        }
    }

    pub fn peek(&mut self) -> Option<&Located<Lexeme>> {
        self.iter.peek()
    }

    pub fn next_if(&mut self, f: impl FnOnce(&Lexeme) -> bool) -> Option<Located<Lexeme>> {
        self.iter.next_if(|l| f(&l.data))
    }

    pub fn next_if_eq(&mut self, token: &Lexeme) -> Option<Located<Lexeme>> {
        self.iter.next_if(|l| l.data == *token)
    }

    pub fn until_level(&mut self, grouper: Grouper) -> TokenLevel<'_> {
        let mut stack = VecDeque::with_capacity(1);
        stack.push_back(grouper);
        TokenLevel { iter: self, stack }
    }
}

pub struct TokenLevel<'a> {
    iter: &'a mut TokenPeeker,
    stack: VecDeque<Grouper>,
}

impl<'a> Iterator for TokenLevel<'a> {
    type Item = ParseResult<Located<Lexeme>>;

    fn next(&mut self) -> Option<Self::Item> {
        let v = self.iter.next();
        match &v {
            Some(Located {
                data: Lexeme::GroupBegin(g),
                ..
            }) => {
                self.stack.push_back(*g);
            }
            Some(Located {
                data: Lexeme::GroupEnd(g),
                loc,
            }) => {
                if self.stack.pop_back() != Some(*g) {
                    return Some(Err(ParseError::UnexpectedToken(Located {
                        data: Lexeme::GroupEnd(*g),
                        loc: *loc,
                    })));
                }
            }
            None if !self.stack.is_empty() => return Some(Err(ParseError::UnexpectedEOF)),
            None => return None,
            _ => {}
        }
        v.map(Ok)
    }
}
