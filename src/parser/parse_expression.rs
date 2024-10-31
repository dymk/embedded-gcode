use super::parse_utils::space_before;
use crate::Parser;
use crate::{
    gcode::expression::{BinOp, BinOpArray, BinOpList, Expression, FuncCall, UnaryFuncName},
    parser::{fold_many0_result, nom_types::err, ok, parse_utils::parse_u32, IParseResult},
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

impl<'b> Parser<'b> {
    pub fn parse_expression<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Expression<'b>> {
        self.parse_expression_generic(input, &PRECEDENCE_LIST)
    }

    pub fn parse_atom<'a>(&'b self) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        let atom = alt((
            // function call e.g. `ATAN[..expr..]/[..expr..]`, `COS[..expr..]`
            self.parse_func_call(),
            // global named parameter e.g. `#<_foo>`
            delimited(
                tuple((tag("#<"), peek(tag("_")))),
                self.parse_named_global_param(),
                tag(">"),
            ),
            // local named parameter e.g. `#<foo>`
            delimited(tag("#<"), self.parse_named_local_param(), tag(">")),
            // numbered parameter e.g. `#5`
            preceded(tag("#"), self.parse_numbered_param()),
            // number literal e.g. `1.0`
            map_res(float, |f| ok(Expression::Lit(f))),
        ));

        preceded(space0, atom)
    }

    fn parse_func_call<'a>(&'b self) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        alt((self.parse_func_call_atan(), self.parse_func_call_unary()))
    }

    fn parse_func_call_atan<'a>(
        &'b self,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        map_res(
            preceded(
                tag_no_case("ATAN"),
                separated_pair(
                    |input| self.parse_expr_in_brackets(input),
                    space_before(tag("/")),
                    |input| self.parse_expr_in_brackets(input),
                ),
            ),
            move |(arg_y, arg_x)| {
                ok(Expression::FuncCall(FuncCall::atan(
                    self.alloc.alloc(arg_y)?,
                    self.alloc.alloc(arg_x)?,
                )))
            },
        )
    }

    fn parse_func_call_unary<'a>(
        &'b self,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        map_res(
            tuple((self.parse_unary_func_name(), |input| {
                self.parse_expr_in_brackets(input)
            })),
            move |(name, arg)| {
                ok(Expression::FuncCall(FuncCall::unary(
                    name,
                    self.alloc.alloc(arg)?,
                )))
            },
        )
    }

    // Parse a (case insensitive) unary function name e.g. `ABS`, `COS`
    fn parse_unary_func_name<'a>(
        &'b self,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, UnaryFuncName> {
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

    fn parse_group<'a>(&'b self) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        delimited(
            space_before(tag("[")),
            |input| self.parse_expression(input),
            space_before(tag("]")),
        )
    }

    fn parse_factor<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Expression<'b>> {
        let factor = alt((self.parse_atom(), self.parse_group()));
        preceded(space0, factor)(input)
    }

    fn parse_expression_generic<'a>(
        &'b self,
        input: &'a [u8],
        levels: &'a [&'a dyn BinOpList],
    ) -> IParseResult<'a, Expression<'b>> {
        if levels.is_empty() {
            return self.parse_factor(input);
        } else {
            let (next_levels, this_level) =
                (&levels[0..levels.len() - 1], levels[levels.len() - 1]);
            let (input, init) = self.parse_expression_generic(input, next_levels)?;
            fold_many0_result(
                pair(self.parse_binop(this_level), |input| {
                    self.parse_expression_generic(input, next_levels)
                }),
                move || init.clone(),
                |acc, (bin_op, val)| {
                    ok(Expression::BinOpExpr {
                        op: bin_op,
                        left: self.alloc.alloc(acc)?,
                        right: self.alloc.alloc(val)?,
                    })
                },
            )(input)
        }
    }

    fn parse_named_local_param<'a>(
        &'b self,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        map_res(self.parse_name(), |name| {
            ok(Expression::NamedLocalParam(name))
        })
    }

    fn parse_named_global_param<'a>(
        &'b self,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        map_res(self.parse_name(), |name| {
            ok(Expression::NamedGlobalParam(name))
        })
    }

    fn parse_numbered_param<'a>(
        &'b self,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, Expression<'b>> {
        map_res(parse_u32(), |digit| ok(Expression::NumberedParam(digit)))
    }

    pub fn parse_name<'a>(&'b self) -> impl FnMut(&'a [u8]) -> IParseResult<'a, &'b str> {
        map_res(
            recognize(pair(
                alt((alpha1, tag("_"))),
                many0_count(alt((alphanumeric1, tag("_")))),
            )),
            move |bytes| {
                let name_str = from_utf8(bytes)?;
                ok(self.alloc.alloc_str(name_str)?)
            },
        )
    }

    pub fn parse_binop<'a>(
        &'b self,
        ops: &'a dyn BinOpList,
    ) -> impl FnMut(&'a [u8]) -> IParseResult<'a, BinOp> {
        preceded(space0, alt(ops))
    }

    fn parse_expr_in_brackets<'a>(&'b self, input: &'a [u8]) -> IParseResult<'a, Expression<'b>> {
        delimited(
            space_before(tag("[")),
            |input| self.parse_expression(input),
            space_before(tag("]")),
        )(input)
    }
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
