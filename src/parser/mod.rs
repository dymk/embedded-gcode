mod fold_many0_result;
mod nom_alloc;
mod nom_types;
pub mod parse_command;
mod parse_expression;
mod parse_utils;
mod toplevel;

#[cfg(test)]
mod parse_expression_tests;

use crate::gcode::{Axes, Axis};
pub use fold_many0_result::fold_many0_result;
use nom::{
    character::complete::{multispace0, one_of},
    combinator::map_res,
    multi::fold_many1,
    number::complete::float,
    sequence::{preceded, tuple},
};
pub use nom_types::GcodeParseError;
use nom_types::{ok, IParseResult};

fn parse_axes<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Axes> {
    fold_many1(
        preceded(multispace0, parse_axis()),
        Axes::default,
        |axes, (axis, value)| axes.set(axis, value),
    )
}

fn parse_axis<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, (Axis, f32)> {
    map_res(tuple((one_of("XYZABCxyzabc"), float)), |(chr, value)| {
        let axis = match Axis::from_chr(chr.to_ascii_uppercase()) {
            Some(axis) => axis,
            None => return Err(()),
        };
        Ok((axis, value))
    })
}

#[cfg(test)]
mod test {
    extern crate std;

    use super::{parse_axes, parse_command};
    use crate::gcode::{expression::Expression, Axes, Axis, Command, Gcode, Ocode, OcodeStatement};
    use crate::parser::nom_alloc::NomAlloc;
    use bump_into::BumpInto;

    #[rstest::rstest]
    #[case(b"X1", Axes::new().set(Axis::X, 1.0))]
    #[case(b"X1Y2.4", Axes::new().set(Axis::X, 1.0).set(Axis::Y, 2.4))]
    #[case(b"X1 Y2.4 ", Axes::new().set(Axis::X, 1.0).set(Axis::Y, 2.4))]
    #[case(b" X1 Y2.4", Axes::new().set(Axis::X, 1.0).set(Axis::Y, 2.4))]
    #[case(b"Y2.4X1", Axes::new().set(Axis::X, 1.0).set(Axis::Y, 2.4))]
    fn test_parse_axes(#[case] input: &[u8], #[case] expected: Axes) {
        let result = parse_axes()(input).unwrap();
        assert_eq!(result.1, expected);
    }

    #[rstest::rstest]
    #[case(b"G0X1")]
    #[case(b"g0X1")]
    #[case(b" G0X1")]
    #[case(b" G0 X1")]
    #[case(b"G0 X1")]
    #[case(b"G0X1 ")]
    #[case(b"G0   X1")]
    fn test_parser(#[case] input: &[u8]) {
        let mut heap = bump_into::space_uninit!(64);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let result = parse_command::parse_command(alloc)(input).unwrap();
        assert_eq!(
            result.1,
            Command::G(Gcode::G0(Some(Axes::new().set(Axis::X, 1.0))))
        );
    }

    #[rstest::rstest]
    #[case("o100 sub", Ocode::new(100, OcodeStatement::Sub))]
    #[case("o100 endsub", Ocode::new(100, OcodeStatement::EndSub))]
    #[case(
        "o100 if [#2]",
        Ocode::new(100, OcodeStatement::If(Expression::NumberedParam(2)))
    )]
    #[case("o100 endif", Ocode::new(100, OcodeStatement::EndIf))]
    fn test_parse_codes<'a>(#[case] input: &str, #[case] expected_ocode: impl Into<Command<'a>>) {
        let mut heap = bump_into::space_uninit!(64);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let (_, actual_code) = parse_command::parse_command(alloc)(input.as_bytes()).unwrap();
        assert_eq!(actual_code, expected_ocode.into());
    }
}
