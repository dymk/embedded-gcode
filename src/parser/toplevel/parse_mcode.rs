use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    combinator::{fail, map_res, opt},
    sequence::{pair, preceded},
};

use crate::{
    gcode::{Mcode, Tcode},
    parser::{
        nom_types::IParseResult,
        ok,
        parse_utils::{number_code, space_before},
    },
    GcodeParser,
};

impl GcodeParser for Mcode {
    fn parse(input: &[u8]) -> IParseResult<'_, Self> {
        parse_mcode(input)
    }
}

fn parse_mcode(input: &[u8]) -> IParseResult<'_, Mcode> {
    let parse_tcode_prefixed = preceded(space_before(tag_no_case("T")), Tcode::parse);

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
