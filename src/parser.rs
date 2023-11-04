// TODO: Fix the case conversion error whose any part of names have
//       special letters like Ñ.
use error_stack::{Report, Result};
use heck::ToTitleCase;
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    fmt::Display,
    io::{BufReader, Read},
    marker::PhantomData,
};

use crate::{types::RawQualifier, utils::LineReader};

pub struct Qualifiers<'a, R: LineReader<'a>> {
    lineno: u32,
    reader: R,
    _phantom: PhantomData<&'a str>,
}

impl<'a, R: Read> Qualifiers<'a, BufReader<R>> {
    pub fn from_reader(reader: R) -> Self {
        Self {
            lineno: 1,
            reader: BufReader::new(reader),
            _phantom: PhantomData,
        }
    }
}

impl<'a> Qualifiers<'a, std::str::Lines<'a>> {
    pub fn from_str(input: &'a str) -> Self {
        Self {
            lineno: 1,
            reader: input.lines(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, R: LineReader<'a>> Qualifiers<'a, R> {
    fn next_lineno(&self) -> Result<u32, ParseError> {
        const U32_MAX: u32 = u32::MAX;

        let current_line = self.lineno;
        if current_line == U32_MAX {
            Err(ParseError::new(ParseErrorType::TooBig, Some(current_line)).into())
        } else {
            Ok(current_line)
        }
    }

    fn parse_line(&mut self, line: &str) -> Result<RawQualifier, ParseError> {
        static APPLICATION_ID_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("^[0-9]{7,7}$").expect("to compile regex"));

        static SEAT_NUMBER_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("^[0-9]{3,3}$").expect("to compile regex"));

        static ROOM_ASSIGNMENT_REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("^[0-9]{1,2}$").expect("to compile regex"));

        static TEST_CENTER_CODE: Lazy<Regex> =
            Lazy::new(|| Regex::new("^[0-9]{4,4}$").expect("to compile regex"));

        let lineno = self.next_lineno()?;

        // PSHS for some reason put leftover tabular character there :)
        let mut parts = line.trim().split('\t');

        macro_rules! next_or_err {
            ($ty:ident) => {
                parts.next().ok_or_else(|| {
                    Report::new(ParseError::new(ParseErrorType::$ty, Some(lineno)))
                })?
            };
        }

        let surname = next_or_err!(MissingSurname);
        let first_name = next_or_err!(MissingFirstName);

        let mut middle_name = None;

        // Middle name and application id parsing
        let id: u64 = {
            let first = next_or_err!(MissingApplicationId);
            if APPLICATION_ID_REGEX.is_match(first) {
                first
            } else {
                if !first.is_empty() {
                    middle_name = Some(first.to_title_case());
                }
                let second = next_or_err!(MissingApplicationId);
                if APPLICATION_ID_REGEX.is_match(second) {
                    second
                } else {
                    return Err(ParseError::new(
                        ParseErrorType::InvalidApplicationId,
                        Some(lineno),
                    )
                    .into());
                }
            }
            .parse()
            .map_err(|err| {
                panic!("failed to compile u64 from already tested string at line {lineno}: {err}")
            })
            .unwrap()
        };

        let seat_number = next_or_err!(MissingSeatNumber);
        let seat_number =
            if SEAT_NUMBER_REGEX.is_match(seat_number) {
                seat_number.parse::<u32>().map_err(|err| {
                panic!("failed to compile u64 from already tested string at line {lineno}: {err}")
            }).unwrap()
            } else {
                return Err(
                    ParseError::new(ParseErrorType::InvalidSeatNumber, Some(lineno)).into(),
                );
            };

        let time = next_or_err!(MissingTime).to_string();

        let room_assignment = next_or_err!(MissingRoomAssignment);
        let room_assignment =
            if ROOM_ASSIGNMENT_REGEX.is_match(room_assignment) {
                room_assignment.parse::<u32>().map_err(|err| {
                panic!("failed to compile u32 from already tested string at line {lineno}: {err}")
            }).unwrap()
            } else {
                return Err(
                    ParseError::new(ParseErrorType::InvalidRoomAssignment, Some(lineno)).into(),
                );
            };

        let test_center_code = next_or_err!(MissingTestCenterCode);
        let test_center_code =
            if TEST_CENTER_CODE.is_match(test_center_code) {
                test_center_code.parse::<u32>().map_err(|err| {
                panic!("failed to compile u32 from already tested string at line {lineno}: {err}")
            }).unwrap()
            } else {
                return Err(
                    ParseError::new(ParseErrorType::InvalidTestCenterCode, Some(lineno)).into(),
                );
            };

        Ok(RawQualifier {
            id,
            // I'm super duper sorry whose names have lowercase segments
            surname: surname.to_title_case(),
            first_name: first_name.to_title_case(),
            middle_name,
            seat_number,
            room_assignment,
            time,
            test_center_code,
            test_center_name: next_or_err!(MissingTestCenterName).to_string(),
            test_center_addr: next_or_err!(MissingTestCenterAddr).to_string(),
        })
    }
}

impl<'a, R: LineReader<'a>> Iterator for Qualifiers<'a, R> {
    type Item = Result<RawQualifier, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.reader.next_line()? {
            Ok(line) => self.parse_line(&line),
            Err(err) => Err(err),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ParseError {
    kind: ParseErrorType,
    line: Option<u32>,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)?;
        if let Some(line) = self.line {
            write!(f, " at line {line}")?;
        }
        Ok(())
    }
}

impl ParseError {
    pub(crate) fn new(kind: ParseErrorType, line: Option<u32>) -> Self {
        Self { kind, line }
    }

    pub fn error_type(&self) -> &ParseErrorType {
        &self.kind
    }

    pub fn line(&self) -> Option<u32> {
        self.line
    }
}

impl error_stack::Context for ParseError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorType {
    IO,
    TooBig,
    MissingSurname,
    MissingFirstName,
    MissingApplicationId,
    InvalidApplicationId,
    MissingSeatNumber,
    InvalidSeatNumber,
    MissingRoomAssignment,
    InvalidRoomAssignment,
    MissingTime,
    MissingTestCenterCode,
    InvalidTestCenterCode,
    MissingTestCenterName,
    MissingTestCenterAddr,
}

impl Display for ParseErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseErrorType::IO => f.write_str("I/O error occurred"),
            ParseErrorType::TooBig => f.write_str("data is too big to handle"),
            ParseErrorType::MissingSurname => f.write_str("missing surname data"),
            ParseErrorType::MissingFirstName => f.write_str("missing first name data"),
            ParseErrorType::MissingApplicationId => f.write_str("missing application id"),
            ParseErrorType::InvalidApplicationId => f.write_str("invalid application id"),
            ParseErrorType::MissingSeatNumber => f.write_str("missing seat number"),
            ParseErrorType::MissingTime => f.write_str("missing time"),
            ParseErrorType::InvalidSeatNumber => f.write_str("invalid seat number"),
            ParseErrorType::MissingRoomAssignment => f.write_str("missing room assignment"),
            ParseErrorType::InvalidRoomAssignment => f.write_str("invalid room assignment"),
            ParseErrorType::MissingTestCenterCode => f.write_str("missing test center code"),
            ParseErrorType::InvalidTestCenterCode => f.write_str("invalid test center code"),
            ParseErrorType::MissingTestCenterName => f.write_str("missing test center name"),
            ParseErrorType::MissingTestCenterAddr => f.write_str("missing test center addr"),
        }
    }
}
