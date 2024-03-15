use deimos_ast::Location;
use std::iter::Peekable;
use std::str::Chars;

use super::{LexError, LexErrorKind};

/// Encapsulates char peeker so that the char location is
/// properly kept and updated.
pub struct ChIter<'a> {
    peeker: Peekable<Chars<'a>>,
    loc: Location,
}

impl<'a> Iterator for ChIter<'a> {
    type Item = char;

    /// Advances iterator and increments location
    fn next(&mut self) -> Option<Self::Item> {
        let val = self.peeker.next();
        match val {
            Some('\n') => {
                self.loc.col = 0;
                self.loc.row += 1;
            }
            Some(_c) => {
                self.loc.col += 1;
            }
            None => {}
        }
        val
    }
}

impl<'a> ChIter<'a> {
    pub fn get_loc(&self) -> Location {
        self.loc
    }

    pub fn new(s: &'a str) -> Self {
        ChIter {
            peeker: s.chars().peekable(),
            loc: Location { row: 1, col: 0 },
        }
    }

    pub fn peek(&mut self) -> Option<char> {
        self.peeker.peek().cloned()
    }

    pub fn next_if(&mut self, f: impl FnOnce(char) -> bool) -> Option<char> {
        match self.peek() {
            Some(c) if f(c) => self.next(),
            _ => None,
        }
    }

    pub fn next_if_eq(&mut self, ch: char) -> bool {
        match self.peek() {
            Some(c) if c == ch => {
                self.next();
                true
            }
            _ => false,
        }
    }

    /// Skip while that doesn't consume last char
    pub fn pass_over(&mut self, f: impl Fn(char) -> bool) {
        while let Some(_t) = self.next_if(&f) {}
    }

    /// Returns error if there isn't another character (EOF error)
    pub fn expect_any(&mut self) -> Result<char, LexError> {
        self.next().ok_or(LexError {
            loc: self.get_loc(),
            kind: LexErrorKind::UnexpectedEOF,
        })
    }

    /// Returns an error regardless whether the next is found
    /// (EOF or unexpected char error)
    pub fn expect_any_err(&mut self) -> LexError {
        match self.expect_any() {
            Ok(c) => LexError {
                loc: self.get_loc(),
                kind: LexErrorKind::UnexpectedChar(c),
            },
            Err(e) => e,
        }
    }
}
