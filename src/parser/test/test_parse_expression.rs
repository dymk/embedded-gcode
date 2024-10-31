extern crate std;

use crate::parser::{
    nom_alloc::NomAlloc,
    nom_types::GcodeParseError,
    toplevel::{parse_atom, parse_binop, parse_name},
};
use crate::{
    gcode::expression::{BinOp, BinOpArray, Expression, UnaryFuncName},
    test_parser,
};
use bump_into::BumpInto;
use core::str::from_utf8;
use nom::error::Error;
use std::prelude::v1::*;

#[rstest::rstest]
#[case(true, "foo")]
#[case(true, "_bar")]
#[case(true, "baz_")]
#[case(false, "0123")]
#[case(true, "_abc123")]
fn test_parse_name(#[case] success: bool, #[case] name: &str) {
    let mut heap = bump_into::space_uninit!(1024);
    let bump = BumpInto::from_slice(heap.as_mut());
    let alloc = NomAlloc::new(&bump);
    let result = parse_name(alloc, name.as_bytes());
    if success {
        let (_, parsed) = result.unwrap();
        assert_eq!(parsed, name);
    } else {
        assert!(result.is_err(), "{:?}", result);
    }
}

#[rstest::rstest]
#[case("#5", Expression::NumberedParam(5))]
#[case("#<_abc>", Expression::NamedGlobalParam("_abc"))]
#[case("#<foo>", Expression::NamedLocalParam("foo"))]
#[case("1", Expression::Lit(1.0))]
#[case("-1.0", Expression::Lit(-1.0))]
fn test_parse_atom(#[case] input: &str, #[case] expected: Expression) {
    let mut heap = bump_into::space_uninit!(1024);
    let bump = BumpInto::from_slice(heap.as_mut());
    let alloc = NomAlloc::new(&bump);
    let parsed = match parse_atom(alloc, input.as_bytes()) {
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
    let bump = BumpInto::from_slice(heap.as_mut());
    let alloc = NomAlloc::new(&bump);
    match (expected, parse_binop(alloc, &list, input.as_bytes())) {
        (Some(expected), Ok((_, parsed))) => assert_eq!(parsed, expected),
        (None, Err(_)) => {}
        (a, b) => panic!("unexpected result: expected={:?} actual={:?}", a, b),
    };
}

test_parser!(expr, lit_1, ["1.0"], |b| { b.lit(1.0) });
test_parser!(expr, lit_1_parens, ["[", "1.0", "]"], |b| { b.lit(1.0) });
test_parser!(expr, lit_add_parens, ["[", "1.0", "+", "2.0", "]"], |b| {
    b.binop(b.lit(1.0), "+", b.lit(2.0))
});

test_parser!(expr, num_add_lit, ["#4", "+", "2"], |b| {
    b.binop(b.num_param(4), "+", b.lit(2.0))
});

test_parser!(expr, num_add_lit_parens, ["#4", "+", "[", "2", "]"], |b| {
    b.binop(b.num_param(4), "+", b.lit(2.0))
});

test_parser!(expr, lit_sub_local, ["3", "-", "#<foo>"], |b| {
    b.binop(b.lit(3.0), "-", b.local_param("foo"))
});

test_parser!(expr, lit_mul_sub, ["1", "*", "2", "-", "3"], |b| {
    b.binop(b.binop(b.lit(1.0), "*", b.lit(2.0)), "-", b.lit(3.0))
});

test_parser!(
    expr,
    lit_mul_parens,
    ["1", "*", "[", "2", "-", "3", "]"],
    |b| { b.binop(b.lit(1.0), "*", b.binop(b.lit(2.0), "-", b.lit(3.0))) }
);

test_parser!(
    expr,
    parens_lit_sub,
    ["", "[", "1", "*", "2", "]", "-3"],
    |b| { b.binop(b.binop(b.lit(1.0), "*", b.lit(2.0)), "-", b.lit(3.0)) }
);

test_parser!(
    expr,
    parens_num_local_sub_global,
    ["[", "#1", "*", "#<foo>", "]", "-#<_bar>"],
    |b| {
        b.binop(
            b.binop(b.num_param(1), "*", b.local_param("foo")),
            "-",
            b.global_param("_bar"),
        )
    }
);

test_parser!(expr, neg_lit_add, ["-1", "+", "2"], |b| {
    b.binop(b.lit(-1.0), "+", b.lit(2.0))
});

test_parser!(expr, lit_add_neg, ["", "2", "+", "-1"], |b| {
    b.binop(b.lit(2.0), "+", b.lit(-1.0))
});

test_parser!(expr, lit_mod_div, ["1", "MOD", "2", "/", "3"], |b| {
    b.binop(b.binop(b.lit(1.0), "MOD", b.lit(2.0)), "/", b.lit(3.0))
});

test_parser!(expr, lit_pow, ["2 ** 3"], |b| b.binop(
    b.lit(2.0),
    "**",
    b.lit(3.0)
));

test_parser!(
    expr,
    lit_mod_div_parens,
    ["1", "MOD", "[", "2", "/", "3", "**", "4", "]"],
    |b| b.binop(
        b.lit(1.0),
        "MOD",
        b.binop(b.lit(2.0), "/", b.binop(b.lit(3.0), "**", b.lit(4.0)))
    )
);

test_parser!(
    expr,
    atan_parens,
    ["ATAN", "[", "1.0", "]", "/", "[", "2.0", "]"],
    |b| { b.atan(b.lit(1.0), b.lit(2.0)) }
);

test_parser!(
    expr,
    atan_lower,
    ["atan", "[", "1.0", "]", "/", "[", "2.0", "]"],
    |b| { b.atan(b.lit(1.0), b.lit(2.0)) }
);

test_parser!(expr, abs_parens, ["ABS", "[", "-1.0", "]"], |b| {
    b.unary(UnaryFuncName::Abs, b.lit(-1.0))
});

test_parser!(expr, abs_parens_lower, ["abs", "[", "-1.0", "]"], |b| {
    b.unary(UnaryFuncName::Abs, b.lit(-1.0))
});
