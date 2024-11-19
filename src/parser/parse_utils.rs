use crate::parser::nom_types::IParseResult;
use crate::GcodeParseError;
use nom::{
    bytes::complete::tag,
    character::complete::digit1,
    character::complete::space0,
    combinator::{map_res, not},
    sequence::{preceded, terminated},
    Parser,
};

use super::Input;

pub fn parse_u32<'a>() -> impl FnMut(Input<'a>) -> IParseResult<'a, u32> {
    map_res(digit1, |input: Input<'a>| {
        str::parse(match input.as_utf8() {
            Ok(s) => s,
            Err(_) => "invalid",
        })
    })
}

#[inline(always)]
pub fn map_res_f1<'a, 'b, T, R>(
    parser: impl Parser<Input<'a>, T, GcodeParseError<'a>>,
    ctor: impl Fn(T) -> R,
) -> impl Parser<Input<'a>, R, GcodeParseError<'a>> {
    map_res(parser, move |value| ok(ctor(value)))
}

#[inline(always)]
pub fn map_res_into_ok<'a, T: Into<R>, R>(
    parser: impl Parser<Input<'a>, T, GcodeParseError<'a>>,
) -> impl Parser<Input<'a>, R, GcodeParseError<'a>> {
    map_res(parser, move |value| ok(value.into()))
}

#[inline(always)]
pub fn number_code<'a>(
    number: &'static str,
) -> impl FnMut(Input<'a>) -> IParseResult<'a, Input<'a>> {
    // exact number str followed by non-digit
    terminated(tag(number), not(digit1))
}

#[inline(always)]
pub fn space_before<'a, T>(
    parser: impl Parser<Input<'a>, T, GcodeParseError<'a>>,
) -> impl FnMut(Input<'a>) -> IParseResult<'a, T> {
    preceded(space0, parser)
}

#[inline(always)]
pub fn ok<'a, T>(t: T) -> Result<T, GcodeParseError<'a>> {
    Ok(t)
}

#[inline(always)]
pub fn err<'a, T>(e: impl Into<GcodeParseError<'a>>) -> Result<T, GcodeParseError<'a>> {
    Err(e.into())
}
