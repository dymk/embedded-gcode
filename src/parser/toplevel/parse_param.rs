use crate::{
    gcode::{
        expression::{NamedGlobalParam, NamedLocalParam, NumberedParam, Param},
        GcodeParser,
    },
    parser::{err, nom_types::IParseResult, ok, parse_u32, space_before},
};
use alloc::string::{String, ToString as _};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::map_res,
    error::{Error, ErrorKind},
    sequence::{delimited, preceded},
    Parser as _,
};

impl GcodeParser for Param {
    fn parse<'i>(input: &'i [u8]) -> IParseResult<'i, Self> {
        parse_param(input)
    }
}

fn parse_param<'i>(input: &'i [u8]) -> IParseResult<'i, Param> {
    space_before(alt((
        // named parameter, global or local
        delimited(tag("#<"), parse_named_param, tag(">")),
        // numbered parameter e.g. `#5`
        preceded(tag("#"), parse_numbered_param),
    )))(input)
}

fn parse_named_param<'i>(input: &'i [u8]) -> IParseResult<'i, Param> {
    map_res(parse_name, |name| {
        ok(if name.starts_with('_') {
            Param::NamedGlobal(NamedGlobalParam(name.to_string()))
        } else {
            Param::NamedLocal(NamedLocalParam(name.to_string()))
        })
    })
    .parse(input)
}

fn parse_numbered_param<'i>(input: &'i [u8]) -> IParseResult<'i, Param> {
    map_res(parse_u32(), |value| {
        ok(Param::Numbered(NumberedParam(value)))
    })
    .parse(input)
}

fn parse_name<'i>(input: &'i [u8]) -> IParseResult<'i, String> {
    map_res(take_while(|b| b != b'>'), move |bytes: &'i [u8]| {
        // count number of non-space characters
        let num_non_space = bytes.iter().filter(|c| !c.is_ascii_whitespace()).count();
        let mut string = String::with_capacity(num_non_space);

        for c in bytes.iter() {
            if !c.is_ascii_alphanumeric() && !c == b'_' {
                return err(Error::new(bytes, ErrorKind::Alpha));
            }

            if !c.is_ascii_whitespace() {
                string.push(c.to_ascii_lowercase() as char);
            }
        }

        ok(string)
    })(input)
}
