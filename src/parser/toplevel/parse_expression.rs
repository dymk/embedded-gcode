use crate::{
    bind,
    gcode::{expression::*, GcodeParser},
    parser::{err, fold_many0_result, map_res_f1, ok, parse_utils::space_before, IParseResult},
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

impl GcodeParser for Expression {
    fn parse<'i>(input: &'i [u8]) -> IParseResult<'i, Self> {
        parse_expression(input)
    }
}

fn parse_expression<'i>(input: &'i [u8]) -> IParseResult<'i, Expression> {
    parse_expression_generic(&PRECEDENCE_LIST, input)
}

pub fn parse_atom<'a>(input: &'a [u8]) -> IParseResult<'a, Expression> {
    space_before(alt((
        // function call e.g. `ATAN[..expr..]/[..expr..]`, `COS[..expr..]`
        parse_func_call,
        map_res_f1(Param::parse, Expression::Param),
        // number literal e.g. `1.0`
        map_res_f1(float, Expression::Lit),
    )))(input)
}

fn parse_func_call<'a>(input: &'a [u8]) -> IParseResult<'a, Expression> {
    alt((parse_func_call_atan, parse_func_call_unary)).parse(input)
}

fn parse_func_call_atan<'i>(input: &'i [u8]) -> IParseResult<'i, Expression> {
    map_res(
        preceded(
            tag_no_case("ATAN"),
            separated_pair(
                parse_expr_in_brackets,
                space_before(tag("/")),
                parse_expr_in_brackets,
            ),
        ),
        move |(arg_y, arg_x)| {
            ok(Expression::FuncCall(FuncCall::atan(
                Box::new(arg_y),
                Box::new(arg_x),
            )))
        },
    )
    .parse(input)
}

fn parse_func_call_unary<'a, 'b>(input: &'a [u8]) -> IParseResult<'a, Expression> {
    map_res(
        tuple((parse_unary_func_name, parse_expr_in_brackets)),
        move |(name, arg)| ok(Expression::FuncCall(FuncCall::unary(name, Box::new(arg)))),
    )
    .parse(input)
}

// Parse a (case insensitive) unary function name e.g. `ABS`, `COS`
fn parse_unary_func_name<'a>(input: &'a [u8]) -> IParseResult<'a, UnaryFuncName> {
    // TODO - parse func name using trie
    map_res(alpha1, |name| {
        for func in UnaryFuncName::ALL.iter() {
            if func.to_value().eq_ignore_ascii_case(name) {
                return ok(*func);
            }
        }
        err(Error::new(name, ErrorKind::Fail))
    })(input)
}

fn parse_group<'a>(input: &'a [u8]) -> IParseResult<'a, Expression> {
    delimited(
        space_before(tag("[")),
        parse_expression,
        space_before(tag("]")),
    )(input)
}

fn parse_factor<'a>(input: &'a [u8]) -> IParseResult<'a, Expression> {
    space_before(alt((parse_atom, parse_group)))(input)
}

fn parse_expression_generic<'a, 'b>(
    levels: &'a [&'a dyn BinOpList],
    input: &'a [u8],
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

pub fn parse_binop<'a>(ops: &'a dyn BinOpList, input: &'a [u8]) -> IParseResult<'a, BinOp> {
    space_before(alt(ops))(input)
}

fn parse_expr_in_brackets<'a>(input: &'a [u8]) -> IParseResult<'a, Expression> {
    delimited(
        space_before(tag("[")),
        parse_expression,
        space_before(tag("]")),
    )(input)
}

impl<'a, E> nom::branch::Alt<&'a [u8], BinOp, E> for &dyn BinOpList
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
                return Ok((rest, *op));
            }
        }

        Err(Err::Error(ParseError::from_error_kind(
            input,
            ErrorKind::Alt,
        )))
    }
}
