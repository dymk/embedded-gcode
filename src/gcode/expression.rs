use crate::enum_value_map;
#[cfg(test)]
use bump_into::BumpInto;
use core::fmt::Debug;
use core::str::from_utf8;

#[derive(PartialEq, Clone)]
pub enum Expression<'b> {
    Lit(f32),
    Param(Param<'b>),
    BinOpExpr {
        op: BinOp,
        left: &'b Expression<'b>,
        right: &'b Expression<'b>,
    },
    FuncCall(FuncCall<'b>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Param<'b> {
    Numbered(NumberedParam),
    NamedLocal(NamedLocalParam<'b>),
    NamedGlobal(NamedGlobalParam<'b>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum NamedParam<'b> {
    NamedLocal(NamedLocalParam<'b>),
    NamedGlobal(NamedGlobalParam<'b>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct NumberedParam(pub u32);
#[derive(Debug, PartialEq, Clone)]
pub struct NamedLocalParam<'b>(pub &'b str);
#[derive(Debug, PartialEq, Clone)]
pub struct NamedGlobalParam<'b>(pub &'b str);

impl Debug for Expression<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Lit(arg0) => f.write_fmt(format_args!("{}", arg0)),
            Self::Param(param) => match param {
                Param::Numbered(numbered) => f.write_fmt(format_args!("#{}", numbered.0)),
                Param::NamedLocal(named_local) => f.write_fmt(format_args!("#<{}>", named_local.0)),
                Param::NamedGlobal(named_global) => {
                    f.write_fmt(format_args!("#<{}>", named_global.0))
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

#[cfg(test)]
impl<'b> Expression<'b> {
    pub fn bump(self, bump: &'b BumpInto<'b>) -> &'b Self {
        bump.alloc(self).unwrap()
    }
}

enum_value_map!(enum BinOp: &'static [u8] {
    Pow <=> b"**",

    Mul <=> b"*",
    Div <=> b"/",
    Mod <=> b"MOD",

    Add <=> b"+",
    Sub <=> b"-",

    Eq <=> b"EQ",
    Ne <=> b"NE",
    Gt <=> b"GT",
    Ge <=> b"GE",
    Lt <=> b"LT",
    Le <=> b"LE",

    And <=> b"AND",
    Or <=> b"OR",
    Xor <=> b"XOR",
});

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
pub enum FuncCall<'b> {
    Exists {
        param: NamedParam<'b>,
    },
    Atan {
        arg_y: &'b Expression<'b>,
        arg_x: &'b Expression<'b>,
    },
    Unary {
        name: UnaryFuncName,
        arg: &'b Expression<'b>,
    },
}

impl<'b> FuncCall<'b> {
    pub fn atan(arg_y: &'b Expression<'b>, arg_x: &'b Expression<'b>) -> Self {
        Self::Atan { arg_y, arg_x }
    }
    pub fn unary(name: UnaryFuncName, arg: &'b Expression<'b>) -> Self {
        Self::Unary { name, arg }
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
