use crate::{
    bind,
    gcode::Command,
    parser::{
        nom_types::IParseResult, ok, parse_comment, parse_gcode, parse_mcode, parse_ocode,
        parse_scode, parse_tcode, parse_utils::space_before,
    },
    ParserAllocator,
};
use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::space0, combinator::map_res,
    sequence::preceded,
};

pub fn parse_command<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Command<'b>> {
    preceded(
        space0,
        alt((
            bind!(alloc, parse_comment),
            parse_prefix(alloc, 'G', Command::G, parse_gcode),
            parse_prefix(alloc, 'M', Command::M, parse_mcode),
            parse_prefix(alloc, 'O', Command::O, parse_ocode),
            parse_prefix(alloc, 'S', Command::S, parse_scode),
            parse_prefix(alloc, 'T', Command::T, parse_tcode),
        )),
    )(input)
}

fn parse_prefix<'a, 'b, SubCommand>(
    alloc: &'b ParserAllocator<'b>,
    // 'G', etc
    command_char: char,
    // Map the parsed sub-command into a Command e.g. Gcode into Command::G(Gcode)
    command_ctor: impl Fn(SubCommand) -> Command<'b>,
    // The parser for the sub-command, results in a Gcode, Mcode, etc
    command_parser: fn(&'b ParserAllocator<'b>, &'a [u8]) -> IParseResult<'a, SubCommand>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Command<'b>> {
    map_res(
        preceded(
            space_before(tag_no_case([command_char as u8])),
            bind!(alloc, command_parser),
        ),
        move |parsed| ok(command_ctor(parsed)),
    )
}
