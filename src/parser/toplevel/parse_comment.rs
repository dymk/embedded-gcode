use crate::{
    gcode::Command,
    parser::{nom_types::IParseResult, ok, parse_utils::space_before, Input},
};
use alloc::string::String;
use nom::{
    bytes::complete::{tag, take_until1},
    combinator::map_res,
    sequence::delimited,
};

pub fn parse_comment(input: Input) -> IParseResult<Command> {
    map_res(
        delimited(space_before(tag("(")), take_until1(")"), tag(")")),
        move |input| {
            let comment_str = String::from_utf8(input.as_bytes().to_vec())?;
            ok(Command::Comment(comment_str))
        },
    )(input)
}
