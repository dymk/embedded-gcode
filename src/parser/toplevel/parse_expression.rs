use crate::{
    bind,
    gcode::expression::*,
    parser::{
        err, fold_many0_result, map_res_f1, ok, parse_utils::space_before, toplevel::*,
        IParseResult,
    },
    ParserAllocator,
};
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::alpha1,
    combinator::{map_res, not},
    error::{Error, ErrorKind},
    number::complete::float,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
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

pub fn parse_expression<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Expression<'b>> {
    parse_expression_generic(alloc, &PRECEDENCE_LIST, input)
}

pub fn parse_atom<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Expression<'b>> {
    space_before(alt((
        // function call e.g. `ATAN[..expr..]/[..expr..]`, `COS[..expr..]`
        bind!(alloc, parse_func_call),
        map_res_f1(bind!(alloc, parse_param), Expression::Param),
        // number literal e.g. `1.0`
        map_res_f1(float, Expression::Lit),
    )))(input)
}

fn parse_func_call<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Expression<'b>> {
    alt((parse_func_call_atan(alloc), parse_func_call_unary(alloc)))(input)
}

fn parse_func_call_atan<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(
        preceded(
            tag_no_case("ATAN"),
            separated_pair(
                bind!(alloc, parse_expr_in_brackets),
                space_before(tag("/")),
                bind!(alloc, parse_expr_in_brackets),
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
    alloc: &'b ParserAllocator<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(
        tuple((
            bind!(alloc, parse_unary_func_name),
            bind!(alloc, parse_expr_in_brackets),
        )),
        move |(name, arg)| {
            ok(Expression::FuncCall(FuncCall::unary(
                name,
                alloc.alloc(arg)?,
            )))
        },
    )
}

// Parse a (case insensitive) unary function name e.g. `ABS`, `COS`
fn parse_unary_func_name<'a, 'b>(
    _: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, UnaryFuncName> {
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

fn parse_group<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Expression<'b>> {
    delimited(
        space_before(tag("[")),
        bind!(alloc, parse_expression),
        space_before(tag("]")),
    )(input)
}

fn parse_factor<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Expression<'b>> {
    space_before(alt((bind!(alloc, parse_atom), bind!(alloc, parse_group))))(input)
}

fn parse_expression_generic<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    levels: &'a [&'a dyn BinOpList],
    input: &'a [u8],
) -> IParseResult<'a, Expression<'b>> {
    if levels.is_empty() {
        parse_factor(alloc, input)
    } else {
        let (next_levels, this_level) = (&levels[0..levels.len() - 1], levels[levels.len() - 1]);
        let (input, init) = parse_expression_generic(alloc, next_levels, input)?;
        fold_many0_result(
            pair(
                bind!(alloc, this_level, parse_binop),
                bind!(alloc, next_levels, parse_expression_generic),
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

pub fn parse_binop<'a, 'b>(
    _: &'b ParserAllocator<'b>,
    ops: &'a dyn BinOpList,
    input: &'a [u8],
) -> IParseResult<'a, BinOp> {
    space_before(alt(ops))(input)
}

fn parse_expr_in_brackets<'a, 'b>(
    alloc: &'b ParserAllocator<'b>,
    input: &'a [u8],
) -> IParseResult<'a, Expression<'b>> {
    delimited(
        space_before(tag("[")),
        bind!(alloc, parse_expression),
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
