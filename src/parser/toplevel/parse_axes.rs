use crate::{
    gcode::{expression::Expression, Axes, Axis},
    parser::{nom_types::IParseResult, parse_utils::space_before},
    GcodeParser,
};
use nom::{character::complete::one_of, combinator::map_res, multi::fold_many1, sequence::pair};

impl GcodeParser for Axes {
    fn parse(input: &[u8]) -> IParseResult<'_, Self> {
        parse_axes(input)
    }
}

impl GcodeParser for (Axis, Expression) {
    fn parse(input: &[u8]) -> IParseResult<'_, Self> {
        parse_axis(input)
    }
}

fn parse_axes(input: &[u8]) -> IParseResult<'_, Axes> {
    fold_many1(
        <(Axis, Expression)>::parse,
        Axes::default,
        |axes, (axis, expr)| axes.set(axis, expr),
    )(input)
}

fn parse_axis(input: &[u8]) -> IParseResult<'_, (Axis, Expression)> {
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
