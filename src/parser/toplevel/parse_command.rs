use crate::{
    gcode::{Command, Gcode, Mcode, Ocode, Scode, Tcode},
    parser::{nom_types::IParseResult, ok, parse_utils::space_before, toplevel::*, Input},
    GcodeParser,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map_res, peek},
    sequence::preceded,
};

impl GcodeParser for Command {
    fn parse(input: Input) -> IParseResult<Self> {
        fn command<'a, SubCommand>(
            // Map the parsed sub-command into a Command e.g. Gcode into Command::G(Gcode)
            command_ctor: impl Fn(SubCommand) -> Command,
            // The parser for the sub-command, results in a Gcode, Mcode, etc
            command_parser: fn(Input) -> IParseResult<SubCommand>,
        ) -> impl FnMut(Input<'a>) -> IParseResult<'a, Command> {
            map_res(command_parser, move |parsed| ok(command_ctor(parsed)))
        }

        space_before(alt((
            parse_comment,
            preceded(space_before(peek(tag("#"))), parse_assignment),
            command(Command::G, Gcode::parse),
            command(Command::M, Mcode::parse),
            command(Command::O, Ocode::parse),
            command(Command::S, Scode::parse),
            command(Command::T, Tcode::parse),
        )))(input)
    }
}
