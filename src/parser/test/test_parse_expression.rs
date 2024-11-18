extern crate std;

use crate::{
    gcode::expression::{Expression, Param, UnaryFuncName},
    parser::nom_types::GcodeParseError,
};

use nom::error::Error;
use std::prelude::v1::*;

#[rstest::rstest]
#[case("#5", Expression::param(Param::numbered(5)))]
#[case("#<_abc>", Expression::param(Param::named_global("_abc")))]
#[case("#<foo>", Expression::param(Param::named_local("foo")))]
#[case("1", Expression::lit(1.0))]
#[case("-1.0", Expression::lit(-1.0))]
fn test_parse_atom(#[case] input: &str, #[case] expected: Expression) {
    use crate::{parser::test::ExpressionAtom, GcodeParser as _};

    let input = input.into();
    let parsed = match ExpressionAtom::parse(input) {
        Ok((_, parsed)) => parsed,
        Err(nom::Err::Error(GcodeParseError::NomError(Error { input, code }))) => {
            panic!("{:?} {}", code, input.as_utf8().unwrap())
        }
        Err(err) => panic!("{:?}", err),
    };
    assert_eq!(parsed, expected);
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
    b.binop(b.lit(3.0), "-", b.local_param_expr("foo".to_string()))
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

test_parse_expr!(exists, ["EXISTS", "[", "#<", "foo", ">", "]"], |b| {
    b.exists(b.local_param("foo"))
});
