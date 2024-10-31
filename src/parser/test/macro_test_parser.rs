extern crate std;

use crate::{
    gcode::{
        expression::{ExprBuilder, Expression},
        Axes, Command, Gcode,
    },
    parse_command,
    parser::{parse_axes, parse_expression::parse_expression, test::permute_whitespace},
    GcodeParseError, NomAlloc,
};
use bump_into::BumpInto;

#[macro_export]
macro_rules! test_parser {
    (expr, $test_name:ident, $input:expr, $builder:expr $(,)?) => {
        paste::paste! {
            #[test]
            fn [<test_parse_expr_ $test_name>]() {
                $crate::parser::test::macro_test_parser::test_expr_impl(&$input, $builder);
            }
        }
    };
    (command, $test_name:ident, $input:expr, $builder:expr) => {
        paste::paste! {
            #[test]
            fn [<test_parse_command_ $test_name>]() {
                $crate::parser::test::macro_test_parser::test_command_impl(
                    &$input,
                    $builder
                );
            }
        }
    };
    (axes, $test_name:ident, $input:expr, $builder:expr) => {
        paste::paste! {
            #[test]
            fn [<test_parse_axes_ $test_name>]() {
                $crate::parser::test::macro_test_parser::test_axes_impl(&$input, $builder);
            }
        }
    };
}

#[track_caller]
pub fn test_expr_impl<NodeBuilder>(tokens: &[&str], mut node_builder: NodeBuilder)
where
    NodeBuilder: for<'i> FnMut(&'i ExprBuilder<'i>) -> &'i Expression<'i>,
{
    for input in permute_whitespace(tokens) {
        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let expr_builder = ExprBuilder::new(alloc);
        let expected = node_builder(&expr_builder);
        let (rest, actual) = match parse_expression(alloc)(input.as_bytes()) {
            Ok((rest, actual)) => (rest, actual),
            Err(nom::Err::Error(GcodeParseError::NomError(err))) => {
                panic!(
                    "[input `{}`] [code {:?}] [rest: `{}`]",
                    input,
                    err.code,
                    from_utf8(err.input)
                )
            }
            Err(err) => panic!("{:?}", err),
        };
        assert_eq!(expected, &actual, "[rest `{}`]", from_utf8(rest));
        assert!(
            rest.iter().all(|b| b.is_ascii_whitespace()),
            "[rest `{}`]",
            from_utf8(rest)
        );
    }
}

#[track_caller]
pub fn test_command_impl<'a, 'b, NodeBuilder>(tokens: &[&str], node_builder: NodeBuilder)
where
    NodeBuilder: for<'i> FnMut(&'i ExprBuilder<'i>) -> Command<'i>,
{
    let mut node_builder: NodeBuilder = node_builder.into();
    for input in permute_whitespace(tokens) {
        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let expr_builder = ExprBuilder::new(alloc);
        let expected = node_builder(&expr_builder);
        let (rest, actual) = match parse_command(alloc)(input.as_bytes()) {
            Ok((rest, actual)) => (rest, actual),
            Err(nom::Err::Error(GcodeParseError::NomError(err))) => {
                panic!(
                    "[input `{}`] [code {:?}] [rest: `{}`]",
                    input,
                    err.code,
                    from_utf8(err.input)
                )
            }
            Err(err) => panic!("{:?}", err),
        };
        assert_eq!(expected, actual, "[rest `{}`]", from_utf8(rest));
        assert!(
            rest.iter().all(|b| b.is_ascii_whitespace()),
            "[rest `{}`]",
            from_utf8(rest)
        );
    }
}

#[track_caller]
pub fn test_axes_impl<NodeBuilder>(tokens: &[&str], mut node_builder: NodeBuilder)
where
    NodeBuilder: for<'i> FnMut(&'i ExprBuilder<'i>) -> Axes<'i>,
{
    for input in permute_whitespace(tokens) {
        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let expr_builder = ExprBuilder::new(alloc);
        let expected = node_builder(&expr_builder);
        let (rest, actual) = match parse_axes(alloc)(input.as_bytes()) {
            Ok((rest, actual)) => (rest, actual),
            Err(nom::Err::Error(GcodeParseError::NomError(err))) => {
                panic!(
                    "[input `{}`] [code {:?}] [rest: `{}`]",
                    input,
                    err.code,
                    from_utf8(err.input)
                )
            }
            Err(err) => panic!("{:?}", err),
        };
        assert_eq!(expected, actual, "[rest `{}`]", from_utf8(rest));
        assert!(
            rest.iter().all(|b| b.is_ascii_whitespace()),
            "[rest `{}`]",
            from_utf8(rest)
        );
    }
}

fn from_utf8(input: &[u8]) -> &str {
    std::str::from_utf8(input).unwrap()
}

test_parser!(expr, atan, ["ATAN[1.0]/[2.0]"], |b: &ExprBuilder<'_>| {
    b.atan(b.lit(1.0), b.lit(2.0))
});

test_parser!(command, g0, ["G0"], |_| { Gcode::G0(None).into() });
