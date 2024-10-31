use nom::combinator::map_res;

use crate::{
    gcode::Tcode,
    parser::{
        nom_types::{ok, IParseResult},
        parse_utils::parse_u32,
    },
    Parser,
};

impl<'b> Parser<'b> {
    pub fn parse_tcode<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Tcode> {
        map_res(parse_u32(), |val| ok(Tcode(val)))(input)
    }
}
