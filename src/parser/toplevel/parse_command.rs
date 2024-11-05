use crate::{
    gcode::{Command, GcodeParser},
    parser::{nom_types::IParseResult, ok, parse_utils::space_before, toplevel::*},
};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{map_res, peek},
    sequence::preceded,
};

impl GcodeParser for Command {
    fn parse<'a>(input: &'a [u8]) -> IParseResult<'a, Self> {
        parse_command(input)
    }
}

fn parse_command<'a>(input: &'a [u8]) -> IParseResult<'a, Command> {
    let assignment = preceded(space_before(peek(tag("#"))), parse_assignment);

    space_before(alt((
        parse_comment,
        assignment,
        parse_prefix('G', Command::G, parse_gcode),
        parse_prefix('M', Command::M, parse_mcode),
        parse_prefix('O', Command::O, parse_ocode),
        parse_prefix('S', Command::S, parse_scode),
        parse_prefix('T', Command::T, parse_tcode),
    )))(input)
}

fn parse_prefix<'a, 'b, SubCommand>(
    // 'G', etc
    command_char: char,
    // Map the parsed sub-command into a Command e.g. Gcode into Command::G(Gcode)
    command_ctor: impl Fn(SubCommand) -> Command,
    // The parser for the sub-command, results in a Gcode, Mcode, etc
    command_parser: fn(&'a [u8]) -> IParseResult<'a, SubCommand>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Command> {
    map_res(
        preceded(
            space_before(tag_no_case([command_char as u8])),
            command_parser,
        ),
        move |parsed| ok(command_ctor(parsed)),
    )
}
