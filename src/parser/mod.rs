mod fold_many0_result;
pub mod nom_alloc;
mod nom_types;
pub mod parse_command;
mod parse_expression;
mod parse_utils;
mod toplevel;

#[cfg(test)]
mod test;

use crate::{
    bind,
    gcode::{expression::Expression, Axes, Axis},
    NomAlloc,
};
pub use fold_many0_result::fold_many0_result;
use nom::{character::complete::one_of, combinator::map_res, multi::fold_many1, sequence::pair};
pub use nom_types::GcodeParseError;
use nom_types::{ok, IParseResult};
use parse_utils::space_before;

pub struct Parser<'b> {
    alloc: NomAlloc<'b>,
}

impl<'b> Parser<'b> {
    fn new(alloc: NomAlloc<'b>) -> Self {
        Self { alloc }
    }

    fn parse_axes<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Axes<'b>> {
        fold_many1(
            bind(self, Self::parse_axis),
            Axes::default,
            |axes, (axis, value)| axes.set(axis, value),
        )(input)
    }

    fn parse_axis<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, (Axis, Expression<'b>)> {
        map_res(
            pair(
                space_before(one_of("XYZABCxyzabc")),
                space_before(bind(self, Self::parse_expression)),
            ),
            |(chr, value)| {
                let axis = match Axis::from_chr(chr) {
                    Some(axis) => axis,
                    None => return Err(()),
                };
                Ok((axis, value))
            },
        )(input)
    }
}
