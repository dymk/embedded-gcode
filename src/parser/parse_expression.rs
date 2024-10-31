use crate::{
    gcode::expression::{BinOp, BinOpArray, BinOpList, Expression, FuncCall, UnaryFuncName},
    parser::{
        fold_many0_result, nom_alloc::NomAlloc, nom_types::err, ok, parse_utils::parse_u32,
        IParseResult,
    },
};
use core::str::from_utf8;
use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case},
    character::complete::{alpha1, alphanumeric1, space0},
    combinator::{map_res, not, peek, recognize},
    error::{Error, ErrorKind},
    multi::many0_count,
    number::complete::float,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
};

use super::parse_utils::space_before;

pub fn parse_atom<'a, 'b>(
    bump: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    let atom = alt((
        // function call e.g. `ATAN[..expr..]/[..expr..]`, `COS[..expr..]`
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
                parse_expr_in_brackets(alloc),
                space_before(tag("/")),
                parse_expr_in_brackets(alloc),
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
        tuple((parse_unary_func_name(), parse_expr_in_brackets(alloc))),
        move |(name, arg)| {
            ok(Expression::FuncCall(FuncCall::unary(
                name,
                alloc.alloc(arg)?,
            )))
        },
    )
}

// Parse a (case insensitive) unary function name e.g. `ABS`, `COS`
fn parse_unary_func_name<'a>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, UnaryFuncName> {
    // TODO - parse func name using trie
    map_res(alpha1, |name| {
        for func in UnaryFuncName::ALL.iter() {
            if func.to_value().eq_ignore_ascii_case(name) {
                return ok(*func);
            }
        }
        err(Error::new(name, ErrorKind::Fail))
    })
}

fn parse_group<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    delimited(
        space_before(tag("[")),
        parse_expression(alloc),
        space_before(tag("]")),
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

fn parse_numbered_param<'a, 'b>() -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    map_res(parse_u32(), |digit| ok(Expression::NumberedParam(digit)))
}

pub fn parse_name<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, &'b str> {
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

pub fn parse_binop<'a>(ops: &'a dyn BinOpList) -> impl FnMut(&'a [u8]) -> IParseResult<'a, BinOp> {
    preceded(space0, alt(ops))
}

fn parse_expr_in_brackets<'a, 'b>(
    alloc: NomAlloc<'b>,
) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
    delimited(
        space_before(tag("[")),
        parse_expression(alloc),
        space_before(tag("]")),
    )
}
