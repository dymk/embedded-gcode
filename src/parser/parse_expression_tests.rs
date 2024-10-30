extern crate std;

use crate::{
    gcode::expression::{BinOp, BinOpArray, ExprBuilder, Expression, UnaryFuncName},
    parser::{nom_alloc::NomAlloc, nom_types::GcodeParseError, parse_expression::*},
    permute_whitespace,
};
use bump_into::BumpInto;
use nom::error::Error;
use paste::paste;
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
    let result = parse_name(alloc)(name.as_bytes());
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
    let parsed = match parse_atom(alloc)(input.as_bytes()) {
        Ok((_, parsed)) => parsed,
        Err(nom::Err::Error(GcodeParseError::NomError(Error { input, code }))) => {
            panic!("{:?} {}", code, from_utf8(input))
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
    match (expected, parse_binop(&list)(input.as_bytes())) {
        (Some(expected), Ok((_, parsed))) => assert_eq!(parsed, expected),
        (None, Err(_)) => {}
        (a, b) => panic!("unexpected result: expected={:?} actual={:?}", a, b),
    };
}

#[rstest::rstest]
#[case(&["[", "1.0", "]"])]
#[case(&["1.0", "+", "2.0"])]
#[case(&["ATAN", "[", "1.0", "]", "/", "[", "2.0", "]"])]
fn test_handle_whitespace(#[case] tokens: &[&str]) {
    let tokens = [&[""], tokens, &[""]].concat();
    let inputs = permute_whitespace(&tokens);
    for input in inputs {
        test_handle_whitespace_impl(&input);
    }

    fn test_handle_whitespace_impl(input: &str) {
        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let (rest, _) = match parse_expression(alloc)(input.as_bytes()) {
            Ok(result) => result,
            Err(nom::Err::Error(GcodeParseError::NomError(Error { input: rest, code }))) => {
                panic!(
                    "failed for input `{}`: rest: `{}`, code: {:?}",
                    input,
                    from_utf8(rest),
                    code
                )
            }
            Err(err) => panic!("{:?}", err),
        };

        assert!(
            rest.iter().all(|&b| b.is_ascii_whitespace()),
            "rest is not whitespace: `{}`",
            from_utf8(rest)
        );
    }
}

macro_rules! test_expr {
    ($input:literal, $builder:expr, $test_name:ident) => {
        paste! {
            #[test]
            fn [<test_parse_expression_ $test_name>]() {
                test_expr_impl($input, $builder);
            }
        }
    };
}

test_expr!("1.0", |b| b.lit(1.0), lit_1);

test_expr!(" 1.0", |b| b.lit(1.0), lit_1_spaces);

test_expr!("[1.0]", |b| b.lit(1.0), lit_parens);

test_expr!(
    "[1.0+2.0]",
    |b| b.binop(b.lit(1.0), "+", b.lit(2.0)),
    lit_add_parens
);

test_expr!(
    "#4+2",
    |b| b.binop(b.num_param(4), "+", b.lit(2.0)),
    num_add_lit
);

test_expr!(
    "#4+[2]",
    |b| b.binop(b.num_param(4), "+", b.lit(2.0)),
    num_add_lit_parens
);

test_expr!(
    "3-#<foo>",
    |b| { b.binop(b.lit(3.0), "-", b.local_param("foo")) },
    lit_sub_local
);

test_expr!(
    "1*2-3",
    |b| { b.binop(b.binop(b.lit(1.0), "*", b.lit(2.0)), "-", b.lit(3.0)) },
    lit_mul_sub
);

test_expr!(
    "1*[2-3]",
    |b| { b.binop(b.lit(1.0), "*", b.binop(b.lit(2.0), "-", b.lit(3.0))) },
    lit_mul_parens
);

test_expr!(
    "[1*2]-3",
    |b| { b.binop(b.binop(b.lit(1.0), "*", b.lit(2.0)), "-", b.lit(3.0)) },
    parens_lit_sub
);

test_expr!(
    "[#1*#<foo>]-#<_bar>",
    |b| {
        b.binop(
            b.binop(b.num_param(1), "*", b.local_param("foo")),
            "-",
            b.global_param("_bar"),
        )
    },
    parens_num_local_sub_global
);

test_expr!(
    "-1+2",
    |b| b.binop(b.lit(-1.0), "+", b.lit(2.0)),
    neg_lit_add
);

test_expr!(
    "2+-1",
    |b| b.binop(b.lit(2.0), "+", b.lit(-1.0)),
    lit_add_neg
);

test_expr!(
    "1 MOD 2 / 3",
    |b| b.binop(b.binop(b.lit(1.0), "MOD", b.lit(2.0)), "/", b.lit(3.0)),
    lit_mod_div
);

test_expr!("2 ** 3", |b| b.binop(b.lit(2.0), "**", b.lit(3.0)), lit_pow);

test_expr!(
    "1 MOD [2 / 3 ** 4]",
    |b| b.binop(
        b.lit(1.0),
        "MOD",
        b.binop(b.lit(2.0), "/", b.binop(b.lit(3.0), "**", b.lit(4.0)))
    ),
    lit_mod_div_parens
);

test_expr!(
    "ATAN[1.0]/[2.0]",
    |b| b.atan(b.lit(1.0), b.lit(2.0)),
    atan_parens
);

test_expr!(
    "atan[1.0]/[2.0]",
    |b| b.atan(b.lit(1.0), b.lit(2.0)),
    atan_parens_lower
);

test_expr!(
    "ABS[-1.0]",
    |b| b.unary(UnaryFuncName::Abs, b.lit(-1.0)),
    abs_parens
);

test_expr!(
    "abs[-1.0]",
    |b| b.unary(UnaryFuncName::Abs, b.lit(-1.0)),
    abs_parens_lower
);

#[track_caller]
fn test_expr_impl<'b, F>(input: &'b str, builder: F)
where
    F: for<'a> Fn(&'a ExprBuilder<'a>) -> &'a Expression<'a>,
{
    let mut heap = bump_into::space_uninit!(1024);
    let bump = BumpInto::from_slice(heap.as_mut());
    let alloc = NomAlloc::new(&bump);
    let expr_builder = ExprBuilder::new(&bump);
    let expected = builder(&expr_builder);
    let (rest, actual) = match parse_expression(alloc)(input.as_bytes()) {
        Ok((rest, actual)) => (rest, actual),
        Err(nom::Err::Error(GcodeParseError::NomError(err))) => {
            panic!("{:?}: unparsed: `{}`", err.code, from_utf8(err.input))
        }
        Err(err) => panic!("{:?}", err),
    };
    assert_eq!(expected, &actual, "unparsed: `{}`", from_utf8(rest));
}

fn from_utf8(input: &[u8]) -> &str {
    std::str::from_utf8(input).unwrap()
}
