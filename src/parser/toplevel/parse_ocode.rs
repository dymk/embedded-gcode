use crate::{
    gcode::{Ocode, OcodeStatement},
    parser::{
        bind, map_res_f1,
        nom_types::IParseResult,
        ok,
        parse_utils::{parse_u32, space_before},
        toplevel::*,
    },
    ParserAllocator,
};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    combinator::map_res,
    sequence::{preceded, tuple},
};

pub fn parse_ocode<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Ocode<'b>> {
    map_res(
        tuple((
            parse_u32(),
            space_before(alt((
                map_res(tag_no_case("sub"), |_| ok(OcodeStatement::Sub)),
                map_res(tag_no_case("endsub"), |_| ok(OcodeStatement::EndSub)),
                preceded(
                    tag_no_case("if"),
                    map_res_f1(bind!(alloc, parse_expression), OcodeStatement::If),
                ),
                map_res(tag_no_case("endif"), |_| ok(OcodeStatement::EndIf)),
            ))),
        )),
        |(id, stmt)| ok(Ocode::new(id, stmt)),
    )(input)
}
