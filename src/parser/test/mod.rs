extern crate std;

#[macro_use]
mod macro_test_parser;
mod test_number_code;
mod test_parse_axes;
mod test_parse_command;
mod test_parse_expression;
mod test_parse_param;

use crate::{gcode::expression::*, ParserAllocator};
use std::{collections::HashSet, prelude::v1::*};

pub struct ExprBuilder<'b> {
    alloc: &'b ParserAllocator<'b>,
}

impl<'b> ExprBuilder<'b> {
    pub fn new(alloc: &'b ParserAllocator<'b>) -> Self {
        Self { alloc }
    }

    pub fn binop(
        &'b self,
        left: Expression<'b>,
        op: &'static str,
        right: Expression<'b>,
    ) -> Expression<'b> {
        Expression::BinOpExpr {
            op: BinOp::from_value(op.as_bytes()).unwrap(),
            left: self.alloc.alloc(left).unwrap(),
            right: self.alloc.alloc(right).unwrap(),
        }
    }
    pub fn lit(&'b self, val: f32) -> Expression<'b> {
        Expression::Lit(val)
    }
    pub fn num_param_expr(&'b self, val: u32) -> Expression<'b> {
        Expression::Param(Param::Numbered(NumberedParam(val)))
    }
    pub fn local_param_expr(&'b self, val: &'b str) -> Expression<'b> {
        Expression::Param(Param::NamedLocal(NamedLocalParam(val)))
    }
    pub fn global_param_expr(&'b self, val: &'b str) -> Expression<'b> {
        Expression::Param(Param::NamedGlobal(NamedGlobalParam(val)))
    }
    pub fn atan(&'b self, arg_y: Expression<'b>, arg_x: Expression<'b>) -> Expression<'b> {
        Expression::FuncCall(FuncCall::atan(
            self.alloc.alloc(arg_y).unwrap(),
            self.alloc.alloc(arg_x).unwrap(),
        ))
    }
    pub fn unary(&'b self, name: UnaryFuncName, arg: Expression<'b>) -> Expression<'b> {
        Expression::FuncCall(FuncCall::unary(name, self.alloc.alloc(arg).unwrap()))
    }
    pub fn num_param(&'b self, val: u32) -> &'b Param<'b> {
        self.alloc
            .alloc(Param::Numbered(NumberedParam(val)))
            .unwrap()
    }
    pub fn local_param(&'b self, val: &'b str) -> &'b Param<'b> {
        self.alloc
            .alloc(Param::NamedLocal(NamedLocalParam(val)))
            .unwrap()
    }
    pub fn global_param(&'b self, val: &'b str) -> &'b Param<'b> {
        self.alloc
            .alloc(Param::NamedGlobal(NamedGlobalParam(val)))
            .unwrap()
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
