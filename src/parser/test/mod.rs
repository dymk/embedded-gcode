extern crate std;

#[macro_use]
mod macro_test_parser;
mod test_number_code;
mod test_parse_axes;
mod test_parse_command;
mod test_parse_expression;
mod test_parse_param;

use crate::gcode::{expression::*, BinOp};
pub use macro_test_parser::TestContext;
use std::{collections::HashSet, prelude::v1::*};

struct ExprBuilder {}

impl ExprBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn binop(
        &self,
        left: impl Into<Expression>,
        op: &'static str,
        right: impl Into<Expression>,
    ) -> Expression {
        Expression::BinOpExpr {
            op: BinOp::from_value(op.as_bytes()).unwrap(),
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
    }
    pub fn lit(&self, val: f32) -> Expression {
        Expression::lit(val)
    }
    pub fn num_param_expr(&self, val: u32) -> Expression {
        Expression::param(Param::numbered(val))
    }
    pub fn local_param_expr(&self, val: impl Into<String>) -> Expression {
        Expression::param(Param::named_local(val))
    }
    pub fn global_param_expr(&self, val: impl Into<String>) -> Expression {
        Expression::param(Param::named_global(val))
    }
    pub fn atan(&self, arg_y: impl Into<Expression>, arg_x: impl Into<Expression>) -> Expression {
        Expression::func_call(FuncCall::atan(
            Box::new(arg_y.into()),
            Box::new(arg_x.into()),
        ))
    }
    pub fn unary(&self, name: UnaryFuncName, arg: impl Into<Expression>) -> Expression {
        Expression::func_call(FuncCall::unary(name, Box::new(arg.into())))
    }
    pub fn exists(&self, param: NamedParam) -> Expression {
        Expression::func_call(FuncCall::exists(param))
    }
    pub fn num_param(&self, val: u32) -> NumberedParam {
        NumberedParam::numbered(val)
    }
    pub fn local_param(&self, val: impl Into<String>) -> NamedParam {
        NamedParam::named_local(val)
    }
    pub fn global_param(&self, val: impl Into<String>) -> NamedParam {
        NamedParam::named_global(val)
    }
}

fn permute_whitespace(tokens: &[&str]) -> Vec<String> {
    let tokens = [&[""], tokens, &[""]].concat();
    let mut results = Vec::new();

    for i in 0..(1 << (tokens.len() - 1)) {
        let mut input = Vec::new();
        for j in 0..tokens.len() {
            input.push(tokens[j]);
            if j < tokens.len() - 1 && (i & (1 << j)) != 0 {
                input.push(" ");
            }
        }
        results.push(input.concat());
    }

    results
}

#[test]
fn test_permute_whitespace() {
    let tokens = ["foo"];
    let expected = HashSet::from(["foo", " foo", "foo ", " foo "].map(|s| s.to_string()));
    let actual: HashSet<_> = permute_whitespace(&tokens).into_iter().collect();
    assert_eq!(expected, actual);
}
