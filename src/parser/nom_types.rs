use core::str::Utf8Error;

use crate::gcode::Gcode;

use super::nom_alloc::AllocError;
use nom::{
    error::{Error as NomError, ErrorKind as NomErrorKind, FromExternalError},
    IResult as NomIResult,
};

#[derive(Debug)]
pub enum GcodeParseError<'a> {
    NomError(NomError<&'a [u8]>),
    OutOfMemory,
    Utf8Error,
}

impl<'a> From<AllocError> for GcodeParseError<'a> {
    fn from(value: AllocError) -> Self {
        match value {
            AllocError::OutOfMemory => GcodeParseError::OutOfMemory,
            AllocError::Utf8Error => GcodeParseError::Utf8Error,
        }
    }
}

impl<'a> From<NomError<&'a [u8]>> for GcodeParseError<'a> {
    fn from(value: NomError<&'a [u8]>) -> Self {
        GcodeParseError::NomError(value)
    }
}

impl<'a> From<Utf8Error> for GcodeParseError<'a> {
    fn from(_: Utf8Error) -> Self {
        GcodeParseError::Utf8Error
    }
}

impl<'a> From<AllocError> for nom::Err<GcodeParseError<'a>> {
    fn from(value: AllocError) -> Self {
        nom::Err::Error(value.into())
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

pub fn ok<'a, T>(t: T) -> Result<T, GcodeParseError<'a>> {
    Ok(t)
}

pub fn err<'a, T>(e: impl Into<GcodeParseError<'a>>) -> Result<T, GcodeParseError<'a>> {
    Err(e.into())
}
