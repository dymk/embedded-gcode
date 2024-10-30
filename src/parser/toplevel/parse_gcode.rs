use crate::{
    gcode::Gcode,
    parser::{nom_types::IParseResult, parse_axes},
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map_res, opt},
    sequence::preceded,
};

pub fn parse_gcode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Gcode> {
    fn make_g<A>(ctor: impl Fn(A) -> Gcode) -> impl Fn(A) -> Result<Gcode, ()> {
        move |axes| Ok(ctor(axes))
    }

    alt((
        map_res(preceded(tag("0"), opt(parse_axes())), make_g(Gcode::G0)),
        map_res(preceded(tag("1"), parse_axes()), make_g(Gcode::G1)),
    ))
}
