use nom::{bytes::complete::tag, combinator::map_res, sequence::tuple};

use crate::{
    gcode::{
        expression::{Expression, Param},
        Command,
    },
    parser::{nom_types::IParseResult, ok, space_before},
    GcodeParser as _,
};

pub fn parse_assignment(input: &[u8]) -> IParseResult<'_, Command> {
    map_res(
        tuple((Param::parse, space_before(tag("=")), Expression::parse)),
        |(param, _, expr)| ok(Command::Assign(param, expr)),
    )(input)
}
