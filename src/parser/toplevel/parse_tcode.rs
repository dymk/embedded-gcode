use crate::{
    gcode::Tcode,
    parser::{map_res_f1, nom_types::IParseResult, parse_utils::parse_u32},
    ParserAllocator,
};
use nom::Parser;

pub fn parse_tcode<'a, 'b>(_: &'b ParserAllocator<'b>, input: &'a [u8]) -> IParseResult<'a, Tcode> {
    map_res_f1(parse_u32(), Tcode).parse(input)
}
