use crate::{
    gcode::Scode,
    parser::{map_res_f1, nom_types::IParseResult},
    ParserAllocator,
};
use nom::{number::complete::float, Parser};

pub fn parse_scode<'a, 'b>(_: &'b ParserAllocator<'b>, input: &'a [u8]) -> IParseResult<'a, Scode> {
    map_res_f1(float, Scode).parse(input)
}
