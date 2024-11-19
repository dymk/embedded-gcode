use crate::{
    enum_value_map,
    eval::{bool_to_float, Eval, EvalContext},
};

use super::expression::Expression;

enum_value_map!(enum LogicalBinOp: &'static [u8] {
    And <=> b"AND",
    Or <=> b"OR",
    Xor <=> b"XOR",
});

enum_value_map!(enum CmpBinOp: &'static [u8] {
    Eq <=> b"EQ",
    Ne <=> b"NE",
    Gt <=> b"GT",
    Ge <=> b"GE",
    Lt <=> b"LT",
    Le <=> b"LE",
});

enum_value_map!(enum ArithmeticBinOp: &'static [u8] {
    Pow <=> b"**",
    Mul <=> b"*",
    Div <=> b"/",
    Mod <=> b"MOD",
    Add <=> b"+",
    Sub <=> b"-",
});

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BinOp {
    Logical(LogicalBinOp),
    Cmp(CmpBinOp),
    Arithmetic(ArithmeticBinOp),
}

impl BinOp {
    pub const fn logical(val: LogicalBinOp) -> Self {
        Self::Logical(val)
    }
    pub const fn cmp(val: CmpBinOp) -> Self {
        Self::Cmp(val)
    }
    pub const fn arithmetic(val: ArithmeticBinOp) -> Self {
        Self::Arithmetic(val)
    }

    pub const fn from_value(value: &'static [u8]) -> Option<Self> {
        if let Some(op) = LogicalBinOp::from_value(value) {
            return Some(Self::logical(op));
        }
        if let Some(op) = CmpBinOp::from_value(value) {
            return Some(Self::cmp(op));
        }
        if let Some(op) = ArithmeticBinOp::from_value(value) {
            return Some(Self::arithmetic(op));
        }
        None
    }
    pub const fn to_value(self) -> &'static [u8] {
        match self {
            Self::Logical(val) => val.to_value(),
            Self::Cmp(val) => val.to_value(),
            Self::Arithmetic(val) => val.to_value(),
        }
    }

    pub fn eval(
        &self,
        left: &Expression,
        right: &Expression,
        context: &dyn EvalContext,
    ) -> Option<f32> {
        #[allow(unused_imports)]
        use micromath::F32Ext as _;

        let left = left.eval(context)?;
        match self {
            BinOp::Logical(op) => {
                let left = left != 0.;
                let right = right.eval(context)? != 0.;
                Some(bool_to_float(match op {
                    LogicalBinOp::And => left && right,
                    LogicalBinOp::Or => left || right,
                    LogicalBinOp::Xor => left ^ right,
                }))
            }
            BinOp::Cmp(op) => {
                let right = right.eval(context)?;
                Some(bool_to_float(match op {
                    CmpBinOp::Eq => left == right,
                    CmpBinOp::Ne => left != right,
                    CmpBinOp::Gt => left > right,
                    CmpBinOp::Ge => left >= right,
                    CmpBinOp::Lt => left < right,
                    CmpBinOp::Le => left <= right,
                }))
            }
            BinOp::Arithmetic(op) => {
                let right = right.eval(context)?;
                Some(match op {
                    ArithmeticBinOp::Pow => left.powf(right),
                    ArithmeticBinOp::Mul => left * right,
                    ArithmeticBinOp::Div => left / right,
                    ArithmeticBinOp::Mod => left % right,
                    ArithmeticBinOp::Add => left + right,
                    ArithmeticBinOp::Sub => left - right,
                })
            }
        }
    }
}

impl From<LogicalBinOp> for BinOp {
    fn from(val: LogicalBinOp) -> Self {
        BinOp::logical(val)
    }
}
impl From<CmpBinOp> for BinOp {
    fn from(val: CmpBinOp) -> Self {
        BinOp::cmp(val)
    }
}
impl From<ArithmeticBinOp> for BinOp {
    fn from(val: ArithmeticBinOp) -> Self {
        BinOp::arithmetic(val)
    }
}
