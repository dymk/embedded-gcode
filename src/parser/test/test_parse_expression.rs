extern crate std;

use crate::{
    gcode::expression::{
        BinOp, BinOpArray, Expression, NamedGlobalParam, NamedLocalParam, NumberedParam, Param,
        UnaryFuncName,
    },
    parser::{nom_types::GcodeParseError, parser_allocator::ParserAllocator, toplevel::*},
};
use core::str::from_utf8;
use nom::error::Error;
use std::prelude::v1::*;

#[rstest::rstest]
#[case("#5", Expression::Param(Param::Numbered(NumberedParam(5))))]
#[case(
    "#<_abc>",
    Expression::Param(Param::NamedGlobal(NamedGlobalParam("_abc")))
)]
#[case("#<foo>", Expression::Param(Param::NamedLocal(NamedLocalParam("foo"))))]
#[case("1", Expression::Lit(1.0))]
#[case("-1.0", Expression::Lit(-1.0))]
fn test_parse_atom(#[case] input: &str, #[case] expected: Expression) {
    let mut heap = bump_into::space_uninit!(1024);
    let alloc = ParserAllocator::new(&mut heap);
    let parsed = match parse_atom(&alloc, input.as_bytes()) {
        Ok((_, parsed)) => parsed,
        Err(nom::Err::Error(GcodeParseError::NomError(Error { input, code }))) => {
            panic!("{:?} {}", code, from_utf8(input).unwrap())
        }
        Err(err) => panic!("{:?}", err),
    };
    assert_eq!(parsed, expected);
}

#[rstest::rstest]
#[case("+", Some(BinOp::Add), [BinOp::Add, BinOp::Sub])]
#[case(" +", Some(BinOp::Add), [BinOp::Add, BinOp::Sub])]
#[case("+", None, [])]
#[case("+", None, [BinOp::Sub])]
#[case("-", Some(BinOp::Sub), [BinOp::Sub])]
#[case("*", Some(BinOp::Mul), [BinOp::Mul])]
#[case("* ", Some(BinOp::Mul), [BinOp::Mul])]
#[case(" *", Some(BinOp::Mul), [BinOp::Mul])]
#[case("**", Some(BinOp::Pow), [BinOp::Mul, BinOp::Pow])]
#[case("**", Some(BinOp::Pow), [BinOp::Pow, BinOp::Mul])]
#[case("MOD", Some(BinOp::Mod), [BinOp::Mod])]
#[case("MOD ", Some(BinOp::Mod), [BinOp::Mod])]
#[case(" MOD ", Some(BinOp::Mod), [BinOp::Mod])]
#[case("MOD_", None, [BinOp::Mod])]
#[case("MODa", None, [BinOp::Mod])]
#[case("_MOD", None, [BinOp::Mod])]
fn test_parse_binop<const N: usize>(
    #[case] input: &'static str,
    #[case] expected: Option<BinOp>,
    #[case] allowed: [BinOp; N],
) {
    let list = BinOpArray::from_list(allowed);
    let mut heap = bump_into::space_uninit!(1024);
    let alloc = ParserAllocator::new(&mut heap);
    match (expected, parse_binop(&alloc, &list, input.as_bytes())) {
        (Some(expected), Ok((_, parsed))) => assert_eq!(parsed, expected),
        (None, Err(_)) => {}
        (a, b) => panic!("unexpected result: expected={:?} actual={:?}", a, b),
    };
}

test_parse_expr!(lit_1, ["1.0"], |b| { b.lit(1.0) });
test_parse_expr!(lit_1_parens, ["[", "1.0", "]"], |b| { b.lit(1.0) });
test_parse_expr!(lit_add_parens, ["[", "1.0", "+", "2.0", "]"], |b| {
    b.binop(b.lit(1.0), "+", b.lit(2.0))
});

test_parse_expr!(num_add_lit, ["#4", "+", "2"], |b| {
    b.binop(b.num_param_expr(4), "+", b.lit(2.0))
});

test_parse_expr!(num_add_lit_parens, ["#4", "+", "[", "2", "]"], |b| {
    b.binop(b.num_param_expr(4), "+", b.lit(2.0))
});

test_parse_expr!(lit_sub_local, ["3", "-", "#<foo>"], |b| {
    b.binop(b.lit(3.0), "-", b.local_param_expr("foo"))
});

test_parse_expr!(lit_mul_sub, ["1", "*", "2", "-", "3"], |b| {
    b.binop(b.binop(b.lit(1.0), "*", b.lit(2.0)), "-", b.lit(3.0))
});

test_parse_expr!(lit_mul_parens, ["1", "*", "[", "2", "-", "3", "]"], |b| {
    b.binop(b.lit(1.0), "*", b.binop(b.lit(2.0), "-", b.lit(3.0)))
});

test_parse_expr!(parens_lit_sub, ["", "[", "1", "*", "2", "]", "-3"], |b| {
    b.binop(b.binop(b.lit(1.0), "*", b.lit(2.0)), "-", b.lit(3.0))
});

test_parse_expr!(
    parens_num_local_sub_global,
    ["[", "#1", "*", "#<foo>", "]", "-#<_bar>"],
    |b| {
        b.binop(
            b.binop(b.num_param_expr(1), "*", b.local_param_expr("foo")),
            "-",
            b.global_param_expr("_bar"),
        )
    }
);

test_parse_expr!(neg_lit_add, ["-1", "+", "2"], |b| {
    b.binop(b.lit(-1.0), "+", b.lit(2.0))
});

test_parse_expr!(lit_add_neg, ["", "2", "+", "-1"], |b| {
    b.binop(b.lit(2.0), "+", b.lit(-1.0))
});

test_parse_expr!(lit_mod_div, ["1", "MOD", "2", "/", "3"], |b| {
    b.binop(b.binop(b.lit(1.0), "MOD", b.lit(2.0)), "/", b.lit(3.0))
});

test_parse_expr!(lit_pow, ["2 ** 3"], |b| b.binop(
    b.lit(2.0),
    "**",
    b.lit(3.0)
));

test_parse_expr!(
    lit_mod_div_parens,
    ["1", "MOD", "[", "2", "/", "3", "**", "4", "]"],
    |b| b.binop(
        b.lit(1.0),
        "MOD",
        b.binop(b.lit(2.0), "/", b.binop(b.lit(3.0), "**", b.lit(4.0)))
    )
);

test_parse_expr!(
    atan_parens,
    ["ATAN", "[", "1.0", "]", "/", "[", "2.0", "]"],
    |b| { b.atan(b.lit(1.0), b.lit(2.0)) }
);

test_parse_expr!(
    atan_lower,
    ["atan", "[", "1.0", "]", "/", "[", "2.0", "]"],
    |b| { b.atan(b.lit(1.0), b.lit(2.0)) }
);

test_parse_expr!(abs_parens, ["ABS", "[", "-1.0", "]"], |b| {
    b.unary(UnaryFuncName::Abs, b.lit(-1.0))
});

test_parse_expr!(abs_parens_lower, ["abs", "[", "-1.0", "]"], |b| {
    b.unary(UnaryFuncName::Abs, b.lit(-1.0))
});
