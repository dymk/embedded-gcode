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
        bind,
        nom_types::{ok, IParseResult},
        parse_utils::{parse_u32, space_before},
        toplevel::*,
    },
    NomAlloc,
};

pub fn parse_ocode<'a, 'b>(alloc: NomAlloc<'b>, input: &'a [u8]) -> IParseResult<'a, Ocode<'b>> {
    map_res(
        tuple((
            parse_u32(),
            space_before(alt((
                map_res(tag_no_case("sub"), |_| ok(OcodeStatement::Sub)),
                map_res(tag_no_case("endsub"), |_| ok(OcodeStatement::EndSub)),
                preceded(
                    tuple((tag_no_case("if"), space0)),
                    map_res(bind!(alloc, parse_expression), |expr| {
                        ok(OcodeStatement::If(expr))
                    }),
                ),
                map_res(tag_no_case("endif"), |_| ok(OcodeStatement::EndIf)),
            ))),
        )),
        |(id, stmt)| ok(Ocode::new(id, stmt)),
    )(input)
}
