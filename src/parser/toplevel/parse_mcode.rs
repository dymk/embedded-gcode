use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    combinator::{fail, map_res, opt},
    sequence::{pair, preceded},
};

use crate::{
    bind,
    gcode::Mcode,
    parser::{
        nom_types::{ok, IParseResult},
        parse_utils::space_before,
    },
    NomAlloc,
};

use super::parse_tcode::parse_tcode;

pub fn parse_mcode<'a, 'b>(alloc: NomAlloc<'b>, input: &'a [u8]) -> IParseResult<'a, Mcode> {
    let parse_tcode_prefixed = preceded(space_before(tag_no_case("T")), bind!(alloc, parse_tcode));

    let parse_m6 = map_res(
        pair(tag("6"), opt(parse_tcode_prefixed)),
        |(_, opt_tcode)| ok(Mcode::M6(opt_tcode)),
    );

    alt((
        map_res(tag("3"), |_| ok(Mcode::M3)),
        map_res(tag("4"), |_| ok(Mcode::M4)),
        map_res(tag("5"), |_| ok(Mcode::M5)),
        parse_m6,
        map_res(tag("7"), |_| ok(Mcode::M7)),
        map_res(tag("8"), |_| ok(Mcode::M8)),
        map_res(tag("9"), |_| ok(Mcode::M9)),
        fail,
    ))(input)
}
