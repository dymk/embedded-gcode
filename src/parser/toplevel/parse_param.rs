use crate::{
    bind,
    gcode::expression::{NamedGlobalParam, NamedLocalParam, NumberedParam, Param},
    parser::{err, nom_types::IParseResult, ok, parse_u32, space_before},
    ParserAllocator,
};
use core::str::from_utf8;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::map_res,
    error::{Error, ErrorKind},
    sequence::{delimited, preceded},
    Parser as _,
};

pub fn parse_param<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Param<'b>> {
    space_before(alt((
        // named parameter, global or local
        delimited(tag("#<"), bind!(alloc, parse_named_param), tag(">")),
        // numbered parameter e.g. `#5`
        preceded(tag("#"), bind!(alloc, parse_numbered_param)),
    )))(input)
}

fn parse_named_param<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Param<'b>> {
    map_res(bind!(alloc, parse_name), |name| {
        ok(if name.starts_with('_') {
            Param::NamedGlobal(NamedGlobalParam(name))
        } else {
            Param::NamedLocal(NamedLocalParam(name))
        })
    })
    .parse(input)
}

fn parse_numbered_param<'a, 'b>(
    _: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Param<'b>> {
    map_res(parse_u32(), |value| {
        ok(Param::Numbered(NumberedParam(value)))
    })
    .parse(input)
}

fn parse_name<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, &'b str> {
    map_res(take_while(|b| b != b'>'), move |bytes: &'a [u8]| {
        // count number of non-space characters
        let num_non_space = bytes.iter().filter(|c| !c.is_ascii_whitespace()).count();
        let name_str = alloc.alloc_str_space(num_non_space)?;
        let mut idx = 0;
        for c in bytes.iter() {
            if !c.is_ascii_alphanumeric() && !c == b'_' {
                return err(Error::new(bytes, ErrorKind::Alpha));
            }

            if !c.is_ascii_whitespace() {
                name_str[idx] = c.to_ascii_lowercase();
                idx += 1;
            }
        }
        ok(from_utf8(name_str)?)
    })(input)
}
