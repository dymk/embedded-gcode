use crate::enum_value_map;

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
