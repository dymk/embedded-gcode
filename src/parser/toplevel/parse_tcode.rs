use nom::combinator::fail;

use crate::{gcode::Tcode, parser::nom_types::IParseResult};

pub fn parse_tcode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Tcode> {
    fail
}
