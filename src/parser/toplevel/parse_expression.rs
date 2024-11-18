use crate::{
    bind,
    gcode::{expression::*, ArithmeticBinOp, BinOp, CmpBinOp, LogicalBinOp},
    parser::{
        err, fold_many0_result, map_res_f1, map_res_into, ok, parse_utils::space_before,
        IParseResult, Input,
    },
    GcodeParser,
};
use alloc::boxed::Box;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::alpha1,
    combinator::{map_res, not},
    error::{Error, ErrorKind},
    number::complete::float,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    Parser as _,
};

const OPS_L1: BinOpArray<1> = BinOpArray::from_list([BinOp::arithmetic(ArithmeticBinOp::Pow)]);
const OPS_L2: BinOpArray<3> = BinOpArray::from_list([
    BinOp::arithmetic(ArithmeticBinOp::Mul),
    BinOp::arithmetic(ArithmeticBinOp::Div),
    BinOp::arithmetic(ArithmeticBinOp::Mod),
]);
const OPS_L3: BinOpArray<2> = BinOpArray::from_list([
    BinOp::arithmetic(ArithmeticBinOp::Add),
    BinOp::arithmetic(ArithmeticBinOp::Sub),
]);
const OPS_L4: BinOpArray<6> = BinOpArray::from_list([
    BinOp::cmp(CmpBinOp::Eq),
    BinOp::cmp(CmpBinOp::Ne),
    BinOp::cmp(CmpBinOp::Gt),
    BinOp::cmp(CmpBinOp::Ge),
    BinOp::cmp(CmpBinOp::Lt),
    BinOp::cmp(CmpBinOp::Le),
]);
const OPS_L5: BinOpArray<3> = BinOpArray::from_list([
    BinOp::logical(LogicalBinOp::And),
    BinOp::logical(LogicalBinOp::Or),
    BinOp::logical(LogicalBinOp::Xor),
]);
const PRECEDENCE_LIST: [&dyn BinOpList; 5] = [&OPS_L1, &OPS_L2, &OPS_L3, &OPS_L4, &OPS_L5];

impl GcodeParser for Expression {
    fn parse(input: Input) -> IParseResult<'_, Self> {
        parse_expression(input)
    }
}

impl GcodeParser for ExpressionAtom {
    fn parse(input: Input) -> IParseResult<'_, Self> {
        parse_atom(input)
    }
}

fn parse_expression(input: Input) -> IParseResult<'_, Expression> {
    parse_expression_generic(&PRECEDENCE_LIST, input)
}

fn parse_atom(input: Input) -> IParseResult<'_, ExpressionAtom> {
    space_before(alt((
        // function call e.g. `ATAN[..expr..]/[..expr..]`, `COS[..expr..]`
        parse_func_call,
        map_res_f1(Param::parse, ExpressionAtom::Param),
        // number literal e.g. `1.0`
        map_res_f1(float, ExpressionAtom::Lit),
    )))(input)
}

fn parse_func_call(input: Input) -> IParseResult<'_, ExpressionAtom> {
    alt((
        parse_func_call_atan,
        parse_func_call_exists,
        parse_func_call_unary,
    ))(input)
}

fn parse_func_call_atan(input: Input) -> IParseResult<'_, ExpressionAtom> {
    map_res(
        preceded(
            tag_no_case("ATAN"),
            separated_pair(parse_group, space_before(tag("/")), parse_group),
        ),
        move |(arg_y, arg_x)| {
            ok(ExpressionAtom::FuncCall(FuncCall::atan(
                Box::new(arg_y),
                Box::new(arg_x),
            )))
        },
    )
    .parse(input)
}

fn parse_func_call_exists(input: Input) -> IParseResult<'_, ExpressionAtom> {
    map_res(
        preceded(
            tag_no_case("EXISTS"),
            delimited(
                space_before(tag("[")),
                NamedParam::parse,
                space_before(tag("]")),
            ),
        ),
        |param| ok(ExpressionAtom::FuncCall(FuncCall::exists(param))),
    )
    .parse(input)
}

fn parse_func_call_unary(input: Input) -> IParseResult<ExpressionAtom> {
    map_res(
        tuple((parse_unary_func_name, parse_group)),
        |(name, arg)| {
            ok(ExpressionAtom::FuncCall(FuncCall::unary(
                name,
                Box::new(arg),
            )))
        },
    )
    .parse(input)
}

// Parse a (case insensitive) unary function name e.g. `ABS`, `COS`
fn parse_unary_func_name(input: Input) -> IParseResult<'_, UnaryFuncName> {
    // TODO - parse func name using trie
    map_res(alpha1, |name: Input| {
        for func in UnaryFuncName::ALL.iter() {
            if func.to_value().eq_ignore_ascii_case(name.as_bytes()) {
                return ok(*func);
            }
        }
        err(Error::new(name, ErrorKind::Fail))
    })(input)
}

fn parse_group(input: Input) -> IParseResult<'_, Expression> {
    delimited(
        space_before(tag("[")),
        parse_expression,
        space_before(tag("]")),
    )(input)
}

fn parse_factor(input: Input) -> IParseResult<'_, Expression> {
    space_before(alt((map_res_into(parse_atom), parse_group)))(input)
}

fn parse_expression_generic<'a>(
    levels: &'a [&'a dyn BinOpList],
    input: Input<'a>,
) -> IParseResult<'a, Expression> {
    if levels.is_empty() {
        parse_factor(input)
    } else {
        let (next_levels, this_level) = (&levels[0..levels.len() - 1], levels[levels.len() - 1]);
        let (input, init) = parse_expression_generic(next_levels, input)?;
        fold_many0_result(
            pair(
                bind!(this_level, parse_binop),
                bind!(next_levels, parse_expression_generic),
            ),
            move || init.clone(),
            |acc, (bin_op, val)| {
                ok(Expression::BinOpExpr {
                    op: bin_op,
                    left: Box::new(acc),
                    right: Box::new(val),
                })
            },
        )(input)
    }
}

pub fn parse_binop<'a>(ops: &'a dyn BinOpList, input: Input<'a>) -> IParseResult<'a, BinOp> {
    space_before(alt(ops))(input)
}

impl<'a, E> nom::branch::Alt<Input<'a>, BinOp, E> for &dyn BinOpList
where
    E: nom::error::ParseError<Input<'a>>,
{
    fn choice(&mut self, input: Input<'a>) -> nom::IResult<Input<'a>, BinOp, E> {
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
                return Ok((rest, *op));
            }
        }

        Err(Err::Error(ParseError::from_error_kind(
            input,
            ErrorKind::Alt,
        )))
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;

    #[rstest::rstest]
    #[case("+", Some(ArithmeticBinOp::Add), [ArithmeticBinOp::Add.into(), ArithmeticBinOp::Sub.into()])]
    #[case(" +", Some(ArithmeticBinOp::Add), [ArithmeticBinOp::Add.into(), ArithmeticBinOp::Sub.into()])]
    #[case("+", None as Option<BinOp>, [])]
    #[case("+", None as Option<BinOp>, [ArithmeticBinOp::Sub.into()])]
    #[case("-", Some(ArithmeticBinOp::Sub), [ArithmeticBinOp::Sub.into()])]
    #[case("*", Some(ArithmeticBinOp::Mul), [ArithmeticBinOp::Mul.into()])]
    #[case("* ", Some(ArithmeticBinOp::Mul), [ArithmeticBinOp::Mul.into()])]
    #[case(" *", Some(ArithmeticBinOp::Mul), [ArithmeticBinOp::Mul.into()])]
    #[case("**", Some(ArithmeticBinOp::Pow), [ArithmeticBinOp::Pow.into(), ArithmeticBinOp::Mul.into()])]
    #[case("**", Some(ArithmeticBinOp::Pow), [ArithmeticBinOp::Pow.into(), ArithmeticBinOp::Mul.into()])]
    #[case("MOD", Some(ArithmeticBinOp::Mod), [ArithmeticBinOp::Mod.into()])]
    #[case("MOD ", Some(ArithmeticBinOp::Mod), [ArithmeticBinOp::Mod.into()])]
    #[case(" MOD ", Some(ArithmeticBinOp::Mod), [ArithmeticBinOp::Mod.into()])]
    #[case("MOD_", None as Option<BinOp>, [ArithmeticBinOp::Mod.into()])]
    #[case("MODa", None as Option<BinOp>, [ArithmeticBinOp::Mod.into()])]
    #[case("_MOD", None as Option<BinOp>, [ArithmeticBinOp::Mod.into()])]
    fn test_parse_binop<const N: usize>(
        #[case] input: &'static str,
        #[case] expected: Option<impl Into<BinOp>>,
        #[case] allowed: [BinOp; N],
    ) {
        let list = BinOpArray::from_list(allowed);
        match (expected.map(|e| e.into()), parse_binop(&list, input.into())) {
            (Some(expected), Ok((_, parsed))) => assert_eq!(parsed, expected),
            (None, Err(_)) => {}
            (a, b) => panic!("unexpected result: expected={:?} actual={:?}", a, b),
        };
    }
}
