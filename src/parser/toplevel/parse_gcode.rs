use crate::parser::toplevel::parse_axes;
use crate::{bind, gcode::Gcode, parser::nom_types::IParseResult, ParserAllocator};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map_res, opt},
    sequence::preceded,
};

pub fn parse_gcode<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Gcode<'b>> {
    fn make_g<'b, A>(ctor: impl Fn(A) -> Gcode<'b>) -> impl Fn(A) -> Result<Gcode<'b>, ()> {
        move |axes| Ok(ctor(axes))
    }

    alt((
        map_res(
            preceded(tag("0"), opt(bind!(alloc, parse_axes))),
            make_g(Gcode::G0),
        ),
        map_res(
            preceded(tag("1"), bind!(alloc, parse_axes)),
            make_g(Gcode::G1),
        ),
    ))(input)
}
