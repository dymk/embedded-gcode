use crate::{
    bind,
    gcode::{expression::Expression, Axes, Axis, GcodeParser},
    parser::{nom_types::IParseResult, parse_utils::space_before},
    ParserAllocator,
};
use nom::{character::complete::one_of, combinator::map_res, multi::fold_many1, sequence::pair};

impl<'a, 'b> GcodeParser<'a, 'b> for Axes<'b> {
    fn parse(alloc: &'b ParserAllocator<'b>, input: &'a [u8]) -> IParseResult<'a, Self> {
        parse_axes(alloc, input)
    }
}

impl<'a, 'b> GcodeParser<'a, 'b> for (Axis, Expression<'b>) {
    fn parse(alloc: &'b ParserAllocator<'b>, input: &'a [u8]) -> IParseResult<'a, Self> {
        parse_axis(alloc, input)
    }
}

fn parse_axes<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Axes<'b>> {
    fold_many1(
        bind!(alloc, <(Axis, Expression)>::parse),
        Axes::default,
        |axes, (axis, expr)| axes.set(axis, expr),
    )(input)
}

fn parse_axis<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, (Axis, Expression<'b>)> {
    map_res(
        pair(
            space_before(one_of("XYZABCxyzabc")),
            space_before(bind!(alloc, Expression::parse)),
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
