use super::{nom_alloc::NomAlloc, nom_types::err, ok, parse_u32, IParseResult};
use crate::{
    gcode::expression::{BinOp, BinOpArray, BinOpList, Expression, FuncCall, UnaryFuncName},
    parser::fold_many0_result::fold_many0_result,
};
use bump_into::BumpInto;
use core::str::from_utf8;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha1, alphanumeric1, space0},
    combinator::{map_res, not, peek, recognize},
    error::{Error, ErrorKind},
    multi::{fold_many0, many0_count},
    number::complete::float,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

#[cfg(test)]
extern crate std;
#[cfg(test)]
macro_rules! println {
    ($($arg:tt)*) => {
        std::println!($($arg)*)
    };
}

#[cfg(not(test))]
macro_rules! println {
    ($($arg:tt)*) => {};
}

fn parse_atom<'a, 'b>(
    bump: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    let atom = alt((
        // function call e.g. `ATAN[..expr..]`
        parse_func_call(bump),
        // global named parameter e.g. `#<_foo>`
        delimited(
            tuple((tag("#<"), peek(tag("_")))),
            parse_named_global_param(bump),
            tag(">"),
        ),
        // local named parameter e.g. `#<foo>`
        delimited(tag("#<"), parse_named_local_param(bump), tag(">")),
        // numbered parameter e.g. `#5`
        preceded(tag("#"), parse_numbered_param()),
        // number literal e.g. `1.0`
        map_res(float, |f| ok(Expression::Lit(f))),
    ));

    preceded(space0, atom)
}

// Parse a (case insensitive) function name e.g. ATAN
fn parse_func_name<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, UnaryFuncName> {
    map_res(alpha1, |name| {
        // do a case insensitive lookup of all the function calls
        for func in UnaryFuncName::ALL.iter() {
            if func.to_value().eq_ignore_ascii_case(name) {
                return ok(func.clone());
            }
        }
        err(Error::new(name, ErrorKind::Fail))
    })
}

fn parse_func_call<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    alt((parse_func_call_atan(alloc), parse_func_call_unary(alloc)))
}

fn parse_func_call_atan<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(
        preceded(
            tag_no_case("ATAN"),
            separated_pair(
                delimited(tag("["), parse_expression(alloc), tag("]")),
                tag("/"),
                delimited(tag("["), parse_expression(alloc), tag("]")),
            ),
        ),
        move |(arg_y, arg_x)| {
            ok(Expression::FuncCall(FuncCall::atan(
                alloc.alloc(arg_y)?,
                alloc.alloc(arg_x)?,
            )))
        },
    )
}

fn parse_func_call_unary<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(
        tuple((
            parse_func_name(),
            delimited(tag("["), parse_expression(alloc), tag("]")),
        )),
        move |(name, arg)| {
            ok(Expression::FuncCall(FuncCall::unary(
                name,
                alloc.alloc(arg)?,
            )))
        },
    )
}

fn parse_group<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    preceded(
        space0,
        delimited(tag("["), parse_expression(alloc), tag("]")),
    )
}

fn parse_factor<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    let factor = alt((parse_atom(alloc), parse_group(alloc)));
    preceded(space0, factor)
}

fn parse_generic_precedence<'a, 'b>(
    alloc: NomAlloc<'b>,
    levels: &'a [&'a dyn BinOpList],
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    move |input| {
        if levels.is_empty() {
            return parse_factor(alloc)(input);
        } else {
            let (next_levels, this_level) =
                (&levels[0..levels.len() - 1], levels[levels.len() - 1]);
            let (input, init) = parse_generic_precedence(alloc, next_levels)(input)?;
            fold_many0_result(
                pair(
                    parse_binop(this_level),
                    parse_generic_precedence(alloc, next_levels),
                ),
                move || init.clone(),
                |acc, (bin_op, val)| {
                    ok(Expression::BinOpExpr {
                        op: bin_op,
                        left: alloc.alloc(acc)?,
                        right: alloc.alloc(val)?,
                    })
                },
            )(input)
        }
    }
}

const OPS_L1: BinOpArray<1> = BinOpArray::from_list([BinOp::Pow]);
const OPS_L2: BinOpArray<3> = BinOpArray::from_list([BinOp::Mul, BinOp::Div, BinOp::Mod]);
const OPS_L3: BinOpArray<2> = BinOpArray::from_list([BinOp::Add, BinOp::Sub]);
const OPS_L4: BinOpArray<6> = BinOpArray::from_list([
    BinOp::Eq,
    BinOp::Ne,
    BinOp::Gt,
    BinOp::Ge,
    BinOp::Lt,
    BinOp::Le,
]);
const OPS_L5: BinOpArray<3> = BinOpArray::from_list([BinOp::And, BinOp::Or, BinOp::Xor]);
const PRECEDENCE_LIST: [&dyn BinOpList; 5] = [&OPS_L1, &OPS_L2, &OPS_L3, &OPS_L4, &OPS_L5];

pub fn parse_expression<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    parse_generic_precedence(alloc, &PRECEDENCE_LIST)
}

fn parse_named_local_param<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(parse_name(alloc), |name| {
        ok(Expression::NamedLocalParam(name))
    })
}

fn parse_named_global_param<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(parse_name(alloc), |name| {
        ok(Expression::NamedGlobalParam(name))
    })
}

fn parse_numbered_param<'a, 'b>() -> impl FnMut(&'a [u8]) -> IParseResult<'_, Expression<'b>> {
    map_res(parse_u32(), |digit| ok(Expression::NumberedParam(digit)))
}

fn parse_name<'a, 'b>(alloc: NomAlloc<'b>) -> impl FnMut(&'a [u8]) -> IParseResult<'a, &'b str> {
    map_res(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0_count(alt((alphanumeric1, tag("_")))),
        )),
        move |bytes| {
            let name_str = from_utf8(bytes)?;
            ok(alloc.alloc_str(name_str)?)
        },
    )
}

impl<'a, 'b, E> nom::branch::Alt<&'a [u8], BinOp, E> for &'b dyn BinOpList
where
    E: nom::error::ParseError<&'a [u8]>,
{
    fn choice(&mut self, input: &'a [u8]) -> nom::IResult<&'a [u8], BinOp, E> {
        use nom::error::ErrorKind;
        use nom::error::ParseError;
        use nom::Err;

        for op in self.op_list().iter() {
            let op_value = op.to_value();
            let result = if op_value[0].is_ascii_alphabetic() {
                // alphabetic operators are case-insensitive and should not be
                // followed by another alphabetic or underscore character
                terminated(tag_no_case(op_value), not(alt((alpha1, tag("_")))))(input)
            } else {
                tag(op_value)(input)
            };
            if let Ok::<_, nom::Err<E>>((rest, _)) = result {
                return Ok((rest, op.clone()));
            }
        }

        Err(Err::Error(ParseError::from_error_kind(
            input,
            ErrorKind::Alt,
        )))
    }
}

fn parse_binop<'a>(ops: &'a dyn BinOpList) -> impl FnMut(&'a [u8]) -> IParseResult<'a, BinOp> {
    preceded(space0, alt(ops))
}

#[cfg(test)]
mod tests {
    extern crate std;
    use core::str::from_utf8;

    use super::{parse_binop, parse_name};
    use crate::{
        gcode::expression::{BinOp, BinOpArray, ExprBuilder, Expression},
        parser::{
            nom_alloc::NomAlloc,
            nom_types::GcodeParseError,
            parse_expression::{parse_atom, parse_expression},
        },
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
        use core::str::from_utf8;

        use nom::error::Error;

        let mut heap = bump_into::space_uninit!(1024);
        let bump = BumpInto::from_slice(heap.as_mut());
        let alloc = NomAlloc::new(&bump);
        let parsed = match parse_atom(alloc)(input.as_bytes()) {
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
        match (expected, parse_binop(&list)(input.as_bytes())) {
            (Some(expected), Ok((_, parsed))) => assert_eq!(parsed, expected),
            (None, Err(_)) => {}
            (a, b) => panic!("unexpected result: expected={:?} actual={:?}", a, b),
        };
    }

    macro_rules! test_expr {
        ($input:literal, $builder:expr, $test_name:ident) => {
            paste! {
                #[test]
                fn [<test_expr_ $test_name>]() {
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
            Err(nom::Err::Error(GcodeParseError::NomError(err))) => panic!(
                "{:?}: unparsed: `{}`",
                err.code,
                from_utf8(err.input).unwrap()
            ),
            Err(err) => panic!("{:?}", err),
        };
        assert_eq!(
            expected,
            &actual,
            "unparsed: `{}`",
            from_utf8(rest).unwrap()
        );
    }
}
