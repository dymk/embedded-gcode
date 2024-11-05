use crate::{
    gcode::Scode,
    parser::{map_res_f1, nom_types::IParseResult},
};
use nom::{number::complete::float, Parser as _};

pub fn parse_scode<'a>(input: &'a [u8]) -> IParseResult<'a, Scode> {
    map_res_f1(float, Scode).parse(input)
}
