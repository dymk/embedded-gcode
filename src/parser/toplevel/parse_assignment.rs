use nom::{bytes::complete::tag, combinator::map_res, sequence::tuple};

use crate::{
    gcode::{
        expression::{Expression, Param},
        Command, GcodeParser as _,
    },
    parser::{nom_types::IParseResult, ok, space_before},
};

pub fn parse_assignment<'a>(input: &'a [u8]) -> IParseResult<'a, Command> {
    map_res(
        tuple((Param::parse, space_before(tag("=")), Expression::parse)),
        |(param, _, expr)| ok(Command::Assign(param, expr)),
    )(input)
}
