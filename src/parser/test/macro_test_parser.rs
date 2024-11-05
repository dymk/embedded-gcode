extern crate std;

use crate::{
    gcode::{expression::Expression, Axes, Command},
    parser::test::{permute_whitespace, ExprBuilder, Param},
    GcodeParseError, ParserAllocator,
};

macro_rules! test_parser_impl {
    ($test_func_name:ident, $node_type:ident) => {
        #[track_caller]
        pub fn $test_func_name(
            tokens: &[&str],
            node_builder: impl for<'i> Fn(&'i ExprBuilder<'i>) -> $node_type<'i>,
        ) {
            for input in permute_whitespace(tokens) {
                let mut heap = bump_into::space_uninit!(1024);
                let alloc = ParserAllocator::new(&mut heap);
                let expr_builder = ExprBuilder::new(&alloc);
                let expected = node_builder(&expr_builder);
                use crate::gcode::GcodeParser;
                let (rest, actual) = match $node_type::parse(&alloc, input.as_bytes()) {
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
                    fn [<$test_func_name _ $test_name>]() {
                        $crate::parser::test::macro_test_parser::$test_func_name(
                            &$input, $builder
                        );
                    }
                }
            };
        }
    };
}

test_parser_impl!(test_parse_axes, Axes);
test_parser_impl!(test_parse_param, Param);
test_parser_impl!(test_parse_command, Command);
test_parser_impl!(test_parse_expr, Expression);

fn from_utf8(input: &[u8]) -> &str {
    std::str::from_utf8(input).unwrap()
}
