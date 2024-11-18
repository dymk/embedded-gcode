extern crate std;

use crate::{
    gcode::{expression::Expression, Axes, Command},
    parser::test::{permute_whitespace, ExprBuilder, Param},
    GcodeParseError,
};

macro_rules! test_parser_impl {
    ($test_func_name:ident, $node_type:ident) => {
        #[track_caller]
        pub fn $test_func_name<IntoNodeType: Into<$node_type>>(
            tokens: &[&str],
            node_builder: impl Fn(&ExprBuilder) -> IntoNodeType,
        ) {
            for input in permute_whitespace(tokens) {
                let expr_builder = ExprBuilder::new();
                let expected = node_builder(&expr_builder).into();
                use crate::parser::GcodeParser as _;
                use crate::parser::Input;
                let input = Input::from(input.as_str());
                let (rest, actual) = match $node_type::parse(input) {
                    Ok((rest, actual)) => (rest, actual),
                    Err(nom::Err::Error(GcodeParseError::NomError(err))) => {
                        panic!(
                            "[input `{}`] [code {:?}] [rest: `{}`]",
                            input,
                            err.code,
                            err.input.as_utf8().unwrap()
                        )
                    }
                    Err(err) => panic!("{:?}", err),
                };
                assert_eq!(
                    expected.clone(),
                    actual.clone(),
                    "[rest `{}`]",
                    rest.as_utf8().unwrap()
                );
                assert!(
                    rest.iter().all(|b| b.is_ascii_whitespace()),
                    "[rest `{}`]",
                    rest.as_utf8().unwrap()
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
