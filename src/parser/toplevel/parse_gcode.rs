use crate::parser::nom_types::ok;
use crate::parser::parse_utils::number_code;
use crate::parser::toplevel::parse_axes;
use crate::{bind, gcode::Gcode, parser::nom_types::IParseResult, ParserAllocator};
use nom::{
    branch::alt,
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
            preceded(number_code("0"), opt(bind!(alloc, parse_axes))),
            make_g(Gcode::G0),
        ),
        map_res(
            preceded(number_code("1"), bind!(alloc, parse_axes)),
            make_g(Gcode::G1),
        ),
        map_res(number_code("20"), |_| ok(Gcode::G20)),
        map_res(number_code("21"), |_| ok(Gcode::G21)),
        map_res(number_code("53"), |_| ok(Gcode::G53)),
        map_res(number_code("54"), |_| ok(Gcode::G54)),
        map_res(number_code("90"), |_| ok(Gcode::G90)),
        map_res(number_code("91"), |_| ok(Gcode::G91)),
    ))(input)
}
