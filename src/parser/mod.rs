pub mod parse_command;
mod parse_expression;

use crate::gcode::{Axes, Axis, Command, Gcode, Mcode, Ocode, OcodeStatement, Scode, Tcode};
use bump_into::BumpInto;
use core::str::from_utf8;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until1},
    character::complete::{digit1, multispace0, one_of},
    combinator::{fail, map_res, opt},
    error::Error,
    multi::fold_many1,
    number::complete::float,
    sequence::{delimited, preceded, tuple},
    IResult,
};
use parse_expression::parse_expression;

type IParseResult<'a, O> = IResult<&'a [u8], O, Error<&'a [u8]>>;

fn ok<'a, T>(t: T) -> Result<T, Error<&'a [u8]>> {
    Ok(t)
}

fn parse_comment<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Command<'b>> {
    map_res(
        delimited(tag("("), take_until1(")"), tag(")")),
        |bytes: &'a [u8]| {
            let comment_str = bump
                .alloc_copy_concat_strs(&[from_utf8(bytes).unwrap()])
                .unwrap();
            ok(Command::Comment(comment_str))
        },
    )
}

fn parse_gcode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Gcode> {
    fn make_g<A>(ctor: impl Fn(A) -> Gcode) -> impl Fn(A) -> Result<Gcode, ()> {
        move |axes| Ok(ctor(axes))
    }

    alt((
        map_res(preceded(tag("0"), opt(parse_axes())), make_g(Gcode::G0)),
        map_res(preceded(tag("1"), parse_axes()), make_g(Gcode::G1)),
    ))
}

fn parse_ocode<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Ocode<'b>> {
    map_res(
        tuple((
            parse_u32(),
            preceded(
                multispace0,
                alt((
                    map_res(tag_no_case("sub"), |_| ok(OcodeStatement::Sub)),
                    map_res(tag_no_case("endsub"), |_| ok(OcodeStatement::EndSub)),
                    preceded(
                        tuple((tag_no_case("if"), multispace0)),
                        map_res(parse_expression(bump), |expr| ok(OcodeStatement::If(expr))),
                    ),
                    map_res(tag_no_case("endif"), |_| ok(OcodeStatement::EndIf)),
                )),
            ),
        )),
        |(id, stmt)| ok(Ocode::new(id, stmt)),
    )
}

fn parse_scode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Scode> {
    fail
}

fn parse_tcode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Tcode> {
    fail
}

fn parse_u32<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, u32> {
    map_res(digit1, |bytes| {
        str::parse(match from_utf8(bytes) {
            Ok(s) => s,
            Err(_) => "invalid",
        })
    })
}

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

fn parse_mcode<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Mcode> {
    fail
}

#[cfg(test)]
mod test {

    extern crate std;
    use bump_into::BumpInto;

    use super::{parse_axes, parse_command};
    use crate::gcode::{expression::Expression, Axes, Axis, Command, Gcode, Ocode, OcodeStatement};

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
        let result = parse_command::parse_command(&bump)(input).unwrap();
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
        let (_, actual_code) = parse_command::parse_command(&bump)(input.as_bytes()).unwrap();
        assert_eq!(actual_code, expected_ocode.into());
    }
}
