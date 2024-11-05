use crate::{
    gcode::Tcode,
    parser::{map_res_f1, nom_types::IParseResult, parse_utils::parse_u32},
    GcodeParser,
};
use nom::Parser;

impl GcodeParser for Tcode {
    fn parse(input: &[u8]) -> IParseResult<'_, Self> {
        parse_tcode(input)
    }
}

fn parse_tcode(input: &[u8]) -> IParseResult<'_, Tcode> {
    map_res_f1(parse_u32(), Tcode).parse(input)
}
