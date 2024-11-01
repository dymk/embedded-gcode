use crate::{
    bind,
    gcode::{expression::Expression, Axes, Axis},
    parser::{nom_types::IParseResult, parse_utils::space_before},
    ParserAllocator,
};
use nom::{character::complete::one_of, combinator::map_res, multi::fold_many1, sequence::pair};

pub fn parse_axes<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Axes<'b>> {
    fold_many1(
        bind!(alloc, parse_axis),
        Axes::default,
        |axes, (axis, expr)| axes.set(axis, expr),
    )(input)
}

pub fn parse_axis<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, (Axis, Expression<'b>)> {
    use crate::parser::toplevel::*;

    map_res(
        pair(
            space_before(one_of("XYZABCxyzabc")),
            space_before(bind!(alloc, parse_expression)),
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
