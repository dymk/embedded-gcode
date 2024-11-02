use crate::{
    gcode::Command,
    parser::{nom_types::IParseResult, ok, parse_utils::space_before},
    ParserAllocator,
};
use nom::{
    bytes::complete::{tag, take_until1},
    combinator::map_res,
    sequence::delimited,
};

pub fn parse_comment<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Command<'b>> {
    map_res(
        delimited(space_before(tag("(")), take_until1(")"), tag(")")),
        move |bytes| {
            let comment_str = alloc.alloc_str_from_bytes(bytes)?;
            ok(Command::Comment(comment_str))
        },
    )(input)
}
