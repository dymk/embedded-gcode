use nom::{bytes::complete::tag, combinator::map_res, sequence::tuple};

use crate::{
    bind,
    gcode::{
        expression::{Expression, Param},
        Command, GcodeParser as _,
    },
    parser::{nom_types::IParseResult, ok, space_before},
    ParserAllocator,
};

pub fn parse_assignment<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Command<'b>> {
    map_res(
        tuple((
            bind!(alloc, Param::parse),
            space_before(tag("=")),
            bind!(alloc, Expression::parse),
        )),
        |(param, _, expr)| ok(Command::Assign(param, expr)),
    )(input)
}
