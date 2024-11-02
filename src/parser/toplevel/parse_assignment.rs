use nom::{bytes::complete::tag, combinator::map_res, sequence::tuple};

use crate::{
    bind,
    gcode::Command,
    parser::{nom_types::IParseResult, ok, space_before},
    ParserAllocator,
};

use super::{parse_expression, parse_param};

pub fn parse_assignment<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Command<'b>> {
    map_res(
        tuple((
            bind!(alloc, parse_param),
            space_before(tag("=")),
            bind!(alloc, parse_expression),
        )),
        |(param, _, expr)| ok(Command::Assign(param, expr)),
    )(input)
}
