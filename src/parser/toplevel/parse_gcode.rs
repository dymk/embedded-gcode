use crate::gcode::{Axes, GcodeParser as _};
use crate::parser::parse_utils::number_code;
use crate::parser::{map_res_f1, ok};
use crate::{gcode::Gcode, parser::nom_types::IParseResult};
use nom::{
    branch::alt,
    combinator::{map_res, opt},
    sequence::preceded,
};

pub fn parse_gcode<'a>(input: &'a [u8]) -> IParseResult<'a, Gcode> {
    fn simple_gcode<'a>(
        number_str: &'static str,
        gcode: Gcode,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Gcode> {
        map_res(number_code(number_str), move |_| ok(gcode.clone()))
    }

    alt((
        map_res_f1(preceded(number_code("0"), opt(Axes::parse)), Gcode::G0),
        map_res_f1(preceded(number_code("1"), Axes::parse), Gcode::G1),
        simple_gcode("20", Gcode::G20),
        simple_gcode("21", Gcode::G21),
        simple_gcode("53", Gcode::G53),
        simple_gcode("54", Gcode::G54),
        simple_gcode("90", Gcode::G90),
        simple_gcode("91", Gcode::G91),
    ))(input)
}
