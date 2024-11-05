use crate::{
    gcode::{expression::Expression, Axes, Axis, GcodeParser},
    parser::{nom_types::IParseResult, parse_utils::space_before},
};
use nom::{character::complete::one_of, combinator::map_res, multi::fold_many1, sequence::pair};

impl GcodeParser for Axes {
    fn parse<'i>(input: &'i [u8]) -> IParseResult<'i, Self> {
        parse_axes(input)
    }
}

impl GcodeParser for (Axis, Expression) {
    fn parse<'i>(input: &'i [u8]) -> IParseResult<'i, Self> {
        parse_axis(input)
    }
}

fn parse_axes<'i>(input: &'i [u8]) -> IParseResult<'i, Axes> {
    fold_many1(
        <(Axis, Expression)>::parse,
        Axes::default,
        |axes, (axis, expr)| axes.set(axis, expr),
    )(input)
}

fn parse_axis<'i>(input: &'i [u8]) -> IParseResult<'i, (Axis, Expression)> {
    map_res(
        pair(
            space_before(one_of("XYZABCxyzabc")),
            space_before(Expression::parse),
        ),
        |(chr, expr)| {
            let axis = match Axis::from_chr(chr) {
                Some(axis) => axis,
                None => return Err(()),
            };
            Ok((axis, expr))
        },
    )(input)
}
