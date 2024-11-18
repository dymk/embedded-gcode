use crate::{
    gcode::expression::{Expression, NamedParam, NumberedParam, Param},
    parser::{err, map_res_into, nom_types::IParseResult, ok, parse_u32, space_before, Input},
    GcodeParser,
};
use alloc::string::String;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    combinator::map_res,
    error::{Error, ErrorKind},
    sequence::{delimited, preceded, tuple},
    Parser as _,
};

impl GcodeParser for Param {
    fn parse(input: Input) -> IParseResult<Self> {
        space_before(alt((
            map_res_into(NamedParam::parse),
            map_res_into(NumberedParam::parse),
        )))(input)
    }
}

/// named parameter, global or local
impl GcodeParser for NamedParam {
    fn parse(input: Input) -> IParseResult<Self> {
        map_res(
            delimited(
                tuple((space_before(tag("#")), space_before(tag("<")))),
                parse_name,
                space_before(tag(">")),
            ),
            |name| {
                ok(if name.starts_with('_') {
                    NamedParam::named_global(name)
                } else {
                    NamedParam::named_local(name)
                })
            },
        )
        .parse(input)
    }
}

/// numbered parameter e.g. `#5`
impl GcodeParser for NumberedParam {
    fn parse(input: Input) -> IParseResult<Self> {
        preceded(
            space_before(tag("#")),
            space_before(alt((
                map_res(parse_u32(), |value| ok(NumberedParam::numbered(value))),
                map_res(Expression::parse, |value| ok(NumberedParam::expr(value))),
            ))),
        )
        .parse(input)
    }
}

fn parse_name(input: Input) -> IParseResult<String> {
    map_res(take_while(|b| b != b'>'), move |bytes: Input| {
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
