use error_stack::{Report, Result};
use std::{
    borrow::Cow,
    io::{BufRead, BufReader, Read},
};

use crate::parser::{ParseError, ParseErrorType};

pub trait LineReader<'a> {
    fn next_line(&mut self) -> Option<Result<Cow<'a, str>, ParseError>>;
}

impl<'a> LineReader<'a> for std::str::Lines<'a> {
    fn next_line(&mut self) -> Option<Result<Cow<'a, str>, ParseError>> {
        self.next().map(|v| Ok(Cow::Borrowed(v)))
    }
}

impl<'a, R: Read> LineReader<'a> for BufReader<R> {
    fn next_line(&mut self) -> Option<Result<Cow<'a, str>, ParseError>> {
        // Copied from: https://github.com/rust-lang/rust/blob/1.73.0/library/std/src/io/mod.rs#L2852
        // Licensed under MIT License
        let mut buf = String::new();
        match self.read_line(&mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.ends_with('\n') {
                    buf.pop();
                    if buf.ends_with('\r') {
                        buf.pop();
                    }
                }
                Some(Ok(Cow::Owned(buf)))
            }
            Err(e) => Some(Err(
                Report::new(e).change_context(ParseError::new(ParseErrorType::IO, None))
            )),
        }
    }
}
