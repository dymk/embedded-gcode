use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::space0,
    combinator::map_res,
    sequence::{preceded, tuple},
};

use crate::{
    gcode::{Ocode, OcodeStatement},
    parser::{
        nom_types::{ok, IParseResult},
        parse_utils::{parse_u32, space_before},
    },
    Parser,
};

impl<'b> Parser<'b> {
    pub fn parse_ocode<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Ocode<'b>> {
        map_res(
            tuple((
                parse_u32(),
                space_before(alt((
                    map_res(tag_no_case("sub"), |_| ok(OcodeStatement::Sub)),
                    map_res(tag_no_case("endsub"), |_| ok(OcodeStatement::EndSub)),
                    preceded(
                        tuple((tag_no_case("if"), space0)),
                        map_res(
                            |i| self.parse_expression(i),
                            |expr| ok(OcodeStatement::If(expr)),
                        ),
                    ),
                    map_res(tag_no_case("endif"), |_| ok(OcodeStatement::EndIf)),
                ))),
            )),
            |(id, stmt)| ok(Ocode::new(id, stmt)),
        )(input)
    }
}
