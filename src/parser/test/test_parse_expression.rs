extern crate std;

use super::macro_test_parser::TestContext;
use crate::{
    gcode::expression::{Expression, Param, UnaryFuncName},
    parser::{nom_types::GcodeParseError, Input},
    GcodeParser as _,
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
    let context = TestContext::default().const_fold(false);
    let input = Input::new(input.as_bytes(), &context);
    let parsed = match Expression::parse(input) {
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

test_parse_expr!(
    cf_add,
    TestContext::default().const_fold(true),
    ["1", "+", "2"],
    |b| { b.lit(3.0) }
);

test_parse_expr!(
    cf_param_not_exists,
    TestContext::default().const_fold(true),
    ["EXISTS[#<foo>]"],
    |_| Expression::lit(0.0)
);

test_parse_expr!(
    cf_local_param_exists,
    TestContext::default()
        .const_fold(true)
        .set_local("foo", 5.0),
    ["EXISTS[#<foo>]"],
    |_| Expression::lit(1.0)
);

test_parse_expr!(
    cf_global_param_exists,
    TestContext::default()
        .const_fold(true)
        .set_global("_bar", 5.0),
    ["EXISTS[#<_bar>]"],
    |_| Expression::lit(1.0)
);

test_parse_expr!(
    cf_eval_local_param,
    TestContext::default()
        .const_fold(true)
        .set_local("foo", 5.0),
    ["#<foo>"],
    |_| Expression::lit(5.0)
);

test_parse_expr!(
    cf_eval_global_param,
    TestContext::default()
        .const_fold(true)
        .set_global("_bar", 5.0),
    ["#<_bar>"],
    |_| Expression::lit(5.0)
);

test_parse_expr!(
    cf_eval_numbered_param,
    TestContext::default().const_fold(true).set_numbered(1, 2.0),
    ["#1"],
    |_| Expression::lit(2.0)
);

test_parse_expr!(
    cf_eval_indirect_numbered_param,
    TestContext::default()
        .const_fold(true)
        .set_numbered(1, 2.0)
        .set_numbered(2, 3.0),
    ["##1"],
    |_| Expression::lit(3.0)
);

test_parse_expr!(
    cf_eval_indirect_numbered_param_binop,
    TestContext::default()
        .const_fold(true)
        .set_numbered(1, 2.0)
        .set_numbered(2, 3.0)
        .set_numbered(5, 10.0),
    ["#[#1 + #2]"],
    |_| Expression::lit(10.0)
);
