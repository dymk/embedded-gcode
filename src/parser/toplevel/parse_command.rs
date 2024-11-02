use crate::{
    bind,
    gcode::Command,
    parser::{nom_types::IParseResult, ok, parse_utils::space_before, toplevel::*},
    ParserAllocator,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{map_res, peek},
    sequence::preceded,
};

pub fn parse_command<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Command<'b>> {
    let assignment = preceded(space_before(peek(tag("#"))), bind!(alloc, parse_assignment));

    space_before(alt((
        bind!(alloc, parse_comment),
        assignment,
        parse_prefix(alloc, 'G', Command::G, parse_gcode),
        parse_prefix(alloc, 'M', Command::M, parse_mcode),
        parse_prefix(alloc, 'O', Command::O, parse_ocode),
        parse_prefix(alloc, 'S', Command::S, parse_scode),
        parse_prefix(alloc, 'T', Command::T, parse_tcode),
    )))(input)
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
