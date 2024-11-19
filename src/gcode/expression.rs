use super::binop::BinOp;
use crate::enum_value_map;
use crate::eval::{bool_to_float, Eval, EvalContext};
use alloc::{boxed::Box, string::String};
use core::fmt::Debug;
use core::str::from_utf8;
use subenum::subenum;

#[allow(unused_imports)]
use micromath::F32Ext as _;

#[subenum(ExpressionAtom)]
#[derive(PartialEq, Clone)]
pub enum Expression {
    #[subenum(ExpressionAtom)]
    Lit(f32),
    #[subenum(ExpressionAtom)]
    Param(Param),
    #[subenum(ExpressionAtom)]
    FuncCall(FuncCall),
    BinOpExpr {
        op: BinOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}
impl Expression {
    pub fn lit(val: f32) -> Self {
        Self::Lit(val)
    }
    pub fn param(param: impl Into<Param>) -> Self {
        Self::Param(param.into())
    }
    pub fn func_call(func_call: impl Into<FuncCall>) -> Self {
        Self::FuncCall(func_call.into())
    }
    pub fn binop(
        op: impl Into<BinOp>,
        left: impl Into<Expression>,
        right: impl Into<Expression>,
    ) -> Self {
        Self::BinOpExpr {
            op: op.into(),
            left: Box::new(left.into()),
            right: Box::new(right.into()),
        }
    }
}

impl Eval for Expression {
    fn eval(&self, context: &dyn EvalContext) -> Option<f32> {
        match self {
            Self::Lit(val) => Some(*val),
            Self::Param(param) => context.get_param(param),
            Self::FuncCall(func_call) => func_call.eval(context),
            Self::BinOpExpr { op, left, right } => op.eval(left, right, context),
        }
    }
}

impl From<f32> for Expression {
    fn from(val: f32) -> Self {
        Expression::lit(val)
    }
}

impl Debug for Expression {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Lit(arg0) => f.write_fmt(format_args!("{}", arg0)),
            Self::Param(param) => match param {
                Param::Numbered(param_num) => f.write_fmt(format_args!("#{}", param_num)),
                Param::Expr(expr) => f.write_fmt(format_args!("#[{:?}]", expr)),
                Param::NamedLocal(named_local) => f.write_fmt(format_args!("#<{}>", named_local)),
                Param::NamedGlobal(named_global) => {
                    f.write_fmt(format_args!("#<{}>", named_global))
                }
            },
            Self::BinOpExpr { op, left, right } => f.write_fmt(format_args!(
                "({:?} {} {:?})",
                left,
                from_utf8(op.to_value()).unwrap(),
                right
            )),
            Self::FuncCall(func_call) => f.write_fmt(format_args!("{:?}", func_call)),
        }
    }
}

impl Debug for ExpressionAtom {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&Expression::from(self.clone()), f)
    }
}

#[subenum(NamedParam, NumberedParam)]
#[derive(Debug, PartialEq, Clone)]
pub enum Param {
    #[subenum(NamedParam)]
    NamedLocal(String),
    #[subenum(NamedParam)]
    NamedGlobal(String),
    #[subenum(NumberedParam)]
    Numbered(u32),
    #[subenum(NumberedParam)]
    Expr(Box<Expression>),
}

impl Param {
    pub fn named_local(val: impl Into<String>) -> Self {
        Self::NamedLocal(val.into())
    }
    pub fn named_global(val: impl Into<String>) -> Self {
        Self::NamedGlobal(val.into())
    }
    pub fn numbered(val: u32) -> Self {
        Self::Numbered(val)
    }
    pub fn expr(expr: impl Into<Expression>) -> Self {
        Self::Expr(Box::new(expr.into()))
    }
}
impl From<Expression> for Param {
    fn from(expr: Expression) -> Self {
        Param::expr(expr)
    }
}
impl From<u32> for Param {
    fn from(val: u32) -> Self {
        Param::numbered(val)
    }
}

impl From<Param> for Expression {
    fn from(param: Param) -> Self {
        Expression::param(param)
    }
}

impl NamedParam {
    pub fn named_local(val: impl Into<String>) -> Self {
        Self::NamedLocal(val.into())
    }
    pub fn named_global(val: impl Into<String>) -> Self {
        Self::NamedGlobal(val.into())
    }
}
impl NumberedParam {
    pub fn numbered(val: u32) -> Self {
        Self::Numbered(val)
    }
    pub fn expr(expr: impl Into<Expression>) -> Self {
        Self::Expr(Box::new(expr.into()))
    }
}

enum_value_map!(enum UnaryFuncName: &'static [u8] {
    Abs <=> b"ABS",
    Acos <=> b"ACOS",
    Asin <=> b"ASIN",
    Cos <=> b"COS",
    Exp <=> b"EXP",
    Fix <=> b"FIX",
    Fup <=> b"FUP",
    Round <=> b"ROUND",
    Ln <=> b"LN",
    Sin <=> b"SIN",
    Sqrt <=> b"SQRT",
    Tan <=> b"TAN",
});

#[derive(Debug, PartialEq, Clone)]
pub enum FuncCall {
    Exists {
        param: NamedParam,
    },
    Atan {
        arg_y: Box<Expression>,
        arg_x: Box<Expression>,
    },
    Unary {
        name: UnaryFuncName,
        arg: Box<Expression>,
    },
}

impl FuncCall {
    pub fn atan(arg_y: Box<Expression>, arg_x: Box<Expression>) -> Self {
        Self::Atan { arg_y, arg_x }
    }
    pub fn exists(param: NamedParam) -> Self {
        Self::Exists { param }
    }
    pub fn unary(name: UnaryFuncName, arg: Box<Expression>) -> Self {
        Self::Unary { name, arg }
    }

    fn eval_unary_func_call(
        name: UnaryFuncName,
        arg: &Expression,
        context: &dyn EvalContext,
    ) -> Option<f32> {
        let arg = arg.eval(context)?;
        Some(match name {
            UnaryFuncName::Abs => {
                if arg >= 0.0 {
                    arg
                } else {
                    -arg
                }
            }
            UnaryFuncName::Acos => arg.acos(),
            UnaryFuncName::Asin => arg.asin(),
            UnaryFuncName::Cos => arg.cos(),
            UnaryFuncName::Exp => arg.exp(),
            UnaryFuncName::Fix => arg.floor(),
            UnaryFuncName::Fup => arg.ceil(),
            UnaryFuncName::Round => arg.round(),
            UnaryFuncName::Ln => arg.ln(),
            UnaryFuncName::Sin => arg.sin(),
            UnaryFuncName::Sqrt => arg.sqrt(),
            UnaryFuncName::Tan => arg.tan(),
        })
    }
}

impl Eval for FuncCall {
    fn eval(&self, context: &dyn EvalContext) -> Option<f32> {
        match self {
            FuncCall::Atan { arg_y, arg_x } => {
                let arg_y = arg_y.eval(context)?;
                let arg_x = arg_x.eval(context)?;
                Some(arg_y.atan2(arg_x))
            }
            FuncCall::Exists { param } => Some(bool_to_float(context.named_param_exists(param))),
            FuncCall::Unary { name, arg } => Self::eval_unary_func_call(*name, arg, context),
        }
    }
}

#[derive(Debug)]
pub struct BinOpArray<const N: usize>([BinOp; N]);
impl<const N: usize> BinOpArray<N> {
    pub const fn from_list(list: [BinOp; N]) -> Self {
        let list = sort_bin_ops(list);
        Self(list)
    }
}

pub trait BinOpList: Debug {
    fn op_list(&self) -> &[BinOp];
}

impl<const N: usize> BinOpList for BinOpArray<N> {
    fn op_list(&self) -> &[BinOp] {
        &self.0
    }
}

pub const fn sort_bin_ops<const N: usize>(mut arr: [BinOp; N]) -> [BinOp; N] {
    loop {
        let mut swapped = false;
        let mut i = 1;
        while i < arr.len() {
            if arr[i - 1].to_value().len() < arr[i].to_value().len() {
                let left = arr[i - 1];
                let right = arr[i];
                arr[i - 1] = right;
                arr[i] = left;
                swapped = true;
            }
            i += 1;
        }
        if !swapped {
            break;
        }
    }
    arr
}
