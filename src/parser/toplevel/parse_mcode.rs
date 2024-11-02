use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    combinator::{fail, map_res, opt},
    sequence::{pair, preceded},
};

use crate::{
    bind,
    gcode::Mcode,
    parser::{
        nom_types::IParseResult,
        ok,
        parse_utils::{number_code, space_before},
    },
    ParserAllocator,
};

use super::parse_tcode::parse_tcode;

pub fn parse_mcode<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Mcode> {
    let parse_tcode_prefixed = preceded(space_before(tag_no_case("T")), bind!(alloc, parse_tcode));

    let parse_m6 = map_res(
        pair(number_code("6"), opt(parse_tcode_prefixed)),
        |(_, opt_tcode)| ok(Mcode::M6(opt_tcode)),
    );

    alt((
        map_res(number_code("3"), |_| ok(Mcode::M3)),
        map_res(number_code("4"), |_| ok(Mcode::M4)),
        map_res(number_code("5"), |_| ok(Mcode::M5)),
        parse_m6,
        map_res(number_code("7"), |_| ok(Mcode::M7)),
        map_res(number_code("8"), |_| ok(Mcode::M8)),
        map_res(number_code("9"), |_| ok(Mcode::M9)),
        fail,
    ))(input)
}
