use crate::{
    gcode::Gcode,
    parser::{bind, nom_types::IParseResult},
    Parser,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    combinator::{map_res, opt},
    sequence::preceded,
};

impl<'b> Parser<'b> {
    pub fn parse_gcode<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Gcode<'b>> {
        fn make_g<'b, A>(ctor: impl Fn(A) -> Gcode<'b>) -> impl Fn(A) -> Result<Gcode<'b>, ()> {
            move |axes| Ok(ctor(axes))
        }

        alt((
            map_res(
                preceded(tag("0"), opt(bind(self, Self::parse_axes))),
                make_g(Gcode::G0),
            ),
            map_res(
                preceded(tag("1"), bind(self, Self::parse_axes)),
                make_g(Gcode::G1),
            ),
        ))(input)
    }
}
