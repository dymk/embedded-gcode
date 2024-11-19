extern crate std;

use crate::{
    eval::{Eval as _, EvalContext},
    gcode::{expression::Expression, Axes, Command},
    parser::test::{permute_whitespace, ExprBuilder, Param},
    GcodeParseError,
};

use super::NamedParam;

macro_rules! test_parser_impl {
    ($test_func_name:ident, $node_type:ident) => {
        #[track_caller]
        pub fn $test_func_name<IntoNodeType: Into<$node_type>>(
            tokens: &[&str],
            node_builder: impl Fn(&ExprBuilder) -> IntoNodeType,
            context: $crate::parser::test::macro_test_parser::TestContext,
        ) {
            for input in permute_whitespace(tokens) {
                let expr_builder = ExprBuilder::new();
                let expected = node_builder(&expr_builder).into();
                use crate::parser::GcodeParser as _;
                use crate::parser::Input;
                let input = Input::new(input.as_bytes(), &context);
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
                        let context = $crate::parser::test::macro_test_parser::TestContext::default();
                        $crate::parser::test::macro_test_parser::$test_func_name(
                            &$input, $builder, context
                        );
                    }
                }
            };

            ($test_name:ident, $context:expr, $input:expr, $builder:expr) => {
                paste::paste! {
                    #[test]
                    fn [<$test_func_name _ $test_name>]() {
                        $crate::parser::test::macro_test_parser::$test_func_name(
                            &$input, $builder, $context
                        );
                    }
                }
            };
        }
    };
}

use std::collections::HashMap;
use std::prelude::v1::*;

#[derive(Debug, Default, Clone)]
pub struct TestContext {
    const_fold: bool,
    local_params: HashMap<String, f32>,
    global_params: HashMap<String, f32>,
    numbered_params: HashMap<u32, f32>,
}
impl TestContext {
    pub fn const_fold(self, const_fold: bool) -> Self {
        Self { const_fold, ..self }
    }
    pub fn set_local(mut self, name: impl Into<String>, val: f32) -> Self {
        self.local_params.insert(name.into(), val);
        self
    }
    pub fn set_global(mut self, name: impl Into<String>, val: f32) -> Self {
        self.global_params.insert(name.into(), val);
        self
    }
    pub fn set_numbered(mut self, num: u32, val: f32) -> Self {
        self.numbered_params.insert(num, val);
        self
    }
}
impl EvalContext for TestContext {
    fn const_fold(&self) -> bool {
        self.const_fold
    }
    fn get_param(&self, param: &Param) -> Option<f32> {
        match param {
            Param::Numbered(num) => self.numbered_params.get(num).copied(),
            Param::NamedLocal(name) => self.local_params.get(name).copied(),
            Param::NamedGlobal(name) => self.global_params.get(name).copied(),
            Param::Expr(expr) => {
                let expr = expr.eval(self)?;
                self.numbered_params.get(&(expr as u32)).copied()
            }
        }
    }
    fn named_param_exists(&self, param: &NamedParam) -> bool {
        match param {
            NamedParam::NamedLocal(name) => self.local_params.contains_key(name),
            NamedParam::NamedGlobal(name) => self.global_params.contains_key(name),
        }
    }
}

test_parser_impl!(test_parse_axes, Axes);
test_parser_impl!(test_parse_param, Param);
test_parser_impl!(test_parse_command, Command);
test_parser_impl!(test_parse_expr, Expression);
