use nom::combinator::map_res;

use crate::{
    gcode::Tcode,
    parser::{
        nom_types::{ok, IParseResult},
        parse_utils::parse_u32,
    },
    ParserAllocator,
};

pub fn parse_tcode<'a, 'b>(_: &'b ParserAllocator<'b>, input: &'a [u8]) -> IParseResult<'a, Tcode> {
    map_res(parse_u32(), |val| ok(Tcode(val)))(input)
}
