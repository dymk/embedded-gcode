extern crate std;

#[macro_use]
mod macro_test_parser;
mod test_number_code;
mod test_parse_axes;
mod test_parse_command;
mod test_parse_expression;
mod test_parse_param;

use crate::gcode::expression::*;
use std::{collections::HashSet, prelude::v1::*};

struct ExprBuilder {}

impl ExprBuilder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn binop(&self, left: Expression, op: &'static str, right: Expression) -> Expression {
        Expression::BinOpExpr {
            op: BinOp::from_value(op.as_bytes()).unwrap(),
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn lit(&self, val: f32) -> Expression {
        Expression::Lit(val)
    }
    pub fn num_param_expr(&self, val: u32) -> Expression {
        Expression::Param(Param::Numbered(NumberedParam(val)))
    }
    pub fn local_param_expr(&self, val: impl Into<String>) -> Expression {
        Expression::Param(Param::NamedLocal(NamedLocalParam(val.into())))
    }
    pub fn global_param_expr(&self, val: impl Into<String>) -> Expression {
        Expression::Param(Param::NamedGlobal(NamedGlobalParam(val.into())))
    }
    pub fn atan(&self, arg_y: Expression, arg_x: Expression) -> Expression {
        Expression::FuncCall(FuncCall::atan(Box::new(arg_y), Box::new(arg_x)))
    }
    pub fn unary(&self, name: UnaryFuncName, arg: Expression) -> Expression {
        Expression::FuncCall(FuncCall::unary(name, Box::new(arg)))
    }
    pub fn num_param(&self, val: u32) -> Param {
        Param::Numbered(NumberedParam(val))
    }
    pub fn local_param(&self, val: impl Into<String>) -> Param {
        Param::NamedLocal(NamedLocalParam(val.into()))
    }
    pub fn global_param(&self, val: impl Into<String>) -> Param {
        Param::NamedGlobal(NamedGlobalParam(val.into()))
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
