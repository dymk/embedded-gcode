use crate::{
    gcode::Gcode,
    parser::{nom_types::IParseResult, parse_axes},
    NomAlloc,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map_res, opt},
    sequence::preceded,
};

pub fn parse_gcode<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Gcode<'b>> {
    fn make_g<'b, A>(ctor: impl Fn(A) -> Gcode<'b>) -> impl Fn(A) -> Result<Gcode<'b>, ()> {
        move |axes| Ok(ctor(axes))
    }

    alt((
        map_res(
            preceded(tag("0"), opt(parse_axes(alloc))),
            make_g(Gcode::G0),
        ),
        map_res(preceded(tag("1"), parse_axes(alloc)), make_g(Gcode::G1)),
    ))
}
