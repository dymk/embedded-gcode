use nom::combinator::fail;

use crate::{gcode::Scode, parser::nom_types::IParseResult};

pub fn parse_scode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Scode> {
    fail
}
