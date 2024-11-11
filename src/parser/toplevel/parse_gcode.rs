use crate::{
    gcode::{Axes, Gcode},
    parser::{
        map_res_f1,
        nom_types::{IParseResult, IntoParser as _},
        parse_code_and_number::parse_code_and_number,
    },
    GcodeParser,
};
use nom::{combinator::opt, Parser as _};

impl GcodeParser for Gcode {
    fn parse(input: &[u8]) -> IParseResult<'_, Self> {
        parse_gcode(input)
    }
}

fn parse_gcode(input: &[u8]) -> IParseResult<'_, Gcode> {
    parse_code_and_number(
        b'G',
        (
            ("0", map_res_f1(opt(Axes::parse), Gcode::G0)),
            ("1", map_res_f1(Axes::parse, Gcode::G1)),
            ("20", Gcode::G20.into_parser()),
            ("21", Gcode::G21.into_parser()),
            ("53", Gcode::G53.into_parser()),
            ("54", Gcode::G54.into_parser()),
            ("90", Gcode::G90.into_parser()),
            ("91", Gcode::G91.into_parser()),
        ),
    )
    .parse(input)
}
