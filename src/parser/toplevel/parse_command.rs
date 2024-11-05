use crate::{
    gcode::{Command, Gcode, Mcode, Ocode, Scode, Tcode},
    parser::{nom_types::IParseResult, ok, parse_utils::space_before, toplevel::*},
    GcodeParser,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{map_res, peek},
    sequence::preceded,
};

impl GcodeParser for Command {
    fn parse(input: &[u8]) -> IParseResult<'_, Self> {
        parse_command(input)
    }
}

fn parse_command(input: &[u8]) -> IParseResult<'_, Command> {
    let assignment = preceded(space_before(peek(tag("#"))), parse_assignment);

    space_before(alt((
        parse_comment,
        assignment,
        parse_prefix('G', Command::G, Gcode::parse),
        parse_prefix('M', Command::M, Mcode::parse),
        parse_prefix('O', Command::O, Ocode::parse),
        parse_prefix('S', Command::S, Scode::parse),
        parse_prefix('T', Command::T, Tcode::parse),
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
