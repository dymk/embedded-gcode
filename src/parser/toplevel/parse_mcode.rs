use nom::combinator::fail;

use crate::{gcode::Mcode, parser::nom_types::IParseResult};

pub fn parse_mcode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Mcode> {
    fail
}
