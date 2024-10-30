use crate::{
    gcode::Command,
    parser::{
        nom_alloc::NomAlloc,
        nom_types::{ok, IParseResult},
        parse_utils::space_before,
    },
};
use nom::{
    bytes::complete::{tag, take_until1},
    combinator::map_res,
    sequence::delimited,
};

pub fn parse_comment<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Command<'b>> {
    map_res(
        delimited(space_before(tag("(")), take_until1(")"), tag(")")),
        move |bytes| {
            let comment_str = alloc.alloc_str_from_bytes(bytes)?;
            ok(Command::Comment(comment_str))
        },
    )
}
