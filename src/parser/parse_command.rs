use super::IParseResult;
use crate::{
    gcode::Command,
    parser::{ok, parse_utils::space_before},
    Parser,
};
use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::space0, combinator::map_res,
    sequence::preceded,
};

impl<'b> Parser<'b> {
    pub fn parse_command<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Command<'b>> {
        preceded(
            space0,
            alt((
                |input| self.parse_comment(input),
                self.parse_prefix('G', Command::G, Self::parse_gcode),
                self.parse_prefix('M', Command::M, Self::parse_mcode),
                self.parse_prefix('O', Command::O, Self::parse_ocode),
                self.parse_prefix('S', Command::S, Self::parse_scode),
                self.parse_prefix('T', Command::T, Self::parse_tcode),
            )),
        )(input)
    }

    fn parse_prefix<'a, SubCommand>(
        &'b self,
        // 'G', etc
        command_char: char,
        // Map the parsed sub-command into a Command e.g. Gcode into Command::G(Gcode)
        command_ctor: impl Fn(SubCommand) -> Command<'b>,
        // The parser for the sub-command, results in a Gcode, Mcode, etc
        mut command_parser: impl FnMut(&'b Parser<'b>, &'a [u8]) -> IParseResult<'a, SubCommand>,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Command<'b>> {
        map_res(
            preceded(
                space_before(tag_no_case([command_char as u8])),
                move |input| command_parser(self, input),
            ),
            move |parsed| ok(command_ctor(parsed)),
        )
    }
}
