use crate::{
    gcode::Scode,
    parser::{map_res_f1, nom_types::IParseResult},
    GcodeParser,
};
use nom::{number::complete::float, Parser as _};

impl GcodeParser for Scode {
    fn parse(input: &[u8]) -> IParseResult<'_, Self> {
        parse_scode(input)
    }
}

fn parse_scode(input: &[u8]) -> IParseResult<'_, Scode> {
    map_res_f1(float, Scode).parse(input)
}
