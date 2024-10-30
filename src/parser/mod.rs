mod fold_many0_result;
mod nom_alloc;
mod nom_types;
pub mod parse_command;
mod parse_expression;
mod parse_utils;
mod toplevel;

#[cfg(test)]
mod parse_expression_tests;

#[cfg(test)]
mod parse_command_tests;

use crate::gcode::{Axes, Axis};
pub use fold_many0_result::fold_many0_result;
use nom::{
    character::complete::one_of, combinator::map_res, multi::fold_many1, number::complete::float,
    sequence::pair,
};
pub use nom_types::GcodeParseError;
use nom_types::{ok, IParseResult};
use parse_utils::space_before;

fn parse_axes<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Axes> {
    fold_many1(parse_axis(), Axes::default, |axes, (axis, value)| {
        axes.set(axis, value)
    })
}

fn parse_axis<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, (Axis, f32)> {
    map_res(
        pair(space_before(one_of("XYZABCxyzabc")), space_before(float)),
        |(chr, value)| {
            let axis = match Axis::from_chr(chr) {
                Some(axis) => axis,
                None => return Err(()),
            };
            Ok((axis, value))
        },
    )
}
