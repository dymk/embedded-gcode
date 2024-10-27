use super::IParseResult;
use crate::{
    gcode::Command,
    parser::{ok, parse_comment, parse_gcode, parse_mcode, parse_ocode, parse_scode, parse_tcode},
};
use bump_into::BumpInto;
use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::multispace0,
    combinator::map_res, sequence::preceded,
};

pub fn parse_command<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Command<'b>> {
    fn parse_command<'a, 'b, SubCommand>(
        // 'G', etc
        command_char: char,
        // Map the parsed sub-command into a Command e.g. Gcode into Command::G(Gcode)
        command_ctor: impl Fn(SubCommand) -> Command<'b>,
        // The parser for the sub-command, results in a Gcode, Mcode, etc
        command_parser: impl FnMut(&'a [u8]) -> IParseResult<'a, SubCommand>,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Command<'b>> {
        map_res(
            preceded(tag_no_case([command_char as u8]), command_parser),
            move |parsed| ok(command_ctor(parsed)),
        )
    }

    preceded(
        multispace0,
        alt((
            parse_comment(bump),
            parse_command('G', Command::G, parse_gcode),
            parse_command('M', Command::M, parse_mcode),
            parse_command('O', Command::O, parse_ocode(bump)),
            parse_command('S', Command::S, parse_scode),
            parse_command('T', Command::T, parse_tcode),
        )),
    )
}
