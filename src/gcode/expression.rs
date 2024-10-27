use core::fmt::Debug;

use bump_into::BumpInto;

use crate::enum_value_map;
use crate::enum_value_map::EnumValueMap as _;

#[derive(PartialEq, Clone)]
pub enum Expression<'b> {
    Lit(f32),
    NumberedParam(u32),
    NamedLocalParam(&'b str),
    NamedGlobalParam(&'b str),
    BinOpExpr {
        op: BinOp,
        left: &'b Expression<'b>,
        right: &'b Expression<'b>,
    },
}

impl<'b> Debug for Expression<'b> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Lit(arg0) => f.write_fmt(format_args!("{}", arg0)),
            Self::NumberedParam(arg0) => f.write_fmt(format_args!("#{}", arg0)),
            Self::NamedLocalParam(arg0) => f.write_fmt(format_args!("#<{}>", arg0)),
            Self::NamedGlobalParam(arg0) => f.write_fmt(format_args!("#<{}>", arg0)),
            Self::BinOpExpr { op, left, right } => f.write_fmt(format_args!(
                "({:?} {} {:?})",
                left,
                op.to_value() as char,
                right
            )),
        }
    }
}

impl<'b> Expression<'b> {
    pub fn bump(self, bump: &'b BumpInto<'b>) -> &'b Self {
        bump.alloc(self).unwrap()
    }
}

enum_value_map!(enum BinOp: u8 {
    Add <=> b'+',
    Sub <=> b'-',
    Mul <=> b'*',
    Div <=> b'/',
});

impl<'a> nom::FindToken<u8> for BinOpList<'a> {
    fn find_token(&self, token: u8) -> bool {
        let op = match BinOp::from_value(token) {
            Some(op) => op,
            None => return false,
        };
        self.0.contains(&op)
    }
}

pub struct BinOpList<'a>(pub &'a [BinOp]);

#[cfg(test)]
pub struct ExprBuilder<'b> {
    bump: &'b BumpInto<'b>,
}
#[cfg(test)]
impl<'b> ExprBuilder<'b> {
    pub fn new(bump: &'b BumpInto<'b>) -> Self {
        Self { bump }
    }

    pub fn binop(
        &self,
        left: &'b Expression,
        op: char,
        right: &'b Expression,
    ) -> &'b Expression<'b> {
        Expression::BinOpExpr {
            op: BinOp::from_value(op as u8).unwrap(),
            left,
            right,
        }
        .bump(self.bump)
    }
    pub fn lit(&self, val: f32) -> &'b Expression<'b> {
        Expression::Lit(val).bump(self.bump)
    }
    pub fn num_param(&self, val: u32) -> &'b Expression<'b> {
        Expression::NumberedParam(val).bump(self.bump)
    }
    pub fn local_param(&self, val: &'b str) -> &'b Expression<'b> {
        Expression::NamedLocalParam(val).bump(self.bump)
    }
    pub fn global_param(&self, val: &'b str) -> &'b Expression<'b> {
        Expression::NamedGlobalParam(val).bump(self.bump)
    }
}
