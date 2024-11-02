use crate::{
    bind,
    gcode::expression::{NamedGlobalParam, NamedLocalParam, NumberedParam, Param},
    parser::{map_res_f1, nom_types::IParseResult, ok, parse_u32, space_before},
    ParserAllocator,
};
use core::str::from_utf8;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1},
    combinator::{map_res, peek, recognize},
    multi::many0_count,
    sequence::{delimited, pair, preceded, tuple},
    Parser as _,
};

pub fn parse_param<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Param<'b>> {
    let param = alt((
        // global named parameter e.g. `#<_foo>`
        map_res_f1(
            delimited(
                tuple((tag("#<"), peek(tag("_")))),
                bind!(alloc, parse_named_global_param),
                tag(">"),
            ),
            Param::NamedGlobal,
        ),
        // local named parameter e.g. `#<foo>`
        map_res_f1(
            delimited(tag("#<"), bind!(alloc, parse_named_local_param), tag(">")),
            Param::NamedLocal,
        ),
        // numbered parameter e.g. `#5`
        map_res_f1(
            preceded(tag("#"), bind!(alloc, parse_numbered_param)),
            Param::Numbered,
        ),
    ));

    space_before(param)(input)
}

pub fn parse_name<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, &'b str> {
    map_res(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
        move |bytes| {
            let name_str = from_utf8(bytes)?;
            ok(alloc.alloc_str(name_str)?)
        },
    )(input)
}

fn parse_named_local_param<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, NamedLocalParam<'b>> {
    map_res_f1(bind!(alloc, parse_name), NamedLocalParam).parse(input)
}

fn parse_named_global_param<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, NamedGlobalParam<'b>> {
    map_res_f1(bind!(alloc, parse_name), NamedGlobalParam).parse(input)
}

fn parse_numbered_param<'a, 'b>(
    _: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, NumberedParam> {
    map_res_f1(parse_u32(), NumberedParam).parse(input)
}
