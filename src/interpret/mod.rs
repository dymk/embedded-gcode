use alloc::string::ToString;
use hashbrown::hash_map::Entry::{Occupied, Vacant};
use model_state::ModelState;
use nom::number;

use crate::gcode::{
    expression::{BinOp, Expression, FuncCall, NamedParam, Param, UnaryFuncName},
    Command,
};

mod interpret_gcode;
mod model_state;
#[cfg(test)]
mod test;

#[derive(Debug, Default)]
pub struct GCodeInterpreter<'a> {
    local_vars_numbered: hashbrown::HashMap<u32, f32>,
    local_vars_named: hashbrown::HashMap<&'a str, f32>,
    global_vars: hashbrown::HashMap<&'a str, f32>,
    model_state: model_state::ModelState,
}

#[derive(Debug)]
pub enum InterpretError<'a> {
    ParamNotFound(Param<'a>),
}

impl<'a> GCodeInterpreter<'a> {
    pub fn interpret(
        &mut self,
        model_state: &mut ModelState,
        command: Command<'a>,
    ) -> Result<(), InterpretError<'a>> {
        match command {
            Command::Comment(_) => todo!(),
            Command::Assign(to, from) => self.interpret_assign(model_state, to, from),
            Command::G(gcode) => self.interpret_gcode(model_state, gcode),
            Command::M(_) => todo!(),
            Command::O(_) => todo!(),
            Command::S(_) => todo!(),
            Command::T(_) => todo!(),
        }
    }

    fn interpret_assign(
        &mut self,
        _: &mut ModelState,
        to: Param<'a>,
        from: Expression<'a>,
    ) -> Result<(), InterpretError<'a>> {
        let from = self.eval_expr(&from);
        let to = self
            .get_param_mut(&to)
            .ok_or(InterpretError::ParamNotFound(to))?;
        *to = from;
        Ok(())
    }

    fn get_param_mut<'p>(&mut self, _: &'p Param) -> Option<&mut f32> {
        todo!()
    }

    fn get_param(&self, param: &Param) -> Option<f32> {
        match param {
            Param::Numbered(numbered_param) => self.local_vars_numbered.get(&numbered_param.0),
            Param::NamedLocal(named_local_param) => self.local_vars_named.get(named_local_param.0),
            Param::NamedGlobal(named_global_param) => self.global_vars.get(named_global_param.0),
        }
        .copied()
    }

    fn eval_expr(&self, expression: &Expression) -> f32 {
        match expression {
            Expression::Lit(value) => *value,
            Expression::Param(param) => self.get_param(param).unwrap_or(0.0),
            Expression::BinOpExpr { op, left, right } => self.eval_binop_expr(*op, left, right),
            Expression::FuncCall(func_call) => self.eval_func_call(func_call),
        }
    }

    fn eval_binop_expr(&self, op: BinOp, left: &Expression, right: &Expression) -> f32 {
        let left = self.eval_expr(left);

        match op {
            BinOp::And => return cast_f32(left != 0. && self.eval_expr(right) != 0.),
            BinOp::Or => return cast_f32(left != 0. || self.eval_expr(right) != 0.),
            BinOp::Xor => return todo!(),
            _ => {}
        };

        let right = self.eval_expr(right);
        match op {
            BinOp::Pow => todo!(),
            BinOp::Mul => left * right,
            BinOp::Div => left / right,
            BinOp::Mod => left % right,
            BinOp::Add => left + right,
            BinOp::Sub => left - right,
            BinOp::Eq => cast_f32(left == right),
            BinOp::Ne => cast_f32(left != right),
            BinOp::Gt => cast_f32(left > right),
            BinOp::Ge => cast_f32(left >= right),
            BinOp::Lt => cast_f32(left < right),
            BinOp::Le => cast_f32(left <= right),
            _ => unreachable!(),
        }
    }

    fn eval_func_call(&self, func: &FuncCall) -> f32 {
        match func {
            FuncCall::Atan { arg_y, arg_x } => todo!(),
            FuncCall::Exists { param } => self.eval_exists_func_call(param),
            FuncCall::Unary { name, arg } => self.eval_unary_func_call(*name, arg),
        }
    }

    fn eval_unary_func_call(&self, name: UnaryFuncName, arg: &Expression) -> f32 {
        let arg = self.eval_expr(arg);
        match name {
            UnaryFuncName::Abs => {
                if arg >= 0.0 {
                    arg
                } else {
                    -arg
                }
            }
            UnaryFuncName::Acos => todo!(),
            UnaryFuncName::Asin => todo!(),
            UnaryFuncName::Cos => todo!(),
            UnaryFuncName::Exp => todo!(),
            UnaryFuncName::Fix => todo!(),
            UnaryFuncName::Fup => todo!(),
            UnaryFuncName::Round => todo!(),
            UnaryFuncName::Ln => todo!(),
            UnaryFuncName::Sin => todo!(),
            UnaryFuncName::Sqrt => todo!(),
            UnaryFuncName::Tan => todo!(),
        }
    }

    fn eval_exists_func_call(&self, param: &NamedParam) -> f32 {
        cast_f32(match param {
            NamedParam::NamedLocal(named_local_param) => {
                self.local_vars_named.contains_key(named_local_param.0)
            }
            NamedParam::NamedGlobal(named_global_param) => {
                self.global_vars.contains_key(named_global_param.0)
            }
        })
    }
}

#[inline(always)]
fn cast_f32(expr: bool) -> f32 {
    if expr {
        1.0
    } else {
        0.0
    }
}
