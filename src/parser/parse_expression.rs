use super::{ok, parse_u32, IParseResult};
use crate::{
    enum_value_map::EnumValueMap,
    gcode::expression::{BinOp, BinOpList, Expression},
};
use bump_into::BumpInto;
use core::str::from_utf8;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, alphanumeric1, multispace0, one_of},
    combinator::{map_res, peek, recognize},
    error::{Error, ErrorKind},
    multi::{fold_many0, many0_count},
    number::complete::float,
    sequence::{delimited, pair, preceded, tuple},
};

fn parse_atom<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    alt((
        // global named parameter
        delimited(
            tuple((tag("#<"), peek(tag("_")))),
            parse_named_global_param(bump),
            tag(">"),
        ),
        // local named parameter
        delimited(tag("#<"), parse_named_local_param(bump), tag(">")),
        // numbered parameter
        preceded(tag("#"), parse_numbered_param()),
        // number literal
        map_res(float, |f| ok(Expression::Lit(f))),
    ))
}

fn parse_group<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    delimited(
        multispace0,
        delimited(tag("["), parse_expression(bump), tag("]")),
        multispace0,
    )
}

fn parse_factor<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    alt((parse_atom(bump), parse_group(bump)))
}

fn parse_generic_precedence<'a, 'b>(
    bump: &'b BumpInto<'b>,
    levels: &'static [&'static [BinOp]],
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    |input| {
        if levels.is_empty() {
            return parse_factor(bump)(input);
        } else {
            let (this_level, next_levels) = (levels[0], &levels[1..]);
            let (input, init) = parse_generic_precedence(bump, next_levels)(input)?;

            fold_many0(
                pair(
                    parse_binop(this_level),
                    parse_generic_precedence(bump, next_levels),
                ),
                move || init.clone(),
                |acc, (bin_op, val)| Expression::BinOpExpr {
                    op: bin_op,
                    left: bump.alloc(acc).unwrap(),
                    right: bump.alloc(val).unwrap(),
                },
            )(input)
        }
    }
}

pub fn parse_expression<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    parse_generic_precedence(
        bump,
        &[&[BinOp::Add, BinOp::Sub], &[BinOp::Mul, BinOp::Div]],
    )
}

fn parse_named_local_param<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(parse_name(bump), |name| {
        ok(Expression::NamedLocalParam(name))
    })
}

fn parse_named_global_param<'a, 'b>(
    bump: &'b BumpInto<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(parse_name(bump), |name| {
        ok(Expression::NamedGlobalParam(name))
    })
}

fn parse_numbered_param<'a, 'b>() -> impl FnMut(&'a [u8]) -> IParseResult<'_, Expression<'b>> {
    map_res(parse_u32(), |digit| ok(Expression::NumberedParam(digit)))
}

fn parse_name<'a, 'b>(bump: &'b BumpInto<'b>) -> impl FnMut(&'a [u8]) -> IParseResult<'a, &'b str> {
    map_res(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
        |bytes| {
            let name_str = match from_utf8(bytes) {
                Ok(name_str) => name_str,
                Err(_) => return Err(Error::new(bytes, ErrorKind::Fail)),
            };
            ok(bump.alloc_copy_concat_strs(&[name_str]).unwrap() as &'b str)
        },
    )
}

fn parse_binop<'a>(chars: &'static [BinOp]) -> impl FnMut(&'a [u8]) -> IParseResult<'a, BinOp> {
    map_res(one_of(BinOpList(chars)), |op| {
        match BinOp::from_value(op as u8) {
            Some(op) => ok(op),
            None => Err(Error::new(&[], ErrorKind::Fail)),
        }
    })
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::{parse_binop, parse_name};
    use crate::{
        gcode::expression::{BinOp, ExprBuilder, Expression},
        parser::parse_expression::{parse_atom, parse_expression},
    };
    use bump_into::BumpInto;
    use paste::paste;

    #[rstest::rstest]
    #[case(true, "foo")]
    #[case(true, "_bar")]
    #[case(true, "baz_")]
    #[case(false, "0123")]
    #[case(true, "_abc123")]
    fn test_parse_name(#[case] success: bool, #[case] name: &str) {
        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let result = parse_name(&bump)(name.as_bytes());
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
        use core::str::from_utf8;

        use nom::error::Error;

        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let parsed = match parse_atom(&bump)(input.as_bytes()) {
            Ok((_, parsed)) => parsed,
            Err(nom::Err::Error(Error { input, code })) => {
                panic!("{:?} {}", code, from_utf8(input).unwrap())
            }
            Err(err) => panic!("{:?}", err),
        };
        assert_eq!(parsed, expected);
    }

    #[rstest::rstest]
    #[case("+", Some(BinOp::Add), &[BinOp::Add, BinOp::Sub])]
    #[case("+", None, &[])]
    #[case("+", None, &[BinOp::Sub])]
    #[case("-", Some(BinOp::Sub), &[BinOp::Sub])]
    fn test_parse_binop(
        #[case] input: &'static str,
        #[case] expected: Option<BinOp>,
        #[case] allowed: &'static [BinOp],
    ) {
        match (expected, parse_binop(allowed)(input.as_bytes())) {
            (Some(expected), Ok((_, parsed))) => assert_eq!(parsed, expected),
            (None, Err(_)) => {}
            _ => panic!("unexpected result"),
        }
    }

    macro_rules! test_expr {
        ($input:literal, $builder:expr, $test_name:ident) => {
            paste! {
                #[test]
                fn [<test_expr_ $test_name>]() {
                    test_expr($input, $builder);
                }
            }
        };
    }

    test_expr!("1.0", |b| b.lit(1.0), lit_1);

    test_expr!("[1.0]", |b| b.lit(1.0), lit_parens);

    test_expr!(
        "[1.0+2.0]",
        |b| b.binop(b.lit(1.0), '+', b.lit(2.0)),
        lit_add_parens
    );

    test_expr!(
        "#4+2",
        |b| b.binop(b.num_param(4), '+', b.lit(2.0)),
        num_add_lit
    );

    test_expr!(
        "#4+[2]",
        |b| b.binop(b.num_param(4), '+', b.lit(2.0)),
        num_add_lit_parens
    );

    test_expr!(
        "3-#<foo>",
        |b| { b.binop(b.lit(3.0), '-', b.local_param("foo")) },
        lit_sub_local
    );

    test_expr!(
        "1*2-3",
        |b| { b.binop(b.binop(b.lit(1.0), '*', b.lit(2.0)), '-', b.lit(3.0)) },
        lit_mul_sub
    );

    test_expr!(
        "1*[2-3]",
        |b| { b.binop(b.lit(1.0), '*', b.binop(b.lit(2.0), '-', b.lit(3.0))) },
        lit_mul_parens
    );

    test_expr!(
        "[1*2]-3",
        |b| { b.binop(b.binop(b.lit(1.0), '*', b.lit(2.0)), '-', b.lit(3.0)) },
        parens_lit_sub
    );

    test_expr!(
        "[#1*#<foo>]-#<_bar>",
        |b| {
            b.binop(
                b.binop(b.num_param(1), '*', b.local_param("foo")),
                '-',
                b.global_param("_bar"),
            )
        },
        parens_num_local_sub_global
    );

    test_expr!(
        "-1+2",
        |b| b.binop(b.lit(-1.0), '+', b.lit(2.0)),
        neg_lit_add
    );

    test_expr!(
        "2+-1",
        |b| b.binop(b.lit(2.0), '+', b.lit(-1.0)),
        lit_add_neg
    );

    #[track_caller]
    fn test_expr<'b, F>(input: &'b str, builder: F)
    where
        F: for<'a> Fn(&'a ExprBuilder<'a>) -> &'a Expression<'a>,
    {
        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let expr_builder = ExprBuilder::new(&bump);
        let expected = builder(&expr_builder);
        let (rest, actual) = parse_expression(&bump)(input.as_bytes()).unwrap();
        assert_eq!(
            expected,
            &actual,
            "unparsed: {:?}",
            std::str::from_utf8(rest).unwrap()
        );
    }
}
