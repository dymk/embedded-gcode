use nom::{combinator::map_res, number::complete::float};

use crate::{
    gcode::Scode,
    parser::nom_types::{ok, IParseResult},
    NomAlloc,
};

pub fn parse_scode<'a, 'b>(_: NomAlloc<'b>, input: &'a [u8]) -> IParseResult<'a, Scode> {
    map_res(float, |value| ok(Scode(value)))(input)
}
