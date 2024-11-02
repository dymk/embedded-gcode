#[macro_use]
mod macro_test_parser;
mod test_number_code;
mod test_parse_axis;
mod test_parse_command;
mod test_parse_expression;
mod test_parse_param;

use crate::{gcode::expression::*, ParserAllocator};

extern crate std;
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
        left: &'b Expression,
        op: &'static str,
        right: &'b Expression,
    ) -> &'b Expression<'b> {
        self.alloc
            .alloc(Expression::BinOpExpr {
                op: BinOp::from_value(op.as_bytes()).unwrap(),
                left,
                right,
            })
            .unwrap()
    }
    pub fn lit(&'b self, val: f32) -> &'b Expression<'b> {
        self.alloc.alloc(Expression::Lit(val)).unwrap()
    }
    pub fn num_param_expr(&'b self, val: u32) -> &'b Expression<'b> {
        self.alloc
            .alloc(Expression::Param(Param::Numbered(NumberedParam(val))))
            .unwrap()
    }
    pub fn local_param_expr(&'b self, val: &'b str) -> &'b Expression<'b> {
        self.alloc
            .alloc(Expression::Param(Param::NamedLocal(NamedLocalParam(val))))
            .unwrap()
    }
    pub fn global_param_expr(&'b self, val: &'b str) -> &'b Expression<'b> {
        self.alloc
            .alloc(Expression::Param(Param::NamedGlobal(NamedGlobalParam(val))))
            .unwrap()
    }
    pub fn atan(&'b self, arg_y: &'b Expression, arg_x: &'b Expression) -> &'b Expression<'b> {
        self.alloc
            .alloc(Expression::FuncCall(FuncCall::atan(arg_y, arg_x)))
            .unwrap()
    }
    pub fn unary(&'b self, name: UnaryFuncName, arg: &'b Expression) -> &'b Expression<'b> {
        self.alloc
            .alloc(Expression::FuncCall(FuncCall::unary(name, arg)))
            .unwrap()
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
