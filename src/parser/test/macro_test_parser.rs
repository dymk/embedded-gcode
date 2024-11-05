extern crate std;

use crate::{
    gcode::{expression::Expression, Axes, Command, Gcode},
    parser::{
        test::{permute_whitespace, ExprBuilder, Param},
        toplevel::*,
    },
    GcodeParseError, ParserAllocator,
};

macro_rules! test_parser_impl {
    ($test_func_name:ident, $parser_func_name:ident, $node_type:ty) => {
        #[track_caller]
        pub fn $test_func_name<NodeBuilder>(tokens: &[&str], mut node_builder: NodeBuilder)
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

        macro_rules! $test_func_name {
            ($test_name:ident, $input:expr, $builder:expr) => {
                paste::paste! {
                    #[test]
                    fn [<test_ $parser_func_name _ $test_name>]() {
                        $crate::parser::test::macro_test_parser::$test_func_name(
                            &$input, $builder
                        );
                    }
                }
            };
        }
    };
}

test_parser_impl!(test_parse_expr, parse_expression, &'i Expression<'i>);
test_parser_impl!(test_parse_command, parse_command, Command<'i>);
test_parser_impl!(test_parse_axis, parse_axes, Axes<'i>);
test_parser_impl!(test_parse_param, parse_param, Param<'i>);

fn from_utf8(input: &[u8]) -> &str {
    std::str::from_utf8(input).unwrap()
}
