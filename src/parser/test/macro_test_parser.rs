extern crate std;

use crate::{
    gcode::{expression::Expression, Axes, Command, Gcode},
    parser::{test::permute_whitespace, toplevel::*},
    GcodeParseError, ParserAllocator,
};

use super::ExprBuilder;

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

macro_rules! test_parser_impl {
    ($impl_func_name:ident, $parser_func_name:ident, $node_type:ty) => {
        #[track_caller]
        pub fn $impl_func_name<NodeBuilder>(tokens: &[&str], mut node_builder: NodeBuilder)
        where
            NodeBuilder: for<'i> FnMut(&'i ExprBuilder<'i>) -> $node_type,
        {
            for input in permute_whitespace(tokens) {
                let mut heap = bump_into::space_uninit!(1024);
                let alloc = ParserAllocator::new(&mut heap);
                let expr_builder = ExprBuilder::new(&alloc);
                let expected = node_builder(&expr_builder);
                let (rest, actual) = match $parser_func_name(&alloc, input.as_bytes()) {
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
                assert_eq!(
                    expected.clone(),
                    actual.clone(),
                    "[rest `{}`]",
                    from_utf8(rest)
                );
                assert!(
                    rest.iter().all(|b| b.is_ascii_whitespace()),
                    "[rest `{}`]",
                    from_utf8(rest)
                );
            }
        }
    };
}

test_parser_impl!(test_expr_impl, parse_expression, &'i Expression<'i>);
test_parser_impl!(test_command_impl, parse_command, Command<'i>);
test_parser_impl!(test_axes_impl, parse_axes, Axes<'i>);

fn from_utf8(input: &[u8]) -> &str {
    std::str::from_utf8(input).unwrap()
}

test_parser!(expr, atan, ["ATAN[1.0]/[2.0]"], |b: &ExprBuilder<'_>| {
    b.atan(b.lit(1.0), b.lit(2.0))
});

test_parser!(command, g0, ["G0"], |_| { Gcode::G0(None).into() });
