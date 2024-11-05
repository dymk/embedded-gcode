use alloc::string::FromUtf8Error;
use core::str::Utf8Error;
use nom::{
    error::{Error as NomError, ErrorKind as NomErrorKind, FromExternalError},
    IResult as NomIResult,
};

#[derive(Debug, PartialEq)]
pub enum GcodeParseError<'a> {
    NomError(NomError<&'a [u8]>),
    OutOfMemory,
    Utf8Error,
}

impl<'a> From<NomError<&'a [u8]>> for GcodeParseError<'a> {
    fn from(value: NomError<&'a [u8]>) -> Self {
        GcodeParseError::NomError(value)
    }
}

impl From<Utf8Error> for GcodeParseError<'_> {
    fn from(_: Utf8Error) -> Self {
        GcodeParseError::Utf8Error
    }
}

impl From<FromUtf8Error> for GcodeParseError<'_> {
    fn from(_: FromUtf8Error) -> Self {
        GcodeParseError::Utf8Error
    }
}

impl<'a, E> FromExternalError<&'a [u8], E> for GcodeParseError<'a> {
    fn from_external_error(input: &'a [u8], kind: NomErrorKind, e: E) -> Self {
        GcodeParseError::NomError(NomError::from_external_error(input, kind, e))
    }
}

impl<'a> nom::error::ParseError<&'a [u8]> for GcodeParseError<'a> {
    fn from_error_kind(input: &'a [u8], kind: NomErrorKind) -> Self {
        GcodeParseError::NomError(NomError::from_error_kind(input, kind))
    }

    fn append(_input: &'a [u8], _kind: NomErrorKind, other: Self) -> Self {
        other
    }
}

pub type IParseResult<'a, O> = NomIResult<&'a [u8], O, GcodeParseError<'a>>;
