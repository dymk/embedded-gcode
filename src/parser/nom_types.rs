use alloc::string::FromUtf8Error;
use core::str::Utf8Error;
use nom::{
    error::{Error as NomError, ErrorKind as NomErrorKind, FromExternalError},
    IResult, Parser,
};

use crate::gcode::ParseNode;

use super::Input;

#[derive(Debug, PartialEq)]
pub enum GcodeParseError<'a> {
    NomError(NomError<Input<'a>>),
    OutOfMemory,
    Utf8Error,
}

impl<'a> From<NomError<Input<'a>>> for GcodeParseError<'a> {
    fn from(value: NomError<Input<'a>>) -> Self {
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

impl<'a, E> FromExternalError<Input<'a>, E> for GcodeParseError<'a> {
    fn from_external_error(input: Input<'a>, kind: NomErrorKind, e: E) -> Self {
        GcodeParseError::NomError(NomError::from_external_error(input, kind, e))
    }
}

impl<'a> nom::error::ParseError<Input<'a>> for GcodeParseError<'a> {
    fn from_error_kind(input: Input<'a>, kind: NomErrorKind) -> Self {
        GcodeParseError::NomError(NomError::from_error_kind(input, kind))
    }

    fn append(_input: Input<'a>, _kind: NomErrorKind, other: Self) -> Self {
        other
    }
}

pub type IParseResult<'a, O> = IResult<Input<'a>, O, GcodeParseError<'a>>;

pub trait IntoParser<'a>
where
    Self: Sized,
{
    fn into_parser(self) -> impl Parser<Input<'a>, Self, GcodeParseError<'a>>;
}
impl<'a, O> IntoParser<'a> for O
where
    O: ParseNode,
{
    fn into_parser(self) -> impl Parser<Input<'a>, Self, GcodeParseError<'a>> {
        move |input| Ok((input, self.clone()))
    }
}
