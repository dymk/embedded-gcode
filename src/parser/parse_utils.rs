use crate::parser::nom_types::IParseResult;
use core::str::from_utf8;
use nom::{
    bytes::complete::tag,
    character::complete::{digit1, space0},
    combinator::{map_res, not},
    sequence::{preceded, terminated},
};

pub fn parse_u32<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, u32> {
    map_res(digit1, |bytes| {
        str::parse(match from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => "invalid",
        })
    })
}

#[inline(always)]
pub fn number_code<'a>(number: &'static str) -> impl FnMut(&'a [u8]) -> IParseResult<'a, &'a [u8]> {
    // exact number str followed by non-digit
    terminated(tag(number), not(digit1))
}

#[inline(always)]
pub fn space_before<'a, T>(
    parser: impl FnMut(&'a [u8]) -> IParseResult<'a, T>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, T> {
    preceded(space0, parser)
}
